//! calc-core — the single source of truth for calc's logic.
//!
//! Compiles to both `wasm32-unknown-unknown` (web/PWA/desktop) and native targets
//! (CLI/TUI). Stays pure: no I/O, no threads, no platform calls. Numerics per
//! ADR-0005.

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_complex::Complex;
use num_rational::BigRational;
use num_traits::{FromPrimitive, Zero};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The result of evaluating an Expression — the project's single number
/// representation (ADR-0005). `Float` is the default fast path; the other
/// variants are opt-in exactness layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Float(f64),
    Rational(BigRational),
    Decimal(Decimal),
    Big(BigDecimal),
    Complex(Complex<f64>),
    Bool(bool),
}

impl Value {
    /// Wrap a plain number as the default `Float` variant.
    pub fn float(n: f64) -> Self {
        Value::Float(n)
    }
}

/// Variable bindings available while evaluating an [`Expression`].
#[derive(Debug, Clone, Default)]
pub struct Env {
    bindings: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Env {
    /// Look up a name.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Bind a name to a value.
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    /// Look up a user-defined function.
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }

    /// Define a user-defined function.
    pub fn set_function(&mut self, name: impl Into<String>, function: Function) {
        self.functions.insert(name.into(), function);
    }

    /// A child environment for a function call: the function table is visible
    /// (so recursion works); the caller's bindings are not.
    fn new_child(&self) -> Env {
        Env {
            bindings: HashMap::new(),
            functions: self.functions.clone(),
        }
    }
}

/// A parsed piece of mathematics that can be evaluated to a [`Value`] — a domain
/// noun (see `CONTEXT.md`). Public so it can be produced by multiple input
/// forms (plain text, LaTeX) and consumed by both [`eval`] and the graphing
/// Sampler; treated opaquely by tests.
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(f64),
    Var(String),
    Call(String, Vec<Expression>),
    Neg(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
    Compare(CmpOp, Box<Expression>, Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
}

/// A comparison operator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpOp {
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
}

/// One statement of a [`Script`] — the unit of the script seam (CONTEXT.md).
/// Assignment mutates the [`Env`]; plain expressions just evaluate.
#[derive(Debug, Clone)]
pub enum Statement {
    Assign(String, Expression),
    FunctionDef(String, Vec<String>, Expression),
    While(Expression, Box<Statement>),
    Expr(Expression),
}

/// A user-defined function: parameter names and a body expression.
#[derive(Debug, Clone)]
pub struct Function {
    params: Vec<String>,
    body: Expression,
}

/// Errors crossing the calc-core seams.
#[derive(Debug, thiserror::Error)]
pub enum CalcError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("type error: {0}")]
    Type(String),
    #[error("unknown name: {0}")]
    UnknownName(String),
    #[error("domain error: {0}")]
    Domain(String),
    #[error("division by zero")]
    ZeroDivision,
    #[error("step limit exceeded")]
    StepLimit,
}

/// Parse plain text into an [`Expression`] (the plain-text input seam).
///
/// Tokenizer + recursive-descent parser with precedence (additive below
/// multiplicative) and left-associative operator folding.
pub fn parse(text: &str) -> Result<Expression, CalcError> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expression()?;
    if parser.peek().is_some() {
        return Err(CalcError::Parse("unexpected trailing input".into()));
    }
    Ok(expr)
}

/// Parse LaTeX math into an [`Expression`] — the LaTeX input form (Q5). A
/// translation layer rewrites LaTeX constructs into plain calc text, then the
/// same grammar parses it: one grammar, two input forms.
pub fn parse_latex(text: &str) -> Result<Expression, CalcError> {
    parse(&translate_latex(text)?)
}

