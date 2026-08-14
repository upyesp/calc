# Calc user guide

Welcome! Calc is a programmable, scriptable calculator. You can use it for a
quick calculation, or build up your own functions and small programs — and
everything is available in six languages.

This guide is for complete beginners. It starts with the simplest possible
calculation and builds up to the full power of the language. Every example
shows what you type and what Calc answers.

There are four ways to use Calc — pick whichever suits you:

| Version | What it is | Best when |
|---|---|---|
| **Web app** (PWA) | Runs in your browser, installable, works offline | You want the fastest start; no installation |
| **Desktop app** | A normal desktop program with its own window | You want a regular application |
| **Command line** (CLI) | Text commands in a terminal; also an interactive session | You live in a terminal and like scripts |
| **Terminal UI** (TUI) | A full-screen program inside the terminal | You want a terminal app with graphs and history on screen |

All four versions understand exactly the same language. Learn it once, use it
anywhere.

## 1. The Calc language

This chapter teaches the language shared by every version of Calc. In the web
app or desktop app, type an expression and press **Enter** (or click the
**=** button). In the CLI, type it after the `calc>` prompt. In the TUI, just
type and press **Enter**. In the CLI you can also write
`calc "expression"` to evaluate one expression directly.

### 1.1 Your first calculation

Type this:

```text
2 + 3 * 4
```

Calc answers:

```text
14
```

Multiplication is done before addition, exactly like in mathematics. That
rule is called *operator precedence*.

### 1.2 Order of operations

The full precedence order, from strongest to weakest:

1. `^` power
2. `*` and `/` multiplication and division
3. `+` and `-` addition and subtraction

Use parentheses to change the order:

```text
(2 + 3) * 4
```

```text
20
```

The `^` operator computes powers, and it works right-to-left:

```text
2 ^ 10
```

```text
1024
```

```text
2 ^ 3 ^ 2
```

```text
512
```

(`2 ^ 3 ^ 2` means `2 ^ (3 ^ 2)`, which is `2 ^ 9` = 512.)

Powers can be fractional — `2 ^ 0.5` is the square root of 2:

```text
2 ^ 0.5
```

```text
1.4142135623730951
```

Subtraction and division work left-to-right:

```text
10 - 3 - 2
```

```text
5
```

### 1.3 The special numbers pi and e

The two famous constants are built in:

```text
pi
```

```text
3.141592653589793
```

```text
2 * pi
```

```text
6.283185307179586
```

```text
e
```

```text
2.718281828459045
```

### 1.4 Comparing and logic

You can compare numbers. The result is either `true` or `false`:

| Comparison | Meaning |
|---|---|
| `a > b` | a is greater than b |
| `a < b` | a is less than b |
| `a >= b` | a is greater than or equal to b |
| `a <= b` | a is less than or equal to b |
| `a == b` | a equals b (note the double `=`) |
| `a != b` | a does not equal b |

```text
3 > 2
```

```text
true
```

```text
1 != 2
```

```text
true
```

Combine comparisons with `and`, `or` and `not`:

```text
3 > 2 and 2 < 3
```

```text
true
```

```text
not 3 > 2
```

```text
false
```

### 1.5 Variables

Give a name to a value with a single `=`:

```text
x = 5
```

```text
5
```

Calc repeats the value back to you. From now on, `x` can be used anywhere:

```text
x ^ 2
```

```text
25
```

You can change a variable whenever you like — it keeps its value until you
change it:

```text
x = x + 1
```

```text
6
```

> Names can contain letters and underscores, like `radius` or `my_total`.
> They cannot contain spaces or start with a number.

### 1.6 Decisions with if

`if` chooses between two values:

```text
if 3 > 2 then 10 else 20
```

```text
10
```

The shape is always `if condition then value_if_true else value_if_false`.
The `else` part is required.

A more useful example with a variable:

