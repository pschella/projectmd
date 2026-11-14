// Command pmdgen writes a synthetic projectmd tree for benchmarking.
//
// The tree is a complete F-ary tree of depth D: the root plus D levels of
// sub-projects, so the file count is (F^(D+1)-1)/(F-1) for F > 1. Each
// project lives in its own folder (root.md at the top, then p1/index.md,
// p1/p2/index.md, ...). Output is deterministic for a given seed.
//
// Phase one plans the whole tree in memory (commitments, milestones, public
// ids) so that cross-file references can point at any project. Phase two
// renders and writes each file.
package main

import (
	"bytes"
	"flag"
	"fmt"
	"math/rand"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"time"
)

const dateLayout = "2006-01-02"

var (
	baseEnd = time.Date(2028, 6, 30, 0, 0, 0, 0, time.UTC)

	nouns = []string{
		"Battery pack", "Cooling loop", "Inverter", "E-motor", "Charging module",
		"Wiring harness", "Chassis frame", "Brake system", "Infotainment unit",
		"Sensor suite", "Gearbox", "Suspension", "Steering unit", "Lighting",
		"Body frame", "Seat system", "Climate control", "Power distribution",
		"Software stack", "Telematics",
	}
	qualifiers = []string{
		"platform", "variant A", "variant B", "phase 2", "prototype",
		"series build", "supplier kit", "validation",
	}
	verbs = []string{
		"Draft", "Review", "Validate", "Prototype", "Qualify", "Document",
		"Source", "Test", "Calibrate", "Release", "Simulate", "Procure",
	}
	objects = []string{
		"thermal envelope", "harness routing", "mounting points", "test rig",
		"supplier samples", "durability cycle", "EMC baseline", "cost model",
		"interface spec", "load cases", "sign-off checklist", "bill of materials",
	}
	milestoneTitles = []string{
		"Requirements frozen", "Design freeze", "Prototype built",
		"Validation complete", "Supplier qualified", "Pilot release",
		"Series readiness", "Integration sign-off",
	}
	firstNames = []string{
		"Anna", "Bram", "Carla", "Daan", "Eva", "Femke", "Gijs", "Hanna", "Ivo",
		"Joris", "Kira", "Luuk", "Mila", "Noah", "Olga", "Priya", "Quinn", "Ruben",
		"Sanne", "Tomas",
	}
	lastNames = []string{
		"de Boer", "Visser", "Jansen", "Bakker", "Smit", "Meijer", "Mulder", "Vos",
		"Peters", "Hendriks", "Dekker", "Kowalski", "Nair", "Schell", "Wagner", "Rossi",
	}
	states = []string{"in-development", "planned", "in-review", "approved"}
)

type milestone struct {
	id     string
	title  string
	date   time.Time
	public bool
}

type node struct {
	folder   string // "" for the root, else e.g. "p1/p3"
	file     string // "root.md" or "p1/p3/index.md", slash separated
	title    string
	owner    string // "Name <email>"
	parent   int
	children []int
	start    time.Time
	end      time.Time
	by       time.Time // end date the parent asks of this project
	ms       []milestone
	pubs     []int // indexes into ms of #public milestones
}

// builder plans the tree and all milestones.
type builder struct {
	rng    *rand.Rand
	fanout int
	depth  int
	nodes  []*node
}