fn translate_latex(text: &str) -> Result<String, CalcError> {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let mut cmd = String::new();
            while let Some(&c2) = chars.peek() {
                if c2.is_ascii_alphabetic() {
                    cmd.push(c2);
                    chars.next();
                } else {
                    break;
                }
            }
            match cmd.as_str() {
                "frac" => {
                    let num = translate_latex(&take_braced(&mut chars)?)?;
                    let den = translate_latex(&take_braced(&mut chars)?)?;
                    out.push_str(&format!("({num})/({den})"));
                }
                "sqrt" => {
                    let inner = translate_latex(&take_braced(&mut chars)?)?;
                    out.push_str(&format!("sqrt({inner})"));
                }
                "cdot" | "times" => out.push('*'),
                "div" => out.push('/'),
                "pi" => out.push_str("pi"),
                "left" | "right" => {
                    // \( \left( ... \right) \) — keep the delimiter char
                    if let Some(&c2) = chars.peek() {
                        out.push(c2);
                        chars.next();
                    }
                }
                _ => {
                    return Err(CalcError::Parse(format!(
                        "unsupported LaTeX command: \\{cmd}"
                    )));
                }
            }
        } else if c == '{' {
            // bare grouping → parentheses
            let inner = translate_latex(&take_braced(&mut chars)?)?;
            out.push_str(&format!("({inner})"));
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// Take the contents of the next `{...}` group, tracking nested braces.
fn take_braced(chars: &mut impl Iterator<Item = char>) -> Result<String, CalcError> {
    match chars.next() {
        Some('{') => {}
        Some(other) => {
            return Err(CalcError::Parse(format!("expected '{{', found {other}")));
        }
        None => return Err(CalcError::Parse("expected '{'".into())),
    }
    let mut depth = 1;
    let mut inner = String::new();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                depth += 1;
                inner.push(c);
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(inner);
                }
                inner.push(c);
            }
            _ => inner.push(c),
        }
    }
    Err(CalcError::Parse("unbalanced braces in LaTeX".into()))
}

/// Parse a sequence of statements separated by `;` (the script seam).
pub fn parse_script(text: &str) -> Result<Vec<Statement>, CalcError> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, pos: 0 };
    let mut statements = Vec::new();
    loop {
        if parser.peek().is_none() {
            break;
        }
        let stmt = parser.parse_statement()?;
        statements.push(stmt);
        match parser.peek() {
            Some(Token::Semicolon) => {
                parser.next();
                // trailing ';' is fine
            }
            None => break,
            Some(_) => {
                return Err(CalcError::Parse("expected ';' between statements".into()));
            }
        }
    }
    Ok(statements)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Comma,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    EqualEqual,
    NotEqual,
    Equals,
    Semicolon,
    LParen,
    RParen,
}

