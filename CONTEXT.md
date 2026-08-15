# epher

epher is a programmable, scriptable calculator: users evaluate Expressions,
define reusable Functions, and write multi-line Scripts. It can graph
Expressions and accept LaTeX as an input form. (Some terms below are still
being sharpened during design.)

## Language

### Core domain

**Expression**:
A piece of mathematics that evaluates to a Value.
_Avoid_: formula, equation, calculation, term

**Value**:
The result of evaluating an Expression.
_Avoid_: result, answer, output, number

**Function**:
A named, parameterized, reusable computation that returns a Value.
_Avoid_: macro, routine, procedure

**Script**:
A sequence of statements (assignments, Function definitions, control flow)
executed in order.
_Avoid_: program, macro, routine

**Graph**:
A visual representation of one or more Expressions (for example, the curve
y = f(x)).
_Avoid_: plot, chart, figure

### Persistence

**Store**:
The persisted collection of a user's data — Functions, Scripts, history, and
settings.
_Avoid_: database, save file, cache

**Native Store**:
The Store instance reachable by frontends that have host filesystem access
(the desktop app, CLI, and TUI — three modes of the single `epher` binary,
ADR-0011). Shared across those frontends on a single device.
_Avoid_: local store, disk store

**Bridge**:
The web frontend's storage seam: `Tauri` (the Native Store over IPC inside
the desktop shell) or `None` (the session-only PWA until the Web Store
lands).
_Avoid_: sync, backend, connector

**Web Store**:
The Store instance inside the browser/PWA sandbox, physically separate from
the Native Store but sharing the same logical schema.
_Avoid_: browser storage, cache, local storage
