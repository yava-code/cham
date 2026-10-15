//! Parser for .cham files.
//!
//! A file is settings at the top (`port = 8080`) and routes below them:
//!
//!     when someone posts /orders:
//!         need name, phone
//!         order = save name, phone to orders
//!         answer "Thanks, " + name, order
//!
//! Blocks are marked by indentation, like Python. Each line is one step.

use crate::msg::{say, Lang};

#[derive(Debug, Clone)]
pub enum Expr {
    Text(String),
    Num(f64),
    /// `name` or `order.id`
    Var(Vec<String>),
    All(String),
    Now,
    Not(Box<Expr>),
    Bin(Box<Expr>, String, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Action {
    Value(Expr),
    Get(String, Vec<String>),
    Save(Vec<String>, String),
    Delete(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Step {
    Need(Vec<String>),
    Set(Option<String>, Action),
    If(Expr, Vec<Line>),
    Answer(u16, Expr, Option<Expr>),
}

#[derive(Debug, Clone)]
pub struct Line {
    pub n: usize,
    pub step: Step,
}

#[derive(Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub body: Vec<Line>,
}

#[derive(Debug, Default)]
pub struct App {
    pub settings: Vec<(String, Expr)>,
    pub routes: Vec<Route>,
}

pub const WORDS: &[&str] = &["when", "need", "if", "not", "get", "all", "save", "delete", "answer", "now"];

pub struct ParseError {
    pub line: usize,
    pub text: String,
}

type Res<T> = Result<T, String>;

// ---------- tokens ----------

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Text(String),
    Num(f64),
    Word(String),
    Sym(String),
}

fn lex(src: &str, lang: Lang) -> Res<Vec<Tok>> {
    let cs: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '"' {
            let start = i + 1;
            i += 1;
            while i < cs.len() && cs[i] != '"' {
                i += 1;
            }
            if i >= cs.len() {
                return Err(say(lang, "unclosed_quote", &[]));
            }
            out.push(Tok::Text(cs[start..i].iter().collect()));
            i += 1;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.') {
                i += 1;
            }
            let s: String = cs[start..i].iter().collect();
            out.push(Tok::Num(s.parse().unwrap_or(0.0)));
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_' || cs[i] == '.') {
                i += 1;
            }
            out.push(Tok::Word(cs[start..i].iter().collect()));
        } else {
            let two: String = cs[i..(i + 2).min(cs.len())].iter().collect();
            if ["==", "!=", ">=", "<="].contains(&two.as_str()) {
                out.push(Tok::Sym(two));
                i += 2;
            } else if "+-=<>,:()".contains(c) {
                out.push(Tok::Sym(c.to_string()));
                i += 1;
            } else {
                return Err(say(lang, "leftover", &[&c.to_string()]));
            }
        }
    }
    Ok(out)
}

// ---------- one line of tokens ----------

struct P<'a> {
    t: &'a [Tok],
    i: usize,
    lang: Lang,
}