```text
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> Calc does not have text values — both branches of an `if` must be numbers
> (or the results of comparisons).

### 1.7 Loops with while

`while` repeats a statement as long as a condition holds:

```text
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Read that script as: *start x at 0; while x is less than 5, add 1 to x; then
show x.* The result is 5 because the loop ran five times.

> **Safety net:** Calc stops any loop after 100,000 steps and shows
> `error: step limit exceeded`. That protects you from loops that would
> never end. If you see it, your condition probably never became false.

### 1.8 Your own functions with def

A function is a calculation with a name and parameters:

```text
def f(x) = x ^ 2
```

Then use it:

```text
f(7)
```

```text
49
```

Functions can take several parameters:

```text
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

You can also define a function with no parameters:

```text
def answer() = 42
answer()
```

```text
42
```

### 1.9 Recursion: a function that calls itself

The most famous example — the Fibonacci numbers:

```text
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```text
fib(10)
```

```text
55
```

`fib(10)` is the 10th Fibonacci number. The function calls itself with
smaller arguments until it reaches `n <= 1`. This works because the
`if ... then ... else ...` form only calculates the branch it needs.

> A function's body is a single expression — one line. Combine several
> calculations with `;` in a script instead (next section).

### 1.10 Scripts: several statements at once

A *script* is several statements joined with `;`, executed one after another:

```text
x = 10; y = x + 5; x + y
```

```text
25
```

Scripts are how you build small programs: set up variables, loop, and show a
final result.

### 1.11 Exact results: frac, dec and big

Normally Calc calculates with decimal numbers like a pocket calculator.
Some numbers look better exact.

**frac(n, d)** makes an exact fraction:

```text
1 / 3
```

```text
0.3333333333333333
```

```text
frac(1, 3)
```

```text
1/3
```

Fractions stay exact through calculations:

```text
frac(1, 3) * 3
```

```text
1
```

**dec(x)** makes an exact decimal. Compare these two:

```text
0.1 + 0.2
```

```text
0.30000000000000004
```

```text
dec(0.1) + dec(0.2)
```

```text
0.3
```

The first result is the tiny rounding error every computer makes with
decimal numbers. `dec()` removes it.

**big(x)** makes an exact whole number, for values too large for a pocket
calculator:

```text
big(10 ^ 20)
```

```text
100000000000000000000
```

### 1.12 Built-in functions

Calc has a small set of built-in functions:

| Function | Meaning | Example | Result |
|---|---|---|---|
| `sqrt(x)` | square root | `sqrt(16)` | `4` |
| `min(a, b)` | the smaller of two | `min(3, 7)` | `3` |
| `frac(n, d)` | exact fraction | `frac(1, 3)` | `1/3` |
| `dec(x)` | exact decimal | `dec(0.1)` | `0.1` |
| `big(x)` | exact whole number | `big(10 ^ 20)` | `100000000000000000000` |

They combine like everything else:

```text
min(sqrt(16), 5)
```

```text
4
```

### 1.13 Reading errors

When something goes wrong, Calc tells you instead of guessing:

```text
1 / 0
```

```text
error: division by zero
```

```text
sqrt(-4)
```

```text
error: domain error: sqrt of negative number -4
```

```text
unknown_name
```

```text
error: unknown name: unknown_name
```

```text
sin(1)
```

```text
error: unknown name: sin
```

The last example is important: `sin` is **not** built in — only the
functions listed in section 1.12. The error message tells you exactly what
Calc does not know, so you can fix your expression.

### 1.14 Quick reference

| What | Syntax | Example |
|---|---|---|
| Add, subtract, multiply, divide | `+ - * /` | `7 / 2` |
| Power | `^` (right-to-left) | `2 ^ 10` |
| Parentheses | `( )` | `(2 + 3) * 4` |
| Constants | `pi`, `e` | `2 * pi` |
| Compare | `> < >= <= == !=` | `3 >= 2` |
| Logic | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Decision | `if c then a else b` | `if x > 0 then 1 else -1` |
| Loop | `while c do statement` | `while x < 5 do x = x + 1` |
| Function | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | statements joined with `;` | `x = 1; x + 1` |
| Exact fraction | `frac(n, d)` | `frac(1, 3)` |
| Exact decimal | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Exact whole number | `big(x)` | `big(10 ^ 20)` |

