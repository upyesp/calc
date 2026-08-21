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
| 1.3.1 Info & relationships | PASS | Native `form`/`input`/`button`, `ul` history, `h1` (visually hidden: the app name). Keypad tabs are an APG tablist (`role="tab"`/`tabpanel`, `aria-selected`, `aria-controls`); each button's accessible name is its token label. The macOS-only "install the epher command" button (ADR-0011) is a native `button` after the status region; its outcome reports through the existing `role="status"` live region. |
| 1.3.2 Meaningful sequence | PASS | One fixed screen (ADR-0016): answer → input → history → keypad, top to bottom; the graph pane follows in DOM order. |
| 1.4.1 Use of color | PASS | No color-only information (result is text; errors are text). |
| 1.4.3 Contrast (AA) | PASS | `--text` on `--bg` 17.0:1; result 17.0:1; input text on `--panel` 15.3:1; `--muted` history on `--bg` 6.6:1; button `#0b1512` on `--accent` 10.0:1; placeholder `#a1a1a6` on `--panel` 6.4:1. |
| 1.4.4 Resize text 200% | PASS | Fixed viewport with internal scroll regions; `overflow-wrap: anywhere` on results. |
| 1.4.10 Reflow | PASS | Below 880px the panes stack as swipeable full-width panes (scroll-snap + pane-switch buttons); no horizontal scroll at 320px; desktop column + graph side by side from 880px. |
| 1.4.11 Non-text contrast | **FIXED** | Input boundary was 1.2:1 vs the page background (invisible field). Border is now `--border: #6a6b70` — 3.5:1 vs `--bg`, 3.1:1 vs `--panel`. Focus indicators: see 2.4.7. Graph curve `--accent` on `--bg` is 9.9:1; the axes blend to ~5.4:1 at opacity 0.5 — both ≥ 3:1. Curve palette: accent 9.9:1, `#4da3ff` 7.0:1, `#ffb340` 10.3:1, `#c39dff` 8.4:1. |
| 1.4.12 Text spacing | PASS | No fixed line-heights that would clip. |
| 1.4.13 Content on hover | N/A | No hover-triggered content. |

### Operable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 2.1.1 Keyboard | PASS | Native input + keypad buttons; Enter activates from the field; every keypad button is reachable and activatable; scrollable regions (history box, graph pane) carry `tabindex="0"` so their content is keyboard-scrollable; the TUI keypad opens with Tab, moves with arrows, inserts with Enter (ADR-0016). |
| 2.1.2 No keyboard trap | PASS | Keypad buttons are ordinary tab stops; the TUI keypad closes with Tab/Esc. |
| 2.4.1 Bypass blocks | N/A | Single screen; nothing to skip. |
| 2.4.2 Page titled | PASS | `<title>epher</title>`. |
| 2.4.3 Focus order | PASS | Document order: answer, input, history, keypad, graph pane; the mobile pane switch buttons precede the panes. |
| 2.4.4 Link purpose | N/A | No links. |
| 2.4.6 Headings & labels | **FIXED** | Input has `aria-label`; button's bare `=` name replaced with `aria-label="Evaluate"`. |
| 2.4.7 Focus visible | **FIXED** | Was: no styles (browser-default ring on a dark theme, inconsistent). Now: `:focus-visible` accent outline (9.9:1 vs `--bg`, 8.9:1 vs `--panel`); the accent button gets an inset dark-teal ring (10.0:1 on the accent surface — an outer ring would not contrast). |
| 2.4.11 Focus not obscured | PASS | No sticky/overlay content (AA; 2.4.12 AAA not targeted). |
| 2.5.8 Target size (AA) | PASS | Keypad buttons ≥44×44px in a 5-column grid; tab buttons ≥44px wide; the install-cli button is ≥48px tall (padding `0.5rem 1rem` on `0.95rem` text — ~48px). |

### Understandable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 3.1.1 Language of page | PASS* | `lang`/`dir` track the resolved locale (detection via `navigator.languages`, `dir="rtl"` for Arabic); the guide pages set both per locale at build time. *The landing page's static `html lang="en"` is updated at runtime from the stored preference; its initial paint is English until app.js runs. |
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

## Marketing site (epher.org: landing, About, Privacy, guide pages)

The 2026 redesign (teal accent replacing amber) keeps the same bar.
Contrast values are recorded in `site/styles.css` (single source of
truth); the design rationale and research live in
`docs/research/modern-ui-accessibility.md`.

| Criterion | Status | Evidence / notes |
|---|---|---|
| 1.4.3 Contrast (AA) | PASS | light: text 17.0:1, muted 6.4:1, links 5.5:1, primary button 5.5:1. dark: text 16.9:1, muted 6.6:1, links 12.5:1, primary button 11.4:1. |
| 1.4.11 Non-text contrast | PASS | Icons/rings ≥ 3.7:1 (light) / 11.0:1 (dark); interactive control borders and card edges 6.8:1 (light) / 3.5:1 (dark, 3.1:1 vs panel). |
| 1.4.1 Use of color | PASS | No color-only indicators; links are underlined. |
| 2.1.1 Keyboard | PASS | The disclosure (hamburger) nav is a native `button`; links are real anchors; no pointer-only interaction. |
| 2.4.1 Bypass blocks | PASS | Skip link on every page; single `main` landmark. |
| 2.4.7 Focus visible | PASS | 3px `--ring` outline, offset 2px; accent-filled controls use an inset ring in their text color. |
| 2.4.11 Focus not obscured | PASS | Sticky header is translucent; focus rings on header controls remain visible. |
| 2.5.8 Target size (AA) | PASS | All interactive targets ≥ 44×44 CSS px (nav links, buttons, icon buttons, select — 2.5.5 best practice, not just the 24px AA floor). |
| 2.3.3 Animation from interactions | PASS | Only 150 ms color transitions; all motion disabled under `prefers-reduced-motion`. Smooth scrolling is also disabled under the preference. |
| 4.1.2 Name, role, value | PASS | Menu button: native `button` + visually-hidden label + `aria-expanded` + `aria-controls`; nav is a labelled `nav` landmark. |
| 3.1.1/3.1.2 Language | PASS | `lang`/`dir` track the active locale (RTL for Arabic) on all pages; guide pages bake both in at build time. |
| 2.4.5/ARIA APG nav | PASS | Disclosure pattern per the WAI-ARIA Authoring Practices: `hidden` removes collapsed links from the tab order, Escape closes and restores focus, click-outside closes, link activation closes. A `<noscript>` style shows the links stacked when JavaScript is off. Desktop never hides the nav behind the button. |

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