impl<'a> P<'a> {
    fn peek(&self) -> Option<&Tok> {
        self.t.get(self.i)
    }
    fn is_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if x == s)
    }
    fn is_word(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Word(x)) if x == s)
    }
    fn word(&mut self, example: &str) -> Res<String> {
        match self.t.get(self.i) {
            Some(Tok::Word(w)) => {
                self.i += 1;
                Ok(w.clone())
            }
            _ => Err(say(self.lang, "needs_arg", &[&self.prev(), example])),
        }
    }
    fn prev(&self) -> String {
        self.t.get(self.i.saturating_sub(1)).map(show).unwrap_or_default()
    }
    /// `a, b, c` up to (not including) `stop`
    fn names(&mut self, stop: Option<&str>, example: &str) -> Res<Vec<String>> {
        let mut v = vec![self.word(example)?];
        while self.is_sym(",") {
            self.i += 1;
            v.push(self.word(example)?);
        }
        if let Some(s) = stop {
            if !self.is_word(s) {
                return Err(say(self.lang, "needs_arg", &[&self.prev(), example]));
            }
        }
        Ok(v)
    }
    fn done(&self) -> Res<()> {
        match self.peek() {
            None => Ok(()),
            Some(t) => Err(say(self.lang, "leftover", &[&show(t)])),
        }
    }

    fn expr(&mut self) -> Res<Expr> {
        let l = self.add()?;
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if self.is_sym(op) {
                self.i += 1;
                let r = self.add()?;
                return Ok(Expr::Bin(Box::new(l), op.into(), Box::new(r)));
            }
        }
        Ok(l)
    }
    fn add(&mut self) -> Res<Expr> {
        let mut l = self.unary()?;
        while self.is_sym("+") || self.is_sym("-") {
            let op = if self.is_sym("+") { "+" } else { "-" };
            self.i += 1;
            let r = self.unary()?;
            l = Expr::Bin(Box::new(l), op.into(), Box::new(r));
        }
        Ok(l)
    }
    fn unary(&mut self) -> Res<Expr> {
        if self.is_word("not") {
            self.i += 1;
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        self.primary()
    }
    fn primary(&mut self) -> Res<Expr> {
        let tok = self.peek().cloned();
        self.i += 1;
        match tok {
            Some(Tok::Text(s)) => Ok(Expr::Text(s)),
            Some(Tok::Num(n)) => Ok(Expr::Num(n)),
            Some(Tok::Sym(s)) if s == "(" => {
                let e = self.expr()?;
                if !self.is_sym(")") {
                    return Err(say(self.lang, "unclosed_paren", &[]));
                }
                self.i += 1;
                Ok(e)
            }
            Some(Tok::Word(w)) if w == "now" => Ok(Expr::Now),
            Some(Tok::Word(w)) if w == "all" => Ok(Expr::All(self.word("all orders")?)),
            Some(Tok::Word(w)) => {
                // `foo "x"` or `foo bar` reads like a call to a function we don't have.
                if matches!(self.peek(), Some(Tok::Word(_) | Tok::Text(_) | Tok::Num(_))) {
                    return Err(unknown(&w, self.lang));
                }
                Ok(Expr::Var(w.split('.').map(String::from).collect()))
            }
            Some(t) => Err(say(self.lang, "leftover", &[&show(&t)])),
            None => Err(say(self.lang, "needs_arg", &[&self.prev(), "answer \"Hi, \" + name"])),
        }
    }

    fn action(&mut self) -> Res<Action> {
        if self.is_word("get") {
            self.i += 1;
            let coll = self.word("get orders id")?;
            return Ok(Action::Get(coll, self.names(None, "get orders id")?));
        }
        if self.is_word("delete") {
            self.i += 1;
            let coll = self.word("delete orders id")?;
            return Ok(Action::Delete(coll, self.names(None, "delete orders id")?));
        }
        if self.is_word("save") {
            self.i += 1;
            let fields = self.names(Some("to"), "save name, phone to orders")?;
            self.i += 1;
            return Ok(Action::Save(fields, self.word("save name, phone to orders")?));
        }
        Ok(Action::Value(self.expr()?))
    }
}

fn show(t: &Tok) -> String {
    match t {
        Tok::Text(s) => format!("\"{s}\""),
        Tok::Num(n) => n.to_string(),
        Tok::Word(w) | Tok::Sym(w) => w.clone(),
    }
}

fn unknown(w: &str, lang: Lang) -> String {
    let hint = closest(w).map(|c| say(lang, "did_you_mean", &[c])).unwrap_or_default();
    say(lang, "unknown_word", &[w, &hint])
}

/// Parses one step. Returns the step and whether it is an `if` that wants an
/// indented block below it.
fn step(toks: &[Tok], lang: Lang) -> Res<(Step, bool)> {
    let mut p = P { t: toks, i: 0, lang };
    let first = match p.peek() {
        Some(Tok::Word(w)) => w.clone(),
        Some(t) => return Err(say(lang, "leftover", &[&show(t)])),
        None => return Err(String::new()),
    };
    match first.as_str() {
        "need" => {
            p.i += 1;
            let v = p.names(None, "need name, phone")?;
            p.done()?;
            Ok((Step::Need(v), false))
        }
        "if" => {
            p.i += 1;
            let cond = p.expr()?;
            if !p.is_sym(":") {
                return Err(say(lang, "need_colon", &["if not order: answer 404 \"Not found\""]));
            }
            p.i += 1;
            if p.peek().is_none() {
                return Ok((Step::If(cond, Vec::new()), true));
            }
            let (inner, _) = step(&toks[p.i..], lang)?;
            Ok((Step::If(cond, vec![Line { n: 0, step: inner }]), false))
        }
        "answer" => {
            p.i += 1;
            let mut status = 200;
            if let (Some(Tok::Num(n)), Some(_)) = (p.t.get(p.i), p.t.get(p.i + 1)) {
                if !matches!(p.t.get(p.i + 1), Some(Tok::Sym(s)) if s == "+" || s == "-") {
                    status = *n as u16;
                    if !(100..600).contains(&status) {
                        return Err(say(lang, "bad_status", &[&n.to_string()]));
                    }
                    p.i += 1;
                }
            }
            let main = p.expr()?;
            let extra = if p.is_sym(",") {
                p.i += 1;
                Some(p.expr()?)
            } else {
                None
            };
            p.done()?;
            Ok((Step::Answer(status, main, extra), false))
        }
        "get" | "save" | "delete" => {
            let a = p.action()?;
            p.done()?;
            Ok((Step::Set(None, a), false))
        }
        _ if matches!(toks.get(1), Some(Tok::Sym(s)) if s == "=") => {
            p.i += 2;
            let a = p.action()?;
            p.done()?;
            Ok((Step::Set(Some(first), a), false))
        }
        _ => Err(unknown(&first, lang)),
    }
}