fn tokenize(text: &str) -> Result<Vec<Token>, CalcError> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '>' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            '<' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            '=' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Equals);
                }
            }
            '!' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::NotEqual);
                } else {
                    return Err(CalcError::Parse("unexpected character: '!'".into()));
                }
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            c if c.is_ascii_digit() || c == '.' => {
                let mut num = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_ascii_digit() || c2 == '.' {
                        num.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let n: f64 = num
                    .parse()
                    .map_err(|_| CalcError::Parse(format!("invalid number: {num:?}")))?;
                tokens.push(Token::Number(n));
            }
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_alphabetic() || c2 == '_' {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            other => return Err(CalcError::Parse(format!("unexpected character: {other:?}"))),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// A statement is `while cond do stmt` (loop), `def name(params) = expr`
    /// (function definition), `name = expr` (assignment), or `expr`.
    fn parse_statement(&mut self) -> Result<Statement, CalcError> {
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "while") {
            self.next(); // consume 'while'
            let cond = self.parse_expression()?;
            self.expect_keyword("do")?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::While(cond, body));
        }
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "def") {
            self.next(); // consume 'def'
            let name = self.expect_ident("function name")?;
            self.expect_token(Token::LParen, "'('")?;
            let mut params = Vec::new();
            if !matches!(self.peek(), Some(Token::RParen)) {
                loop {
                    params.push(self.expect_ident("parameter name")?);
                    match self.next() {
                        Some(Token::Comma) => continue,
                        Some(Token::RParen) => break,
                        Some(other) => {
                            return Err(CalcError::Parse(format!(
                                "expected ',' or ')', found {other:?}"
                            )));
                        }
                        None => return Err(CalcError::Parse("unexpected end of input".into())),
                    }
                }
            } else {
                self.next(); // zero-parameter function
            }
            self.expect_token(Token::Equals, "'='")?;
            let body = self.parse_expression()?;
            return Ok(Statement::FunctionDef(name, params, body));
        }
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            if matches!(self.tokens.get(self.pos + 1), Some(Token::Equals)) {
                self.next(); // consume the identifier
                self.next(); // consume '='
                let expr = self.parse_expression()?;
                return Ok(Statement::Assign(name, expr));
            }
        }
        let expr = self.parse_expression()?;
        Ok(Statement::Expr(expr))
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, CalcError> {
        match self.next() {
            Some(Token::Ident(name)) => Ok(name),
            Some(other) => Err(CalcError::Parse(format!("expected {what}, found {other:?}"))),
            None => Err(CalcError::Parse("unexpected end of input".into())),
        }
    }

    fn expect_token(&mut self, token: Token, what: &str) -> Result<(), CalcError> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            Some(other) => Err(CalcError::Parse(format!("expected {what}, found {other:?}"))),
            None => Err(CalcError::Parse("unexpected end of input".into())),
        }
    }

    /// Top level: `if cond then a else b` or a comparison.
    fn parse_expression(&mut self) -> Result<Expression, CalcError> {
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "if") {
            self.next(); // consume 'if'
            let cond = self.parse_expression()?;
            self.expect_keyword("then")?;
            let then_expr = self.parse_expression()?;
            self.expect_keyword("else")?;
            let else_expr = self.parse_expression()?;
            Ok(Expression::If(
                Box::new(cond),
                Box::new(then_expr),
                Box::new(else_expr),
            ))
        } else {
            self.parse_or()
        }
    }

    /// Boolean `or` level.
    fn parse_or(&mut self) -> Result<Expression, CalcError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Ident(kw)) if kw == "or") {
            self.next();
            let right = self.parse_and()?;
            left = Expression::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Boolean `and` level.
    fn parse_and(&mut self) -> Result<Expression, CalcError> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Some(Token::Ident(kw)) if kw == "and") {
            self.next();
            let right = self.parse_not()?;
            left = Expression::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Boolean `not` level: binds looser than comparison (`not x > 3` is
    /// `not (x > 3)`).
    fn parse_not(&mut self) -> Result<Expression, CalcError> {
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "not") {
            self.next();
            let inner = self.parse_not()?;
            Ok(Expression::Not(Box::new(inner)))
        } else {
            self.parse_comparison()
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), CalcError> {
        match self.next() {
            Some(Token::Ident(found)) if found == kw => Ok(()),
            Some(other) => Err(CalcError::Parse(format!("expected '{kw}', found {other:?}"))),
            None => Err(CalcError::Parse("unexpected end of input".into())),
        }
    }

    /// Comparison level: `>` `<` `>=` `<=` `==` `!=`, non-chaining, with
    /// arithmetic binding tighter.
    fn parse_comparison(&mut self) -> Result<Expression, CalcError> {
        let left = self.parse_additive()?;
        let op = match self.peek() {
            Some(Token::GreaterThan) => Some(CmpOp::Gt),
            Some(Token::LessThan) => Some(CmpOp::Lt),
            Some(Token::GreaterEqual) => Some(CmpOp::Ge),
            Some(Token::LessEqual) => Some(CmpOp::Le),
            Some(Token::EqualEqual) => Some(CmpOp::Eq),
            Some(Token::NotEqual) => Some(CmpOp::Ne),
            _ => None,
        };
        if let Some(op) = op {
            self.next();
            let right = self.parse_additive()?;
            Ok(Expression::Compare(op, Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    /// Additive level: `+` and `-`, folded left-associatively.
    fn parse_additive(&mut self) -> Result<Expression, CalcError> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expression::Add(Box::new(left), Box::new(right));
                }
                Some(Token::Minus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expression::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Multiplicative level: `*` and `/`, folded left-associatively.
    fn parse_term(&mut self) -> Result<Expression, CalcError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    let right = self.parse_unary()?;
                    left = Expression::Mul(Box::new(left), Box::new(right));
                }
                Some(Token::Slash) => {
                    self.next();
                    let right = self.parse_unary()?;
                    left = Expression::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Unary level: `-` binds looser than `^` (math convention: `-2 ^ 2 = -4`).
    fn parse_unary(&mut self) -> Result<Expression, CalcError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.next();
            let inner = self.parse_unary()?;
            Ok(Expression::Neg(Box::new(inner)))
        } else {
            self.parse_pow()
        }
    }

    /// Power level: `^`, right-associative, binds tighter than `*` and `/`; the
    /// exponent may itself be a unary expression (`2 ^ -2`).
    fn parse_pow(&mut self) -> Result<Expression, CalcError> {
        let base = self.parse_factor()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.next();
            let exponent = self.parse_unary()?;
            Ok(Expression::Pow(Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    fn parse_factor(&mut self) -> Result<Expression, CalcError> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Literal(n)),
            Some(Token::Ident(name)) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next(); // consume '(' — call syntax
                    let mut args = Vec::new();
                    if matches!(self.peek(), Some(Token::RParen)) {
                        self.next(); // zero-argument call
                    } else {
                        loop {
                            let arg = self.parse_expression()?;
                            args.push(arg);
                            match self.next() {
                                Some(Token::Comma) => continue,
                                Some(Token::RParen) => break,
                                Some(other) => {
                                    return Err(CalcError::Parse(format!(
                                        "expected ',' or ')', found {other:?}"
                                    )));
                                }
                                None => {
                                    return Err(CalcError::Parse(
                                        "unexpected end of input".into(),
                                    ));
                                }
                            }
                        }
                    }
                    Ok(Expression::Call(name, args))
                } else {
                    Ok(Expression::Var(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    Some(other) => {
                        Err(CalcError::Parse(format!("expected ')', found {other:?}")))
                    }
                    None => Err(CalcError::Parse("unexpected end of input".into())),
                }
            }
            Some(other) => Err(CalcError::Parse(format!("expected a number, found {other:?}"))),
            None => Err(CalcError::Parse("unexpected end of input".into())),
        }
    }
}

/// Evaluate an [`Expression`] to a [`Value`] against an [`Env`] (the evaluation
/// seam).
pub fn eval(expr: &Expression, env: &Env) -> Result<Value, CalcError> {
    match expr {
        Expression::Literal(n) => Ok(Value::float(*n)),
        Expression::Var(name) => env
            .get(name)
            .cloned()
            .or_else(|| builtin_const(name))
            .ok_or_else(|| CalcError::UnknownName(name.clone())),
        Expression::Neg(inner) => match eval(inner, env)? {
            Value::Float(n) => Ok(Value::Float(-n)),
            other => Err(CalcError::Type(format!("cannot negate {other:?}"))),
        },
        Expression::Add(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Add),
        Expression::Sub(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Sub),
        Expression::Mul(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Mul),
        Expression::Div(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Div),
        Expression::Pow(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Pow),
        Expression::Compare(op, lhs, rhs) => {
            let l = eval(lhs, env)?;
            let r = eval(rhs, env)?;
            match (&l, &r) {
                (Value::Float(x), Value::Float(y)) => {
                    let result = match op {
                        CmpOp::Gt => x > y,
                        CmpOp::Lt => x < y,
                        CmpOp::Ge => x >= y,
                        CmpOp::Le => x <= y,
                        CmpOp::Eq => x == y,
                        CmpOp::Ne => x != y,
                    };
                    Ok(Value::Bool(result))
                }
                _ => Err(CalcError::Type(format!("cannot compare {l:?} and {r:?}"))),
            }
        }
        Expression::If(cond, then_expr, else_expr) => match eval(cond, env)? {
            Value::Bool(true) => eval(then_expr, env),
            Value::Bool(false) => eval(else_expr, env),
            other => Err(CalcError::Type(format!(
                "if condition must be a boolean, got {other:?}"
            ))),
        },
        Expression::And(lhs, rhs) => match eval(lhs, env)? {
            Value::Bool(false) => Ok(Value::Bool(false)),
            Value::Bool(true) => match eval(rhs, env)? {
                Value::Bool(b) => Ok(Value::Bool(b)),
                other => Err(CalcError::Type(format!(
                    "and expects booleans, got {other:?}"
                ))),
            },
            other => Err(CalcError::Type(format!(
                "and expects booleans, got {other:?}"
            ))),
        },
        Expression::Or(lhs, rhs) => match eval(lhs, env)? {
            Value::Bool(true) => Ok(Value::Bool(true)),
            Value::Bool(false) => match eval(rhs, env)? {
                Value::Bool(b) => Ok(Value::Bool(b)),
                other => Err(CalcError::Type(format!(
                    "or expects booleans, got {other:?}"
                ))),
            },
            other => Err(CalcError::Type(format!(
                "or expects booleans, got {other:?}"
            ))),
        },
        Expression::Not(inner) => match eval(inner, env)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            other => Err(CalcError::Type(format!(
                "not expects a boolean, got {other:?}"
            ))),
        },
        Expression::Call(name, args) => {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval(arg, env)?);
            }
            if let Some(f) = env.function(name) {
                if f.params.len() != values.len() {
                    return Err(CalcError::Type(format!(
                        "{name} expects {} arguments, got {}",
                        f.params.len(),
                        values.len()
                    )));
                }
                let mut child = Env::new_child(env);
                for (param, value) in f.params.iter().zip(values) {
                    child.set(param.clone(), value);
                }
                return eval(&f.body, &child);
            }
            call_builtin(name, values)
        }
    }
}