func (b *builder) add(parent int, folder string, level int, by, end time.Time) int {
	r := b.rng
	idx := len(b.nodes)
	n := &node{folder: folder, parent: parent, by: by, end: end}
	n.start = end.AddDate(0, 0, -(180 + r.Intn(360)))
	label := folder
	if parent < 0 {
		n.file = "root.md"
		label = "root"
	} else {
		n.file = path.Join(folder, "index.md")
	}
	n.title = fmt.Sprintf("%s, %s (%s)", nouns[r.Intn(len(nouns))], qualifiers[r.Intn(len(qualifiers))], label)

	first := firstNames[r.Intn(len(firstNames))]
	last := lastNames[r.Intn(len(lastNames))]
	n.owner = fmt.Sprintf("%s %s <%s.%s@example.com>", first, last,
		strings.ToLower(first), strings.ToLower(strings.ReplaceAll(last, " ", "")))

	m := 2 + r.Intn(3)    // 2..4 milestones
	npub := 1 + r.Intn(2) // one or two of them public
	perm := r.Perm(len(milestoneTitles))
	span := int(end.Sub(n.start).Hours() / 24)
	n.ms = make([]milestone, m)
	for k := range n.ms {
		n.ms[k] = milestone{
			title:  milestoneTitles[perm[k]],
			date:   end.AddDate(0, 0, -r.Intn(span+1)),
			public: k < npub,
		}
	}
	sort.SliceStable(n.ms, func(i, j int) bool { return n.ms[i].date.Before(n.ms[j].date) })
	for k := range n.ms {
		n.ms[k].id = fmt.Sprintf("m%d-%s", k+1, strings.ToLower(strings.ReplaceAll(n.ms[k].title, " ", "-")))
		if n.ms[k].public {
			n.pubs = append(n.pubs, k)
		}
	}

	b.nodes = append(b.nodes, n)
	if parent >= 0 {
		b.nodes[parent].children = append(b.nodes[parent].children, idx)
	}
	if level < b.depth {
		for i := 1; i <= b.fanout; i++ {
			cfolder := path.Join(folder, "p"+strconv.Itoa(i))
			cby := end.AddDate(0, 0, -(7 + r.Intn(54))) // parent asks within its end
			cend := cby.AddDate(0, 0, -r.Intn(31))      // child commits before the ask
			b.add(idx, cfolder, level+1, cby, cend)
		}
	}
	return idx
}

var handles []string

func init() {
	for _, f := range firstNames {
		handles = append(handles, strings.ToLower(f))
	}
}

// writer renders files from the planned tree.
type writer struct {
	rng       *rand.Rand
	nodes     []*node
	cur       int
	items     int
	xref      float64
	buf       bytes.Buffer
	local     []string // ids defined so far in the current file
	counter   int
	remaining int
}

// crossRef returns a link to a random #public milestone in another project.
func (w *writer) crossRef() string {
	t := w.rng.Intn(len(w.nodes) - 1)
	if t >= w.cur {
		t++
	}
	self := w.nodes[w.cur]
	target := w.nodes[t]
	m := target.ms[target.pubs[w.rng.Intn(len(target.pubs))]]
	rel, err := filepath.Rel(filepath.FromSlash(path.Dir(self.file)), filepath.FromSlash(target.file))
	if err != nil {
		panic(err)
	}
	return fmt.Sprintf("[%s](%s#%s)", m.title, filepath.ToSlash(rel), m.id)
}

// emitTask writes one task line (and maybe nested sub-tasks), consuming budget.
func (w *writer) emitTask(indent string, sub bool) {
	r := w.rng
	w.counter++
	w.remaining--

	title := verbs[r.Intn(len(verbs))] + " " + objects[r.Intn(len(objects))]
	dur := fmt.Sprintf("%dw", 1+r.Intn(12))
	if sub {
		dur = fmt.Sprintf("%dd", 2+r.Intn(19))
	}
	fmt.Fprintf(&w.buf, "%s- [ ] %s  %s", indent, title, dur)

	id := ""
	if r.Float64() < 0.4 {
		id = "t" + strconv.Itoa(w.counter)
		fmt.Fprintf(&w.buf, "  ^%s", id)
	}
	if len(w.local) > 0 && r.Float64() < 0.3 {
		fmt.Fprintf(&w.buf, "  after: ^%s", w.local[r.Intn(len(w.local))])
	}
	if len(w.nodes) > 1 && r.Float64() < w.xref {
		fmt.Fprintf(&w.buf, "  after: %s", w.crossRef())
	}

	// Owner: inline on the line, as a property bullet, or none.
	switch o := r.Float64(); {
	case o < 0.3:
		fmt.Fprintf(&w.buf, "  owner: @%s", handles[r.Intn(len(handles))])
		w.buf.WriteByte('\n')
	case o < 0.4 && !sub:
		w.buf.WriteByte('\n')
		fmt.Fprintf(&w.buf, "%s  - owner: @%s\n", indent, handles[r.Intn(len(handles))])
	default:
		w.buf.WriteByte('\n')
	}
	if id != "" {
		w.local = append(w.local, id)
	}

	if !sub && w.remaining > 0 && r.Float64() < 0.2 {
		k := 1 + r.Intn(2)
		for j := 0; j < k && w.remaining > 0; j++ {
			w.emitTask(indent+"  ", true)
		}
	}
}

