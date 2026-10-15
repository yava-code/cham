mod msg;
mod script;
mod store;

use msg::{say, Lang};
use script::{Action, App, Expr, Line, Route, Step};
use serde_json::{json, Map, Value};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use store::Store;
use tiny_http::{Header, Method, Request, Response, Server};

const NEW_APP: &str = include_str!("../templates/app.cham");
const NEW_PAGE: &str = include_str!("../templates/index.html");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let env_lang = std::env::var("CHAM_LANG").ok().and_then(|l| Lang::parse(&l));
    let mut lang = env_lang.unwrap_or(Lang::En);

    if args.first().map(String::as_str) == Some("new") {
        let name = args.get(1).cloned().unwrap_or_else(|| "myapp".into());
        scaffold(&name, lang);
        return;
    }

    // `cham`, `cham myapp` or `cham myapp/app.cham`
    let mut script_path = PathBuf::from(args.first().cloned().unwrap_or_else(|| ".".into()));
    if script_path.is_dir() {
        script_path = script_path.join("app.cham");
    }
    let shown = script_path.display().to_string();
    let src = std::fs::read_to_string(&script_path).unwrap_or_else(|_| fail(&say(lang, "no_script", &[&shown])));

    // Settings come first, so read them before anything can go wrong.
    let parsed = script::parse(&src, lang);
    let mut settings = Map::new();
    if let Ok(app) = &parsed {
        for (k, e) in &app.settings {
            if let Ok(v) = eval(e, &settings, &mut Store::open("."), lang, 0) {
                settings.insert(k.clone(), v);
            }
        }
    }
    if env_lang.is_none() {
        if let Some(l) = settings.get("lang").and_then(Value::as_str).and_then(Lang::parse) {
            lang = l;
        }
    }
    // Re-parse so script errors come out in the language the script asked for.
    let app = script::parse(&src, lang);

    let base = script_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut store = Store::open(base.join("data"));
    let site = if base.join("public").is_dir() { base.join("public") } else { base.clone() };

    let port = std::env::var("PORT")
        .ok()
        .or_else(|| settings.get("port").map(|v| plain(v)))
        .unwrap_or_else(|| "8080".into());
    let addr = format!("127.0.0.1:{port}");
    let server = Server::http(&addr).unwrap_or_else(|_| fail(&say(lang, "port_busy", &[&port])));

    let broken = match &app {
        Ok(app) => {
            println!("{}", say(lang, "listening", &[&addr, &shown, &app.routes.len().to_string()]));
            println!("{}", say(lang, "routes", &[]));
            for r in &app.routes {
                println!("  {:<6} {}", r.method, r.path);
            }
            println!("{}\n", say(lang, "cheat", &[]));
            None
        }
        Err(e) => {
            // Keep serving, so the browser shows the same message as the console.
            let text = say(lang, "line", &[&shown, &e.line.to_string(), &e.text]);
            eprintln!("{text}");
            Some(text)
        }
    };

    for mut req in server.incoming_requests() {
        let reply = match (&app, &broken) {
            (Ok(app), _) => handle(app, &settings, &mut store, &site, lang, &mut req),
            (_, Some(text)) => error(500, say(lang, "broken_script", &[text])),
            _ => unreachable!(),
        };
        println!("{} {} -> {}", req.method(), req.url(), reply.0);
        let mut res = Response::from_data(reply.1).with_status_code(reply.0);
        res.add_header(header("Content-Type", reply.2));
        res.add_header(header("Access-Control-Allow-Origin", "*"));
        res.add_header(header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS"));
        res.add_header(header("Access-Control-Allow-Headers", "Content-Type"));
        let _ = req.respond(res);
    }
}

fn scaffold(name: &str, lang: Lang) {
    let dir = Path::new(name);
    if dir.exists() {
        fail(&say(lang, "exists", &[name]));
    }
    let write = |file: &str, body: &str| {
        std::fs::create_dir_all(dir).and_then(|_| std::fs::write(dir.join(file), body))
    };
    if let Err(e) = write("app.cham", NEW_APP).and_then(|_| write("index.html", NEW_PAGE)) {
        fail(&e.to_string());
    }
    println!("{}", say(lang, "created", &[name]));
}