/// Evaluate source text as an expression with an empty environment — the CLI
/// one-shot convenience (composition of `parse` + `eval`, not a seam).
pub fn evaluate(text: &str) -> Result<Value, CalcError> {
    let env = Env::default();
    eval(&parse(text)?, &env)
}

/// Built-in constants (π, e), resolved when a name isn't in the environment.
fn builtin_const(name: &str) -> Option<Value> {
    match name {
        "pi" => Some(Value::float(std::f64::consts::PI)),
        "e" => Some(Value::float(std::f64::consts::E)),
        _ => None,
    }
}

/// Dispatch a builtin function call. User-defined functions (L2) will share
/// this seam later.
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, CalcError> {
    match name {
        "sqrt" => {
            let [x] = args.as_slice() else {
                return Err(CalcError::Type(format!(
                    "sqrt expects 1 argument, got {}",
                    args.len()
                )));
            };
            match x {
                Value::Float(n) => {
                    if *n < 0.0 {
                        Err(CalcError::Domain(format!("sqrt of negative number {n}")))
                    } else {
                        Ok(Value::Float(n.sqrt()))
                    }
                }
                other => Err(CalcError::Type(format!(
                    "sqrt expects a number, got {other:?}"
                ))),
            }
        }
        "min" => {
            let [a, b] = args.as_slice() else {
                return Err(CalcError::Type(format!(
                    "min expects 2 arguments, got {}",
                    args.len()
                )));
            };
            match (a, b) {
                (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.min(*y))),
                _ => Err(CalcError::Type(format!(
                    "min expects numbers, got {a:?} and {b:?}"
                ))),
            }
        }
        "frac" => {
            let [n, d] = args.as_slice() else {
                return Err(CalcError::Type(format!(
                    "frac expects 2 arguments, got {}",
                    args.len()
                )));
            };
            match (n, d) {
                (Value::Float(n), Value::Float(d)) => {
                    let to_int = |x: f64| -> Option<BigInt> {
                        if x.is_finite() && x.fract() == 0.0 && x.abs() <= i64::MAX as f64 {
                            Some(BigInt::from(x as i64))
                        } else {
                            None
                        }
                    };
                    let (Some(n), Some(d)) = (to_int(*n), to_int(*d)) else {
                        return Err(CalcError::Type(format!(
                            "frac expects integer arguments, got {n:?} and {d:?}"
                        )));
                    };
                    if d == BigInt::from(0) {
                        return Err(CalcError::ZeroDivision);
                    }
                    Ok(Value::Rational(BigRational::new(n, d)))
                }
                _ => Err(CalcError::Type(format!(
                    "frac expects numbers, got {n:?} and {d:?}"
                ))),
            }
        }
        "dec" => {
            let [x] = args.as_slice() else {
                return Err(CalcError::Type(format!(
                    "dec expects 1 argument, got {}",
                    args.len()
                )));
            };
            match x {
                Value::Float(n) => float_to_decimal(*n)
                    .map(Value::Decimal)
                    .ok_or_else(|| CalcError::Type(format!("cannot convert {n} to a decimal"))),
                other => Err(CalcError::Type(format!(
                    "dec expects a number, got {other:?}"
                ))),
            }
        }
        "big" => {
            let [x] = args.as_slice() else {
                return Err(CalcError::Type(format!(
                    "big expects 1 argument, got {}",
                    args.len()
                )));
            };
            match x {
                Value::Float(n) => float_to_big(*n)
                    .map(Value::Big)
                    .ok_or_else(|| CalcError::Type(format!("cannot convert {n} to big"))),
                other => Err(CalcError::Type(format!(
                    "big expects a number, got {other:?}"
                ))),
            }
        }
        _ => Err(CalcError::UnknownName(name.to_string())),
    }
}