## 2. The web app (PWA)

### 2.1 Opening it

The web app lives at:

```text
https://upyesp.github.io/calc/pwa/
```

No installation is needed — it works in any modern browser on a computer,
phone, or tablet.

### 2.2 Your first calculation

1. Click the text field (it is already focused when the page loads).
2. Type an expression, for example `2 + 3 * 4`.
3. Press **Enter** or click the **=** button.

The result appears in large text below the field. Everything from chapter 1
works here, including variables, functions, and scripts.

### 2.3 History

Every calculation is added to the history list beneath the result, so you
can scroll back and see what you did. The history is kept while the page is
open.

### 2.4 Installing it and using it offline

The web app is a *progressive web app*: after one visit it works fully
offline, and you can install it like a normal app.

- **Chrome, Edge, or Android:** click the install icon in the address bar
  (or *Install app* in the browser menu), then confirm.
- **iPhone / iPad (Safari):** tap **Share** → **Add to Home Screen**.
- **Other browsers:** look for *Install* or *Add to Home Screen* in the menu.

Once installed, launch it from your home screen or app list — it opens
instantly, even with no internet connection.

### 2.5 What the web app does not do

The web app is intentionally simple: it evaluates expressions and keeps a
session history. The **save**, **save script**, and **language** commands
work in the desktop, command line, and terminal versions (chapters 3, 4,
and 5) — in the web app they answer with a note that saving works there.
The history is not saved between visits.

## 3. The desktop app

The desktop app is a normal window around the same web app. Everything in
chapter 2 applies; the difference is only how you install and start it.

### 3.1 Installing

Download the desktop app for your system from the Calc website:

- **Linux (Debian/Ubuntu):** the `.deb` package

```text
sudo apt install ./calc-desktop-linux-x86_64.deb
```

- **Linux (Fedora/RHEL):** the `.rpm` package

```text
sudo dnf install ./calc-desktop-linux-x86_64.rpm
```

- **Linux (any distro):** the AppImage — make it executable and run it:

```text
chmod +x calc-desktop-linux-x86_64.AppImage
./calc-desktop-linux-x86_64.AppImage
```

- **macOS:** open the `.dmg` and drag Calc into Applications. Because the
  build is not signed, the first launch needs a right-click → **Open**.
- **Windows:** run the installer. Because the build is not signed, choose
  *More info* → *Run anyway* on the first launch.

### 3.2 Using it

Launch Calc like any other application. You get a window with the same
interface as the web app: type an expression, press **Enter** or click
**=**, and read the result. The window can be resized freely.

### 3.3 Storage: one store with the CLI and TUI

The desktop app shares its storage with the command line and terminal
versions. Functions, scripts, history, and the language preference live in
one place — `~/.calc` on your computer (or `CALC_STORE_DIR`, chapter
4.5) — and everything saved in one version is available in the others:

```text
def area(w, h) = w * h
save area
```

Define `area` in the desktop app, `save` it, close the window — then open
the CLI and `area(3, 4)` just works. It works the other way too: functions
and scripts you saved in the CLI or TUI are already there when the desktop
window opens, including variables set by saved scripts. The `save`,
`save script`, and `language` commands from chapter 4 work exactly the
same here.

> The web app in the browser is the one version that does not use this
> storage — it keeps each session to itself (chapter 2.5).

## 4. The command line (CLI)

The CLI is the text version of Calc. It comes in two modes: a one-shot mode
for quick results, and an interactive session for longer work.

### 4.1 One-shot calculations

Give the expression as an argument:

```text
calc "2 + 3 * 4"
```

```text
14
```

You can do anything from chapter 1 that is a single expression:

```text
calc "if 3 > 2 then 10 else 20"
```

