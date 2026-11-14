# projectmd viewer

View-only Gantt viewer: plain HTML/CSS/ES modules, no build step, no dependencies.
The parser runs in the browser as `pmd.wasm` (contract: `docs/VIEWER-CONTRACT.md`).

Run (from the repo root):

    python3 -m http.server 8642
    open "http://localhost:8642/viewer/?root=/examples/car/aurora.md"

Files: `index.html`, `style.css`, `app.js` (UI, data model, rendering), `pmd.js` (WASM wrapper), `pmd.wasm` (built by `parser/build-wasm.sh`).

Use: click a row for details; arrow expands a sub-project (lazy fetch) or heading; Ctrl/Cmd-click,
double-click or the hover button on a sub-project makes it the top level (breadcrumbs / browser back work);
Ctrl+wheel or the toolbar zooms; drag the splitter to resize the grid. The top level renders from the root
file plus `pmd.lock` (searched in the root's folder and parents); children are only fetched on expand.