/// Execute a script's statements in order against a mutable [`Env`], returning
/// the last statement's value (the script seam).
pub fn run(script: &[Statement], env: &mut Env) -> Result<Value, CalcError> {
    let mut steps = STEP_LIMIT;
    run_inner(script, env, &mut steps)
}

/// Maximum statement executions per `run` — protects against runaway loops.
const STEP_LIMIT: u64 = 100_000;

fn consume_step(steps: &mut u64) -> Result<(), CalcError> {
    if *steps == 0 {
        return Err(CalcError::StepLimit);
    }
    *steps -= 1;
    Ok(())
}

fn run_inner(script: &[Statement], env: &mut Env, steps: &mut u64) -> Result<Value, CalcError> {
    let mut result = None;
    for stmt in script {
        consume_step(steps)?;
        match stmt {
            Statement::Expr(expr) => result = Some(eval(expr, env)?),
            Statement::Assign(name, expr) => {
                let value = eval(expr, env)?;
                env.set(name.clone(), value.clone());
                result = Some(value);
            }
            Statement::FunctionDef(name, params, body) => {
                env.set_function(
                    name.clone(),
                    Function {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                // a definition produces no value
            }
            Statement::While(cond, body) => run_while(cond, body, env, steps)?,
        }
    }
    result.ok_or_else(|| CalcError::Parse("script produced no value".into()))
}

/// Execute one statement for its effect (used by loop bodies; loops produce no
/// value).
fn execute_stmt(stmt: &Statement, env: &mut Env, steps: &mut u64) -> Result<(), CalcError> {
    consume_step(steps)?;
    match stmt {
        Statement::Expr(expr) => {
            eval(expr, env)?;
            Ok(())
        }
        Statement::Assign(name, expr) => {
            let value = eval(expr, env)?;
            env.set(name.clone(), value);
            Ok(())
        }
        Statement::FunctionDef(name, params, body) => {
            env.set_function(
                name.clone(),
                Function {
                    params: params.clone(),
                    body: body.clone(),
                },
            );
            Ok(())
        }
        Statement::While(cond, body) => run_while(cond, body, env, steps),
    }
}

/// Drive a while loop: evaluate the condition, run the body while it's true.
fn run_while(
    cond: &Expression,
    body: &Statement,
    env: &mut Env,
    steps: &mut u64,
) -> Result<(), CalcError> {
    loop {
        match eval(cond, env)? {
            Value::Bool(true) => execute_stmt(body, env, steps)?,
            Value::Bool(false) => break,
            other => {
                return Err(CalcError::Type(format!(
                    "while condition must be a boolean, got {other:?}"
                )));
            }
        }
    }
    Ok(())
}

/// A binary arithmetic operator, dispatched per number layer (ADR-0005).
#[derive(Debug, Clone, Copy, PartialEq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// Apply a binary op to two [`Value`]s, promoting to a common number layer
/// (Float → Rational → Decimal → Big) when operands differ (ADR-0005).
fn binop(lhs: Value, rhs: Value, op: BinOp) -> Result<Value, CalcError> {
    match (&lhs, &rhs) {
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_binop(op, *a, *b)?)),
        (Value::Rational(a), Value::Rational(b)) => {
            Ok(Value::Rational(rational_binop(op, a.clone(), b.clone())?))
        }
        (Value::Float(a), Value::Rational(b)) => {
            let a = BigRational::from_float(*a)
                .ok_or_else(|| CalcError::Type(format!("cannot promote {a} to a rational")))?;
            Ok(Value::Rational(rational_binop(op, a, b.clone())?))
        }
        (Value::Rational(a), Value::Float(b)) => {
            let b = BigRational::from_float(*b)
                .ok_or_else(|| CalcError::Type(format!("cannot promote {b} to a rational")))?;
            Ok(Value::Rational(rational_binop(op, a.clone(), b)?))
        }
        (Value::Decimal(a), Value::Decimal(b)) => {
            Ok(Value::Decimal(decimal_binop(op, a.clone(), b.clone())?))
        }
        (Value::Float(a), Value::Decimal(b)) => {
            let a = float_to_decimal(*a)
                .ok_or_else(|| CalcError::Type(format!("cannot promote {a} to a decimal")))?;
            Ok(Value::Decimal(decimal_binop(op, a, b.clone())?))
        }
        (Value::Decimal(a), Value::Float(b)) => {
            let b = float_to_decimal(*b)
                .ok_or_else(|| CalcError::Type(format!("cannot promote {b} to a decimal")))?;
            Ok(Value::Decimal(decimal_binop(op, a.clone(), b)?))
        }
        (Value::Big(a), Value::Big(b)) => Ok(Value::Big(big_binop(op, a.clone(), b.clone())?)),
        (Value::Float(a), Value::Big(b)) => {
            let a = float_to_big(*a)
                .ok_or_else(|| CalcError::Type(format!("cannot promote {a} to big")))?;
            Ok(Value::Big(big_binop(op, a, b.clone())?))
        }
        (Value::Big(a), Value::Float(b)) => {
            let b = float_to_big(*b)
                .ok_or_else(|| CalcError::Type(format!("cannot promote {b} to big")))?;
            Ok(Value::Big(big_binop(op, a.clone(), b)?))
        }
        _ => Err(CalcError::Type(format!("cannot combine {lhs:?} and {rhs:?}"))),
    }
}

