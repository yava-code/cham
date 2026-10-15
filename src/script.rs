//! Parser for .cham files. One instruction per line, `#` starts a comment.
//! A route line (`post /orders`) opens a block; the step lines below it run
//! top to bottom for every request.

use crate::msg::{say, Lang};

#[derive(Debug, Clone)]
pub enum Step {
    Need(Vec<String>),
    Default(String, String),
    Stamp(String),
    Unique(String, Vec<String>),
    Save(String),
    List(String),
    Find(String, String),
    Reply(u16, Option<String>),
}

#[derive(Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Default)]
pub struct App {
    pub lang: Option<Lang>,
    pub static_dir: Option<String>,
    pub routes: Vec<Route>,
}

const WORDS: &[&str] = &[
    "lang", "static", "get", "post", "put", "delete", "need", "default",
    "stamp", "unique", "save", "list", "find", "reply",
];

/// Parse error carries the 1-based line number and a ready-to-print text.
pub struct ParseError {
    pub line: usize,
    pub text: String,
}

pub fn parse(src: &str, lang: Lang) -> Result<App, ParseError> {
    let mut app = App::default();
    for (i, raw) in src.lines().enumerate() {
        let n = i + 1;
        let fail = |text: String| ParseError { line: n, text };
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let (word, rest) = match line.split_once(char::is_whitespace) {
            Some((w, r)) => (w.to_lowercase(), r.trim()),
            None => (line.to_lowercase(), ""),
        };
        let need_arg = |example: &str| {
            if rest.is_empty() {
                Err(fail(say(lang, "needs_arg", &[&word, example])))
            } else {
                Ok(())
            }
        };

        match word.as_str() {
            "lang" => {
                need_arg("lang uk")?;
                app.lang = Lang::parse(rest);
            }
            "static" => {
                need_arg("static ./public")?;
                app.static_dir = Some(rest.to_string());
            }
            "get" | "post" | "put" | "delete" => {
                need_arg("post /orders")?;
                app.routes.push(Route {
                    method: word.to_uppercase(),
                    path: rest.to_string(),
                    steps: Vec::new(),
                });
            }
            _ => {
                let step = match word.as_str() {
                    "need" => {
                        need_arg("need name, phone")?;
                        Step::Need(split_list(rest))
                    }
                    "default" => {
                        need_arg("default size = large")?;
                        let (k, v) = rest.split_once('=').ok_or_else(|| {
                            fail(say(lang, "needs_arg", &["default", "default size = large"]))
                        })?;
                        Step::Default(k.trim().into(), unquote(v.trim()))
                    }
                    "stamp" => Step::Stamp(if rest.is_empty() { "created_at".into() } else { rest.into() }),
                    "unique" => {
                        need_arg("unique bookings device, slot")?;
                        let mut parts = rest.splitn(2, char::is_whitespace);
                        let coll = parts.next().unwrap_or("").to_string();
                        let fields = split_list(parts.next().unwrap_or(""));
                        if fields.is_empty() {
                            return Err(fail(say(lang, "needs_arg", &["unique", "unique bookings device, slot"])));
                        }
                        Step::Unique(coll, fields)
                    }
                    "save" => {
                        need_arg("save orders")?;
                        Step::Save(rest.into())
                    }
                    "list" => {
                        need_arg("list orders")?;
                        Step::List(rest.into())
                    }
                    "find" => {
                        need_arg("find orders by id")?;
                        let (coll, field) = rest.split_once(" by ").ok_or_else(|| {
                            fail(say(lang, "needs_arg", &["find", "find orders by id"]))
                        })?;
                        Step::Find(coll.trim().into(), field.trim().into())
                    }
                    "reply" => {
                        need_arg("reply 201 \"Thanks!\"")?;
                        let (code, text) = match rest.split_once(char::is_whitespace) {
                            Some((c, t)) => (c, Some(t.trim())),
                            None => (rest, None),
                        };
                        let status: u16 = code
                            .parse()
                            .ok()
                            .filter(|s| (100..600).contains(s))
                            .ok_or_else(|| fail(say(lang, "bad_status", &[code])))?;
                        let text = match text {
                            Some(t) if t.starts_with('"') && !(t.len() > 1 && t.ends_with('"')) => {
                                return Err(fail(say(lang, "unclosed_quote", &[])));
                            }
                            Some(t) => Some(unquote(t)),
                            None => None,
                        };
                        Step::Reply(status, text)
                    }
                    other => {
                        let hint = closest(other)
                            .map(|w| say(lang, "did_you_mean", &[w]))
                            .unwrap_or_default();
                        return Err(fail(say(lang, "unknown_word", &[other, &hint])));
                    }
                };
                match app.routes.last_mut() {
                    Some(r) => r.steps.push(step),
                    None => return Err(fail(say(lang, "step_outside", &[&word]))),
                }
            }
        }
    }
    Ok(app)
}

fn strip_comment(line: &str) -> &str {
    // A # inside quotes is text, not a comment.
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

fn split_list(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .map(String::from)
        .collect()
}

fn unquote(s: &str) -> String {
    s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s).to_string()
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
