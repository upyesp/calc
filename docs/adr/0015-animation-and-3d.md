# ADR-0015: Animation and 3D graphing

- Status: accepted
- Date: 2026-08-17
- Supersedes the "not reachable short-term" judgment on 3D in ADR-0014; the
  projection design makes it reachable for both renderers.

## Context

The competitive survey (docs/research/graphing-features.md) and its focused
follow-up (docs/research/animation-and-3d.md, 34 first-party citations) show
that every calculator that animates does it one way: **parameter-driven
playback** — a numeric parameter steps through a bounded interval and
everything referencing it redraws (Desmos slider Play, GeoGebra Animation,
Wolfram `{u, umin, umax, du}`, TI-Nspire slider Animate). None of them animate
a hidden clock. For 3D, the universal entry point is the **surface mesh**
(z = f(x, y) sampled on a grid), with orbit via drag plus arrow keys and an
axis box; nobody ships 3D points-of-interest or trace.

epher already has the parameters (user constants + sliders, ADR-0012/0014),
the samplers, and two renderers that draw lines (SVG, ASCII). The question is
how to add playback and a third dimension without a renderer rewrite.

## Decision

### Animation: a transport layer over constants

- No new language. A constant is animated; the guide's time-based example is
  `const t = 0` + `graph sin(x - t)` and playing t's slider — the Desmos model.
- Web/desktop: every slider row gets a play/pause button. Playback steps the
  constant by the slider's step (0.1) within the slider's shown bounds and
  **loops** (wraps), one step per 120 ms — a v±2 cycle takes ≈5 s, the vendor
  norm. Dragging the animated slider stops playback; the play button is also
  the pause button (WCAG 2.2.2: user-triggered, one control).
- **Reduced motion (WCAG 2.3.3):** `prefers-reduced-motion` degrades the play
  button to a **step button** — each press advances the parameter once, no
  looping playback. The research note found no vendor honoring the
  preference; epher closes the gap instead of ignoring it.
- TUI: the space bar (empty input) starts/stops playback; the loop ticks on a
  50 ms event poll and re-samples everything referencing the constant. The
  animated constant is the first one referenced by any plot.
- The animation loop in the web app communicates through a live
  `Rc<RefCell<PlaySpec>>` cell (plus a cell holding the freshest
  resample callback), not through Yew handles — handles captured at spawn
  read stale snapshots (the same lesson as the trace fix in ADR-0014).
- During playback the 3D plot keeps the **viewBox frozen** at play start, so
  the plot — and its pause button — do not jump around every tick.
- Explicit deferrals: speed control, direction modes (forward/backward/
  oscillate), loop modes (repeat/once), and orientation presets.

### 3D: project in core, draw with the existing renderers

- Grammar: `graph3d <expr(x, y)> [from a to b]` — z = f(x, y) over a square
  domain (default −5..5); several `graph3d` lines overlay; `graph3d clear`
  empties. `from a to b` is the existing 2D domain syntax (ADR-0014).
- Sampling: `epher-core::graph::sample_surface` — a grid (40 TUI, 30 web) of
  `z` values; undefined cells are NaN and split the mesh (discontinuity
  gaps, the survey's universal behavior). A surface with no finite values is
  an error.
- Projection: `View3D` (yaw, pitch, clamped to −1.4..1.4, perspective camera
  at 12) maps world points to screen coordinates in core
  (`project_point`); `project_surface` emits painter-sorted segments and
  `project_mesh` emits whole grid lines as polylines with mean depths. The
  renderers never see 3D — they draw 2D lines, exactly as they do for
  curves. The ground square + three axes come from `surface_frame`.
- Web/desktop: the mesh renders as SVG polylines, painter-sorted far-to-near,
  with per-line depth shading (**opacity**, not color — WCAG 1.4.1), the
  frame on top. Orbit: pointer drag (pointer capture) and arrow keys when
  the plot has focus (WCAG 2.1.1). The SVG content is an innerHTML string,
  not diffed nodes — a thousand-line mesh re-renders cheaply while orbiting.
- TUI: `render_ascii3d` plots the projected segments with Bresenham lines,
  depth-shaded glyphs (`*` near, `+` middle, `.` far), the frame in `o`;
  arrow keys (empty input) orbit; the legend names each surface (`z = …`)
  as the text alternative.
- Explicit deferrals: parametric surfaces and space curves, implicit
  surfaces, solids/color maps/lighting, a resolution knob (`points n`),
  3D points of interest, 3D trace, and named orientation presets.

## Consequences

- One numeric engine and one projection feed both renderers; no WebGL and no
  renderer rewrite. The 3D plot is an SVG image with a text alternative
  (aria-label), keyboard orbit, and no color-only cues, so the WCAG 2.2 AA
  posture of the 2D plot carries over.
- Animation reuses the exact resample path a slider drag uses, so animated
  constants persist via `save name` exactly like dragged ones (ADR-0012).
- Playback never starts on its own anywhere; reduced-motion users get a step
  button instead of motion.
- The TUI event loop polls with a timeout only while playing; idle behavior
  is unchanged.