fn fail(text: &str) -> ! {
    eprintln!("{text}");
    std::process::exit(1)
}

fn header(k: &str, v: &str) -> Header {
    Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap()
}

type Reply = (u16, Vec<u8>, &'static str);

fn json_reply(status: u16, v: Value) -> Reply {
    (status, serde_json::to_vec_pretty(&v).unwrap(), "application/json; charset=utf-8")
}

fn error(status: u16, text: String) -> Reply {
    eprintln!("  {text}");
    json_reply(status, json!({ "error": text }))
}

fn handle(app: &App, settings: &Map<String, Value>, store: &mut Store, site: &Path, lang: Lang, req: &mut Request) -> Reply {
    if *req.method() == Method::Options {
        return (204, Vec::new(), "text/plain");
    }
    let method = req.method().as_str().to_uppercase();
    let path = req.url().split('?').next().unwrap_or("/").to_string();

    for route in &app.routes {
        if route.method != method {
            continue;
        }
        if let Some(params) = match_path(&route.path, &path) {
            let mut raw = String::new();
            let _ = req.as_reader().read_to_string(&mut raw);
            return run(route, settings, params, &raw, store, lang);
        }
    }
    if method == "GET" {
        if let Some(file) = serve_file(site, &path) {
            return file;
        }
    }
    let known: Vec<String> = app.routes.iter().map(|r| format!("{} {}", r.method, r.path)).collect();
    error(404, say(lang, "no_route", &[&method, &path, &known.join(", ")]))
}

fn match_path(pattern: &str, path: &str) -> Option<Map<String, Value>> {
    let a: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let b: Vec<&str> = path.trim_matches('/').split('/').collect();
    if a.len() != b.len() {
        return None;
    }
    let mut params = Map::new();
    for (p, s) in a.iter().zip(&b) {
        match p.strip_prefix(':') {
            Some(name) => {
                params.insert(name.to_string(), Value::String(s.to_string()));
            }
            None if p == s => {}
            None => return None,
        }
    }
    Some(params)
}

/// What a step can end with: keep going, or an answer to send.
enum Flow {
    Next,
    Done(Reply),
}

fn run(route: &Route, settings: &Map<String, Value>, params: Map<String, Value>, raw: &str, store: &mut Store, lang: Lang) -> Reply {
    // `vars` starts as settings + request body + path params. Assignments add to it.
    let mut vars = settings.clone();
    if !raw.trim().is_empty() {
        match serde_json::from_str::<Value>(raw) {
            Ok(Value::Object(o)) => vars.extend(o),
            Ok(other) => return error(400, say(lang, "not_object", &[&other.to_string()])),
            Err(e) => return error(400, say(lang, "bad_json", &[&e.to_string()])),
        }
    }
    vars.extend(params);

    match steps(&route.body, &mut vars, store, lang) {
        Ok(Flow::Done(r)) => r,
        Ok(Flow::Next) => json_reply(200, json!({})),
        Err(r) => r,
    }
}

fn steps(lines: &[Line], vars: &mut Map<String, Value>, store: &mut Store, lang: Lang) -> Result<Flow, Reply> {
    for line in lines {
        let n = line.n;
        match &line.step {
            Step::Need(fields) => {
                let missing = fields.iter().find(|f| match vars.get(*f) {
                    None | Some(Value::Null) => true,
                    Some(Value::String(s)) => s.trim().is_empty(),
                    _ => false,
                });
                if let Some(f) = missing {
                    return Err(error(422, say(lang, "missing", &[f, &fields.join(", ")])));
                }
            }
            Step::Set(name, action) => {
                let v = act(action, vars, store, lang, n)?;
                if let Some(name) = name {
                    vars.insert(name.clone(), v);
                }
            }
            Step::If(cond, body) => {
                if truthy(&eval(cond, vars, store, lang, n)?) {
                    if let Flow::Done(r) = steps(body, vars, store, lang)? {
                        return Ok(Flow::Done(r));
                    }
                }
            }
            Step::Answer(status, main, extra) => {
                let main = eval(main, vars, store, lang, n)?;
                let body = match (main, extra) {
                    (Value::String(s), _) if *status >= 400 => json!({ "error": s }),
                    (Value::String(s), Some(e)) => json!({ "message": s, "data": eval(e, vars, store, lang, n)? }),
                    (Value::String(s), None) => json!({ "message": s }),
                    (v, _) => v,
                };
                return Ok(Flow::Done(json_reply(*status, body)));
            }
        }
    }
    Ok(Flow::Next)
}

fn pick(fields: &[String], vars: &Map<String, Value>, lang: Lang, n: usize) -> Result<Map<String, Value>, Reply> {
    let mut m = Map::new();
    for f in fields {
        let v = vars.get(f).ok_or_else(|| error(400, say(lang, "undefined", &[f, &n.to_string()])))?;
        m.insert(f.clone(), v.clone());
    }
    Ok(m)
}

fn act(a: &Action, vars: &Map<String, Value>, store: &mut Store, lang: Lang, n: usize) -> Result<Value, Reply> {
    match a {
        Action::Value(e) => eval(e, vars, store, lang, n),
        Action::Get(coll, fields) => Ok(store.find(coll, &pick(fields, vars, lang, n)?).unwrap_or(Value::Null)),
        Action::Save(fields, coll) => {
            let rec = pick(fields, vars, lang, n)?;
            for f in store.new_fields(coll, &rec) {
                println!("{}", say(lang, "new_field", &[coll, &f]));
            }
            store.insert(coll, rec).map_err(|e| error(500, say(lang, "disk", &[coll, &e.to_string()])))
        }
        Action::Delete(coll, fields) => store
            .remove(coll, &pick(fields, vars, lang, n)?)
            .map(|k| Value::from(k as u64))
            .map_err(|e| error(500, say(lang, "disk", &[coll, &e.to_string()]))),
    }
}

fn eval(e: &Expr, vars: &Map<String, Value>, store: &mut Store, lang: Lang, n: usize) -> Result<Value, Reply> {
    Ok(match e {
        Expr::Text(s) => Value::String(s.clone()),
        Expr::Num(x) => num(*x),
        Expr::Now => SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0).into(),
        Expr::All(coll) => Value::Array(store.all(coll).clone()),
        Expr::Not(x) => (!truthy(&eval(x, vars, store, lang, n)?)).into(),
        Expr::Var(path) => {
            let mut v = vars.get(&path[0]).ok_or_else(|| error(400, say(lang, "undefined", &[&path[0], &n.to_string()])))?;
            for key in &path[1..] {
                v = v.get(key).unwrap_or(&Value::Null);
            }
            v.clone()
        }
        Expr::Bin(l, op, r) => {
            let (a, b) = (eval(l, vars, store, lang, n)?, eval(r, vars, store, lang, n)?);
            match (op.as_str(), a.as_f64(), b.as_f64()) {
                ("+", Some(x), Some(y)) => num(x + y),
                ("+", _, _) => Value::String(plain(&a) + &plain(&b)),
                ("-", Some(x), Some(y)) => num(x - y),
                ("==", _, _) => (plain(&a) == plain(&b)).into(),
                ("!=", _, _) => (plain(&a) != plain(&b)).into(),
                (">", Some(x), Some(y)) => (x > y).into(),
                ("<", Some(x), Some(y)) => (x < y).into(),
                (">=", Some(x), Some(y)) => (x >= y).into(),
                ("<=", Some(x), Some(y)) => (x <= y).into(),
                _ => Value::Null,
            }
        }
    })
}

fn num(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 1e15 { Value::from(x as i64) } else { Value::from(x) }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(_) => true,
    }
}

fn plain(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn serve_file(dir: &Path, url_path: &str) -> Option<Reply> {
    let rel = Path::new(url_path.trim_start_matches('/'));
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    // The script and the data folder are not part of the website.
    if rel.starts_with("data") || rel.extension().is_some_and(|e| e == "cham") {
        return None;
    }
    let mut file = dir.join(rel);
    if file.is_dir() {
        file = file.join("index.html");
    }
    let bytes = std::fs::read(&file).ok()?;
    let ctype = match file.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    };
    Some((200, bytes, ctype))
}
