# projectmd viewer

View-only Gantt viewer: plain HTML/CSS/ES modules, no build step, no dependencies.
The parser runs in the browser as `pmd.wasm` (contract: `docs/VIEWER-CONTRACT.md`).

Run (from the repo root):

    python3 -m http.server 8642
    open "http://localhost:8642/viewer/?root=/examples/lunar/hq/lunar-programme.md&today=1967-11-15"

Files: `index.html`, `style.css`, `app.js` (UI, data model, rendering), `pmd.js` (WASM wrapper), `pmd.wasm` (built by `parser/build-wasm.sh`).

Use:
- Click a row (any kind, loaded or not) for a details panel: dates, forecast, conflict reason, `after:` links (click to jump to the row), tags, owner, other keys, id, file location and, for sub-projects, the interface (commitment, milestones) read live from its file.
- Keyboard: Up/Down move, Right/Left expand/collapse (or step into/out of a group), Enter/Space toggle, Home/End, i shows details, / searches, Esc closes the panel (or clears the search).
- The chevron expands a sub-project (fetched on demand) or a heading. Ctrl/Cmd-click, double-click or the hover "Open" button makes a sub-project the top level (breadcrumbs and browser back work).
- Zoom: Weeks / Months / Quarters / Years, +/-, Ctrl+wheel, Fit (whole project), Today. Presets: "4 weeks" (today -1 week to +4 weeks) and "Quarter" (today -2 weeks to +3 months). Drag the splitter to resize the table.
- Levels: "Level 1 / 2 / 3 / All" collapse or expand the loaded tree to that depth. A level fetches a sub-project only when it reaches into it; "All" expands only what is loaded.
- Attention bar: counts over the loaded tree, following the plan date. Click a chip to filter the table to those rows and their parents (click again to clear). Chips: Conflicts; Disputed (parent decides); Overdue (child delivers); Decisions owed (open leaf tasks with an `owner:` and no date, range or estimate, whose forecast ends today); Due in 14 days (open items whose declared or forecast end is within 14 days of the plan date); Late deliveries (done later than declared).
- Filters (row 2): search title, owner or `^id` (`/` focuses, Escape clears); Owner; Hide done; Milestones only (milestones, deliverables and sub-projects, flat under the project: the view to report upward). Filters combine; while any is active, loaded items are shown expanded and the Level buttons do not apply.
- Columns: Slack / late (`+5d`, red `-10d`, or `done 5d late`) and Next move (`parent: fix`, `parent: decide`, `child: deliver`, `child: answer`, or the owner of an open decision). Show or hide them from the Columns menu (remembered in this browser). Container rows show done/total leaf items and a green fill in their chart bar.
- Share and print: Copy link copies the current view (root, plan date, filters, selection). Print gives the table and chart on landscape pages with the date and attention counts as text.
- The details panel sits beside the chart at widths of 1100px and more; below that it covers the chart. Its description is shown near the top.
- Hover highlights the row in table and chart; bars show a tooltip. Solid bars are declared, dashed ones are forecast only (placed by dependencies or duration), a thin line under a bar is the declared span when it differs from the forecast. Red marks a conflict (reason in tooltip and panel). The legend is at the bottom. Light and dark follow the system.

- Views: "Gantt | Board" in the toolbar (`g` Gantt, `b` Board, not while typing; URL `view=board`). The Board shows Waiting / Ready / Doing / Done for the root file and every loaded sub-project file, one section each, from `pmd_board`. Cards show title, owner and the detail (waits on / since / done). Owner and search apply to cards; a card opens its item in the Gantt details panel. The footer gives work in progress, throughput, cycle time and flow forecast. The board is read-only: to move a card, edit the file.
- Link rows (sub-project and deliverable with `by:`) show slack as ask minus promise: `-10d` in red when the promise is later, `+6d` when earlier (the JSON slack wins when present). A deliverable's name shows the milestone first, the project name after it in muted text.
- Attention chip "Tight": open items with a declared end (or `by:`) and 0 to 2 days of slack.

Direct links: the viewer reads the opened file's direct links live (one hop) to draw sub-projects and deliverables; a link it cannot read is shown as unknown. Children are expanded on demand.