```text
10
```

If your expression starts with a minus sign, tell the CLI where the
expression begins with `--`:

```text
calc -- "-2 + 5"
```

```text
3
```

One-shot mode evaluates exactly one expression. Statements — variables,
functions, loops — need the interactive session.

### 4.2 The interactive session (REPL)

Start the session with no arguments:

```text
calc
```

Calc prints its prompt and waits:

```text
calc>
```

Now type anything from chapter 1, one line at a time. Variables keep their
values between lines:

```text
calc> x = 5
= 5
calc> x ^ 2
= 25
```

Each answer is shown as `= result`. To leave, type `quit` (or `exit`):

```text
calc> quit
```

Your history is remembered: the next time you start `calc`, the previous
session's lines are still there.

### 4.3 Saving functions and scripts

Define a function, then save it:

```text
calc> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
calc> save fib
saved fib
```

The `save fib` command stores the function on disk. Next time you start
`calc`, `fib` is already defined:

```text
calc> fib(10)
= 55
```

To save a whole script (the last line you typed) use `save script`:

```text
calc> x = 0; while x < 5 do x = x + 1; x
= 5
calc> save script count_to_five
saved script count_to_five
```

Saved scripts run automatically when Calc starts, so anything they define is
ready for you.

### 4.4 Changing the interface language

The interface language is chosen from the languages you set on your device.
To override it, type `language` followed by one of: `en`, `zh-CN`, `hi`,
`es`, `fr`, `ar`:

```text
calc> language fr
language set to fr
```

The choice is remembered for next time. Note: the language you *type* — the
expression language — is always the same, in any interface language.

### 4.5 Where your data lives

Functions, scripts, history, and your language choice are stored in one
folder on your computer:

```text
~/.calc
```

Delete that folder to start completely fresh. To use a different location,
set the environment variable `CALC_STORE_DIR` before starting Calc:

```text
CALC_STORE_DIR=/tmp/my-calc calc
```

## 5. The terminal UI (TUI)

The TUI is a full-screen version of the interactive session, inside your
terminal. Start it with:

```text
calc-tui
```

### 5.1 The screen

The screen is divided into panels:

- **Expression** — the input line (top).
- The current **result** right below it.
- **History** — every line you entered, with its answer.
- **Graph** — the plot from the `graph` command (bottom).
- A hint line shows the keyboard shortcuts.

### 5.2 Keys

| Key | Action |
|---|---|
| Type | add to the expression |
| **Enter** | evaluate |
| **Esc** | clear the input line |
| **Ctrl+C** | quit |
| **q** | quit (when the input is empty) |

### 5.3 Graphing

Type `graph` followed by an expression, and press **Enter**:

```text
graph x ^ 2
```

Calc samples the curve from x = −10 to x = 10 and draws it as an ASCII plot
in the Graph panel. The caption above the plot shows what is plotted:
`y = x ^ 2`.

You can graph any expression, including your own functions — first define
one, then graph it:

```text
def f(x) = x ^ 3
graph f(x)
```

Points where the expression has no value (for example division by zero)
are simply skipped, leaving a gap in the plot.

### 5.4 Saving and persistence

The TUI shares its storage with the CLI: everything saved in one is
available in the other. Functions, scripts, history, and the language
preference live in `~/.calc` (chapter 4.5), and the same `save`,
`save script`, and `language` commands work here.

## 6. Your data and privacy

- The **CLI and TUI** store functions, scripts, history, and the language
  choice locally in `~/.calc` (or `CALC_STORE_DIR`). Nothing leaves your
  computer.
- The **web app** keeps nothing on disk: history lasts only while the page
  is open. The web app can work offline because the page itself is stored by
  your browser.
- The **desktop app** stores functions, scripts, history, and the language
  choice locally in `~/.calc` (or `CALC_STORE_DIR`), the same store as the
  CLI and TUI. Nothing leaves your computer.

All four versions run the calculation entirely on your device — nothing is
sent anywhere.