// render produces the bytes of file i.
func (w *writer) render(i int) []byte {
	n := w.nodes[i]
	w.cur = i
	w.buf.Reset()
	w.local = w.local[:0]
	w.counter = 0
	w.remaining = w.items
	r := w.rng
	b := &w.buf

	fmt.Fprintf(b, "# %s\n\n", n.title)
	fmt.Fprintf(b, "- Owner: %s\n", n.owner)
	fmt.Fprintf(b, "- Commitment: %s .. %s\n", n.start.Format(dateLayout), n.end.Format(dateLayout))
	fmt.Fprintf(b, "- State: %s\n", states[r.Intn(len(states))])
	if r.Float64() < 0.1 {
		label := n.folder
		if n.folder == "" {
			label = "root"
		}
		fmt.Fprintf(b, "- ID: gen.example.com/%s\n", label)
	}

	b.WriteString("\nGenerated benchmark project. Schedule and scope are synthetic.\n\n")
	b.WriteString("## Key dates\n\n")
	for _, m := range n.ms {
		fmt.Fprintf(b, "- [ ] %s %s", m.date.Format(dateLayout), m.title)
		if m.public {
			b.WriteString("  #public")
		}
		fmt.Fprintf(b, "  ^%s\n", m.id)
		w.local = append(w.local, m.id)
	}

	b.WriteString("\n## Work\n\n")
	for w.remaining > 0 {
		w.emitTask("", false)
	}

	if len(n.children) > 0 {
		b.WriteString("\n## Sub-projects\n\n")
		for _, c := range n.children {
			ch := w.nodes[c]
			rel, err := filepath.Rel(filepath.FromSlash(path.Dir(n.file)), filepath.FromSlash(ch.file))
			if err != nil {
				panic(err)
			}
			fmt.Fprintf(b, "- [ ] [%s](%s)  by: %s\n", ch.title, filepath.ToSlash(rel), ch.by.Format(dateLayout))
		}
	}

	b.WriteString("\n## Risks\n\n")
	b.WriteString("Supplier lead times may slip if qualification samples arrive late.\n\n")
	b.WriteString("An interface change after the design freeze would move the pilot release.\n")
	return b.Bytes()
}

func fail(format string, args ...interface{}) {
	fmt.Fprintf(os.Stderr, "pmdgen: "+format+"\n", args...)
	os.Exit(2)
}

func main() {
	out := flag.String("out", "", "output directory (required)")
	depth := flag.Int("depth", 3, "levels below the root project")
	fanout := flag.Int("fanout", 3, "sub-projects per project")
	items := flag.Int("items", 10, "tasks per project, including nested sub-tasks")
	xrefs := flag.Float64("xrefs", 0, "probability per task of a cross-file after: reference")
	seed := flag.Int64("seed", 1, "random seed")
	flag.Parse()

	if *out == "" {
		fail("-out is required")
	}
	if *depth < 0 || *fanout < 1 || *items < 0 {
		fail("need -depth >= 0, -fanout >= 1, -items >= 0")
	}
	if *xrefs < 0 || *xrefs > 1 {
		fail("-xrefs must be in [0,1]")
	}

	start := time.Now()

	b := &builder{rng: rand.New(rand.NewSource(*seed)), fanout: *fanout, depth: *depth}
	b.add(-1, "", 0, time.Time{}, baseEnd)

	if err := os.MkdirAll(*out, 0o755); err != nil {
		fail("%v", err)
	}
	w := &writer{
		rng:   rand.New(rand.NewSource(*seed + 1)),
		nodes: b.nodes,
		items: *items,
		xref:  *xrefs,
	}
	var totalBytes, totalLines int
	for i, n := range b.nodes {
		data := w.render(i)
		full := filepath.Join(*out, filepath.FromSlash(n.file))
		if err := os.MkdirAll(filepath.Dir(full), 0o755); err != nil {
			fail("%v", err)
		}
		if err := os.WriteFile(full, data, 0o644); err != nil {
			fail("%v", err)
		}
		totalBytes += len(data)
		totalLines += bytes.Count(data, []byte{'\n'})
	}

	fmt.Printf("files=%d bytes=%d lines=%d time=%s out=%s\n",
		len(b.nodes), totalBytes, totalLines, time.Since(start).Round(time.Millisecond), *out)
}