fn float_binop(op: BinOp, a: f64, b: f64) -> Result<f64, CalcError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b == 0.0 {
                Err(CalcError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => Ok(a.powf(b)),
    }
}

fn rational_binop(op: BinOp, a: BigRational, b: BigRational) -> Result<BigRational, CalcError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b == BigRational::from_integer(0.into()) {
                Err(CalcError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => Err(CalcError::Type(
            "rational exponentiation is not supported yet".into(),
        )),
    }
}

fn decimal_binop(op: BinOp, a: Decimal, b: Decimal) -> Result<Decimal, CalcError> {
    match op {
        BinOp::Add => a
            .checked_add(b)
            .ok_or_else(|| CalcError::Type("decimal overflow".into())),
        BinOp::Sub => a
            .checked_sub(b)
            .ok_or_else(|| CalcError::Type("decimal overflow".into())),
        BinOp::Mul => a
            .checked_mul(b)
            .ok_or_else(|| CalcError::Type("decimal overflow".into())),
        BinOp::Div => {
            if b.is_zero() {
                Err(CalcError::ZeroDivision)
            } else {
                a.checked_div(b)
                    .ok_or_else(|| CalcError::Type("decimal division error".into()))
            }
        }
        BinOp::Pow => Err(CalcError::Type(
            "decimal exponentiation is not supported yet".into(),
        )),
    }
}

/// Convert a float to its clean decimal representation (the shortest
/// round-trip string form), rejecting non-finite values.
fn float_to_decimal(n: f64) -> Option<Decimal> {
    n.to_string().parse().ok()
}

/// Convert a float to its clean decimal representation (shortest round-trip
/// string form), rejecting non-finite values.
fn float_to_big(n: f64) -> Option<BigDecimal> {
    n.to_string().parse().ok()
}

fn big_binop(op: BinOp, a: BigDecimal, b: BigDecimal) -> Result<BigDecimal, CalcError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b.is_zero() {
                Err(CalcError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => Err(CalcError::Type(
            "big exponentiation is not supported yet".into(),
        )),
    }
}
