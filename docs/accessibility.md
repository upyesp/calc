# Accessibility (WCAG 2.2 AA) — audit and fixes

Living audit of epher against WCAG 2.2. Target: **Level AA** for the web/PWA
(and the Tauri shell, which wraps the same page). The TUI and CLI live in
terminals, so the applicable criteria are the keyboard-operability,
understandability, and theme-respect ones; the terminal emulator owns the rest
(contrast, font zoom, screen-reader output).

Contrast ratios below are computed (WCAG relative-luminance formula); text
checks use 4.5:1, non-text (UI component boundaries, focus indicators) 3:1.

## Web / PWA (and Tauri shell)

### Perceivable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 1.1.1 Non-text content | PASS | Icon is a favicon (no alt needed). Button has text content plus `aria-label="Evaluate"` (2.4.6). The graph SVG is `role="img"` with a `title` and an `aria-label` naming the plotted expression, and a visible caption (`y = <source>`) sits above it — the TUI pattern, ported (ADR-0009). |
| 1.3.1 Info & relationships | PASS | Native `form`/`input`/`button`, `ul` history, single `h1`. |
| 1.3.2 Meaningful sequence | PASS | Single-column flex. |
| 1.4.1 Use of color | PASS | No color-only information (result is text; errors are text). |
| 1.4.3 Contrast (AA) | PASS | `--text` on `--bg` 17.0:1; result 17.0:1; input text on `--panel` 13.9:1; `--muted` history on `--bg` 5.2:1; button `#000` on `--accent` 10.2:1; placeholder `#a1a1a6` on `--panel` 5.4:1. |
| 1.4.4 Resize text 200% | PASS | Flex column, no fixed heights, `overflow-wrap: anywhere` on results. |
| 1.4.10 Reflow | PASS | Single column, no horizontal scroll at 320px. |
| 1.4.11 Non-text contrast | **FIXED** | Input boundary was 1.2:1 vs the page background (invisible field). Border is now `--border: #76767a` — 3.8:1 vs `--bg`, 3.1:1 vs `--panel`. Focus indicators: see 2.4.7. Graph curve `--accent` on `--bg` is 8.3:1; the axes blend to ~3.9:1 at opacity 0.4 — both ≥ 3:1. |
| 1.4.12 Text spacing | PASS | No fixed line-heights that would clip. |
| 1.4.13 Content on hover | N/A | No hover-triggered content. |

### Operable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 2.1.1 Keyboard | PASS | Native input + submit button; Enter activates from the field, Enter/Space on the button. |
| 2.1.2 No keyboard trap | PASS | Two elements, nothing traps. |
| 2.4.1 Bypass blocks | N/A | Single view; nothing to skip. |
| 2.4.2 Page titled | PASS | `<title>epher</title>`. |
| 2.4.3 Focus order | PASS | Input → button (document order). |
| 2.4.4 Link purpose | N/A | No links. |
| 2.4.6 Headings & labels | **FIXED** | Input has `aria-label`; button's bare `=` name replaced with `aria-label="Evaluate"`. |
| 2.4.7 Focus visible | **FIXED** | Was: no styles (browser-default ring on a dark theme, inconsistent). Now: `:focus-visible` accent outline (8.3:1 vs `--bg`, 6.8:1 vs `--panel`); the accent button gets an inset black ring (10.2:1 on the accent surface — an outer ring would not contrast). |
| 2.4.11 Focus not obscured | PASS | No sticky/overlay content (AA; 2.4.12 AAA not targeted). |
| 2.5.8 Target size (AA) | PASS | Button 48×48px (≥24px minimum); input height ~48px. |

### Understandable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 3.1.1 Language of page | PASS* | `lang="en"` matches the (currently English-only) UI. *Must track the resolved locale when `navigator.languages` detection lands, and set `dir="rtl"` for Arabic — noted in `index.html` and ADR-0008. |
| 3.2.1/3.2.2 On focus/input | PASS | Focus lands in the field on load (intentional: it is the whole app); submit only updates the result region. |
| 3.3.1 Error identification | **FIXED** | Errors already appear as text (announced); now also `aria-invalid="true"` + `aria-describedby="epher-result"` on the input while an error is showing. |
| 3.3.2 Labels | PASS | `aria-label` on the input; placeholder is a hint only. |
| 3.3.3 Error suggestion | PASS | Core error strings are descriptive ("division by zero", "unknown name …"). |
| 3.3.7 Redundant entry | N/A | Single step; history shows prior entries. |

### Robust

| Criterion | Status | Evidence / notes |
|---|---|---|
| 4.1.2 Name/role/value | PASS | Native elements only, no ARIA roles on divs. |
| 4.1.3 Status messages | PASS | Result is `role="status"` + `aria-live="polite"` — submit results and errors are announced without stealing focus. |

## TUI (terminal)

The terminal emulator provides font size, zoom, contrast themes, and
screen-reader output; the app must stay usable through them.

| Item | Status | Evidence / notes |
|---|---|---|
| Keyboard operability | PASS | All actions are keys (Enter evaluate, Esc clear, Ctrl+C / q quit); hints footer now shows them (`tui-hints`, localized). |
| Focus visible | **FIXED** | The terminal cursor now sits at the end of the input text every frame (was: wherever the shell left it). Width is unicode-aware. |
| Theme respect | PASS | No forced background colors; text colors are palette-based (`Color::Green` result, `DarkGray` hints), so user themes (incl. high-contrast) apply. |
| Screen-reader output | **FIXED** | The graph panel now carries a text caption (`y = <source>`) above the ASCII plot, so terminal screen readers announce what the plot shows. |
| Zoom/reflow | PASS | Layout is proportional (`Min(0)` history, fixed 20-row graph); 200% terminal zoom reflows. |

## CLI

Plain text in, plain text out — no ANSI colors, no interaction beyond stdin.
Theme-safe by construction; nothing to fix.

## Known gaps (tracked elsewhere)

- Web UI strings are hardcoded English (i18n wiring into the Yew app is
  pending the browser test harness); `lang` and labels must move to the
  Localizer with locale detection (ADR-0008).
- No automated a11y checks yet — no headless-browser harness in this
  environment. When one exists, add axe-core to the web build pipeline and
  re-run this audit.

## How to re-verify

1. **Contrast**: recompute ratios (formula above) after any color change; the
   3:1 boundary rule applies to borders and focus indicators.
2. **Keyboard**: Tab input → button; Enter evaluates; button activates via
   Enter and Space; no outline is removed without a `:focus-visible`
   replacement.
3. **Screen reader**: page reads h1, labeled field, result announcements;
   `aria-invalid` flips on errors.
4. **TUI**: cursor is visible in the input field; hints line shown; `graph`
   shows the caption; all keys work with a screen reader's pass-through.
5. **Automated** (future): `npx @axe-core/cli` against a served `dist/`.
