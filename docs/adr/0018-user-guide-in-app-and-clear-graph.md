# ADR-0018: The user guide inside the app, and one button to clear the graph pane

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Two requests landed together:

1. **A visible way to clear the graph pane.** The `graph clear` and
   `graph3d clear` commands already existed in the web app, the TUI, and
   the guide — but a discoverable UI affordance was missing.
2. **The user guide inside every version of the app.** The website has
   a full guide (eight languages, generated from `site/guide/*.md`);
   the apps had nothing. Requirements: same content as the website,
   shown in the app's current interface language, with clickable
   examples that load into the entry field.

## Decision

### One guide, one source, three renderers

The single source of truth stays `site/guide/<locale>.md`. Three
consumers render it:

- `scripts/build-guide.mjs` — the website's static guide pages
  (existing, uses `marked` + the epher syntax highlighter).
- A new crate, `epher-guide`, embeds the same markdown at build time
  (build.rs copies the eight files into OUT_DIR) and offers two small
  renderers:
  - `render_html` for the web/desktop overlay — headings, paragraphs,
    flat lists, pipe tables, blockquotes, and inline `code`/bold/italic;
    every `` ```epher ``/`` ```sh `` fence becomes a clickable example
    button carrying the code in `data-code`.
  - `render_text` for the TUI pager — the same parser producing styled
    lines (headings bold, code and quotes muted).

The guide's markdown feature set is bounded by what the guide actually
uses (audited: no links, images, nested lists, or raw HTML), so both
renderers stay small, and every byte of user text is escaped before it
becomes HTML. There is no network fetch and no second copy: a guide
edit rebuilds the website pages and the apps from the same files.
Languages without a translation fall back to English, exactly like the
website.

### Help menu everywhere

- Web/desktop: a fourth top-level menu **Help → User guide**; the
  mobile hamburger panel gains a **Help** group with the same item.
- TUI: the menu bar grows to File, Edit, Graph, Settings, Help; Help
  opens the guide as a modal pager (Up/Down/PgUp/PgDn/Home/End scroll,
  Esc/q close).

The web overlay is an `role="dialog"` panel: the close button takes
focus on open (imperatively — the HTML `autofocus` attribute only fires
on first insert, which cost us a bug and a regression test), Escape
closes, the scrollable body carries `tabindex="0"`, and clicking an
example inserts its code into the entry field, closes the guide, and
returns to the calculator pane.

### Clear graph

- A **Clear graph** button appears at the top of the graph pane in the
  web/desktop app whenever anything is plotted; one click empties
  curves, points of interest, 3D surfaces, and any running animation,
  and confirms with a status message.
- The TUI gains a **Graph** menu whose single item does the same.
- The commands (`graph clear`, `graph3d clear`) already existed and
  stay; the guide now documents both spellings.

## Consequences

- `crates/guide` joins the workspace; the web and TUI crates depend on
  it. The web bundle grows by the embedded guide (~90 KB markdown
  before compression) — accepted for offline parity with the site.
- The guide's eight languages all render; the keys tables and TUI
  sections gained rows for F10, the menus, and the in-app guide, with
  the example fences kept byte-identical across locales.
- The desktop app inherits the web overlay unchanged (it wraps the web
  artifact).