// ---------- the file ----------

struct Raw {
    n: usize,
    indent: usize,
    text: String,
}

pub fn parse(src: &str, lang: Lang) -> Result<App, ParseError> {
    let lines: Vec<Raw> = src
        .lines()
        .enumerate()
        .filter_map(|(i, l)| {
            let text = strip_comment(l).trim_end();
            if text.trim().is_empty() {
                return None;
            }
            let indent = text.len() - text.trim_start().len();
            Some(Raw { n: i + 1, indent, text: text.trim().to_string() })
        })
        .collect();

    let mut app = App::default();
    let mut i = 0;
    while i < lines.len() {
        let l = &lines[i];
        let fail = |text: String| ParseError { line: l.n, text };
        if l.indent > 0 {
            return Err(fail(say(lang, "bad_indent", &[])));
        }
        if l.text.starts_with("when") {
            let (method, path) = route_head(&l.text, lang).map_err(fail)?;
            i += 1;
            let body = block(&lines, &mut i, 0, lang)?;
            app.routes.push(Route { method, path, body });
            continue;
        }
        let toks = lex(&l.text, lang).map_err(fail)?;
        match step(&toks, lang).map_err(fail)? {
            (Step::Set(Some(name), Action::Value(e)), _) => app.settings.push((name, e)),
            _ => {
                let w = l.text.split_whitespace().next().unwrap_or("").to_string();
                return Err(fail(say(lang, "not_indented", &[&w])));
            }
        }
        i += 1;
    }
    Ok(app)
}

fn route_head(text: &str, lang: Lang) -> Res<(String, String)> {
    let Some(head) = text.strip_suffix(':') else {
        return Err(say(lang, "need_colon", &["when someone posts /orders:"]));
    };
    let words: Vec<&str> = head.split_whitespace().filter(|w| *w != "someone").collect();
    let method = match words.get(1) {
        Some(&"posts") => "POST",
        Some(&"opens") => "GET",
        Some(&"deletes") => "DELETE",
        _ => return Err(say(lang, "bad_when", &[])),
    };
    match words.get(2) {
        Some(p) if p.starts_with('/') && words.len() == 3 => Ok((method.into(), p.to_string())),
        _ => Err(say(lang, "bad_when", &[])),
    }
}

/// Reads lines indented deeper than `parent` into one block.
fn block(lines: &[Raw], i: &mut usize, parent: usize, lang: Lang) -> Result<Vec<Line>, ParseError> {
    let mut out = Vec::new();
    let Some(first) = lines.get(*i) else { return Ok(out) };
    let indent = first.indent;
    if indent <= parent {
        return Ok(out);
    }
    while let Some(l) = lines.get(*i) {
        if l.indent < indent {
            if l.indent > parent {
                return Err(ParseError { line: l.n, text: say(lang, "bad_indent", &[]) });
            }
            break;
        }
        let fail = |text: String| ParseError { line: l.n, text };
        if l.indent > indent {
            return Err(fail(say(lang, "bad_indent", &[])));
        }
        if l.text.starts_with("when ") {
            return Err(fail(say(lang, "bad_indent", &[])));
        }
        let toks = lex(&l.text, lang).map_err(fail)?;
        let (mut st, wants_block) = step(&toks, lang).map_err(fail)?;
        let n = l.n;
        *i += 1;
        if wants_block {
            let inner = block(lines, i, indent, lang)?;
            if inner.is_empty() {
                return Err(ParseError { line: n, text: say(lang, "empty_if", &[]) });
            }
            if let Step::If(_, body) = &mut st {
                *body = inner;
            }
        } else if let Step::If(_, body) = &mut st {
            body[0].n = n;
        }
        out.push(Line { n, step: st });
    }
    Ok(out)
}

fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

fn closest(word: &str) -> Option<&'static str> {
    WORDS
        .iter()
        .map(|w| (distance(word, w), *w))
        .filter(|(d, _)| *d <= 2)
        .min()
        .map(|(_, w)| w)
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == *cb { 0 } else { 1 };
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}
