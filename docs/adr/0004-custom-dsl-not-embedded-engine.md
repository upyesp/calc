# Build a custom DSL instead of embedding a language engine

Calc's scripting language is a custom domain-specific language with its own
lexer, parser, and evaluator — not an embedded engine (Rhai, Lua, or JS).

We chose to own the language so that LaTeX input, the layered numerics
(ADR-0005), graphing (ADR-0006), and the math-oriented grammar all share one AST.
Embedding would trade the problem we most want to control — the math/value model
— for a dependency whose number types and syntax fight those same goals. v1
targets "L2": expressions, variables, named functions with recursion, control
flow (`if`/`while`/`for`), and lists; closures, modules, and pattern matching are
deferred.
