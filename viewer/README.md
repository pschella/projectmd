# projectmd viewer

View-only Gantt viewer: plain HTML/CSS/ES modules, no build step, no dependencies.
The parser runs in the browser as `pmd.wasm` (contract: `docs/VIEWER-CONTRACT.md`).

Run (from the repo root):

    python3 -m http.server 8642
    open "http://localhost:8642/viewer/?root=/examples/lunar/hq/lunar-programme.md&today=1967-11-15"

Files: `index.html`, `style.css`, `app.js` (UI, data model, rendering), `pmd.js` (WASM wrapper), `pmd.wasm` (built by `parser/build-wasm.sh`).

Use:
- Click a row (any kind, loaded or not) for a details panel: dates, forecast, conflict reason, `after:` links (click to jump to the row), tags, owner, other keys, id, file location and, for sub-projects, the interface (commitment, milestones) read live from its file.
- Keyboard: Up/Down move, Right/Left expand/collapse (or step into/out of a group), Enter/Space toggle, Home/End, Esc closes the panel.
- The chevron expands a sub-project (fetched on demand) or a heading. Ctrl/Cmd-click, double-click or the hover "Open" button makes a sub-project the top level (breadcrumbs and browser back work).
- Zoom: Weeks / Months / Quarters / Years, +/-, Ctrl+wheel, Fit (whole project), Today. Drag the splitter to resize the table.
- Hover highlights the row in table and chart; bars show a tooltip. Solid bars are declared, dashed ones are forecast only (placed by dependencies or duration), a thin line under a bar is the declared span when it differs from the forecast. Red marks a conflict (reason in tooltip and panel). The legend is at the bottom. Light and dark follow the system.

Direct links: the viewer reads the opened file's direct links live (one hop) to draw sub-projects and deliverables; a link it cannot read is shown as unknown. Children are expanded on demand.
