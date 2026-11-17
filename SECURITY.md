# Security

## Model

projectmd files are untrusted input. `pmd` parses them as text: it executes nothing from a file, and it
reads only the files a link names (local paths, and URLs when you ask for them). A malicious plan can at
worst make a tool slow or its output misleading, and such inputs are bugs worth reporting.

The viewer is a static page. It fetches the URLs you give it (`?root=`), renders all file text escaped
(never as HTML), and ships a Content-Security-Policy that limits where it may load scripts and
connect. Do not open a viewer link from a source you do not trust: it will fetch what the link names.

## Reporting a vulnerability

Please report privately with a GitHub security advisory on this repository (Security tab, "Report a
vulnerability"). Do not open a public issue for a vulnerability. Include a minimal plan file or URL that
reproduces it. We aim to acknowledge reports within a week.

## Supported versions

The latest release (format 0.9, beta). Earlier drafts receive no fixes.
