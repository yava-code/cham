mod msg;
mod script;
mod store;

use msg::{say, Lang};
use script::{App, Route, Step};
use serde_json::{json, Map, Value};
use std::path::{Component, Path};
use std::time::{SystemTime, UNIX_EPOCH};
use store::Store;
use tiny_http::{Header, Method, Request, Response, Server};

fn main() {
    let script_path = std::env::args().nth(1).unwrap_or_else(|| "app.cham".into());
    let mut lang = std::env::var("CHAM_LANG").ok().and_then(|l| Lang::parse(&l)).unwrap_or(Lang::En);

    let src = match std::fs::read_to_string(&script_path) {
        Ok(s) => s,
        Err(_) => fail(&say(lang, "no_script", &[&script_path])),
    };
    let app = match script::parse(&src, lang) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("{}", say(lang, "line", &[&script_path, &e.line.to_string(), &e.text]));
            fail(&say(lang, "script_failed", &[]));
        }
    };
    // CHAM_LANG wins over the `lang` line, so you can demo any language
    // without editing the script.
    if std::env::var("CHAM_LANG").is_err() {
        lang = app.lang.unwrap_or(lang);
    }

    // Data and static paths are relative to the script, not to wherever
    // you happened to start the binary from.
    let base = Path::new(&script_path).parent().unwrap_or(Path::new(".")).to_path_buf();
    let mut store = Store::open(base.join("data"));
    let static_dir = app.static_dir.as_ref().map(|d| base.join(d));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("127.0.0.1:{port}");
    let server = Server::http(&addr).unwrap_or_else(|_| fail(&say(lang, "port_busy", &[&port])));
    println!("{}", say(lang, "listening", &[&addr, &script_path, &app.routes.len().to_string()]));

    for mut req in server.incoming_requests() {
        let (status, body, ctype) = handle(&app, &mut store, static_dir.as_deref(), lang, &mut req);
        println!("{} {} -> {}", req.method(), req.url(), status);
        let mut res = Response::from_data(body).with_status_code(status);
        res.add_header(header("Content-Type", ctype));
        res.add_header(header("Access-Control-Allow-Origin", "*"));
        res.add_header(header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"));
        res.add_header(header("Access-Control-Allow-Headers", "Content-Type"));
        let _ = req.respond(res);
    }
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

fn handle(app: &App, store: &mut Store, static_dir: Option<&Path>, lang: Lang, req: &mut Request) -> Reply {
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
            return run(route, params, &raw, store, lang);
        }
    }

    if method == "GET" {
        if let Some(dir) = static_dir {
            if let Some(file) = serve_file(dir, &path) {
                return file;
            }
        }
    }
    let known: Vec<String> = app.routes.iter().map(|r| format!("{} {}", r.method, r.path)).collect();
    error(404, say(lang, "no_route", &[&method, &path, &known.join(", ")]))
}

/// `/orders/:id` against `/orders/7` gives {"id": "7"}.
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

fn run(route: &Route, params: Map<String, Value>, raw: &str, store: &mut Store, lang: Lang) -> Reply {
    // `input` is the request: body fields plus path params.
    let mut input = Map::new();
    if !raw.trim().is_empty() {
        match serde_json::from_str::<Value>(raw) {
            Ok(Value::Object(o)) => input = o,
            Ok(other) => return error(400, say(lang, "not_object", &[&other.to_string()])),
            Err(e) => return error(400, say(lang, "bad_json", &[&e.to_string()])),
        }
    }
    input.extend(params);

    // `result` is what the route will answer with unless a step changes it.
    let mut result = Value::Object(input.clone());

    for step in &route.steps {
        match step {
            Step::Need(fields) => {
                let missing = fields.iter().find(|f| match input.get(*f) {
                    None | Some(Value::Null) => true,
                    Some(Value::String(s)) => s.trim().is_empty(),
                    _ => false,
                });
                if let Some(f) = missing {
                    return error(422, say(lang, "missing", &[f, &fields.join(", ")]));
                }
            }
            Step::Default(k, v) => {
                input.entry(k.clone()).or_insert_with(|| guess(v));
            }
            Step::Stamp(k) => {
                let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                input.insert(k.clone(), secs.into());
            }
            Step::Unique(coll, fields) => {
                let clash = store.all(coll).iter().any(|row| fields.iter().all(|f| row.get(f) == input.get(f)));
                if clash {
                    return error(409, say(lang, "taken", &[coll, &fields.join(" + ")]));
                }
            }
            Step::Save(coll) => {
                for f in store.new_fields(coll, &input) {
                    println!("{}", say(lang, "new_field", &[coll, &f]));
                }
                match store.insert(coll, input.clone()) {
                    Ok(saved) => result = saved,
                    Err(e) => return error(500, say(lang, "disk", &[coll, &e.to_string()])),
                }
            }
            Step::List(coll) => {
                result = Value::Array(store.all(coll).clone());
            }
            Step::Find(coll, field) => {
                let want = input.get(field).cloned().unwrap_or(Value::Null);
                let hit = store.all(coll).iter().find(|row| same(row.get(field), &want)).cloned();
                match hit {
                    Some(row) => result = row,
                    None => return error(404, say(lang, "not_found", &[coll, field, &plain(&want)])),
                }
            }
            Step::Reply(status, text) => {
                let body = match text {
                    Some(t) => json!({ "message": fill(t, &input), "data": result }),
                    None => result,
                };
                return json_reply(*status, body);
            }
        }
    }
    json_reply(200, result)
}

/// `"Thanks, {name}"` -> `"Thanks, Olena"`.
fn fill(text: &str, input: &Map<String, Value>) -> String {
    let mut out = text.to_string();
    for (k, v) in input {
        out = out.replace(&format!("{{{k}}}"), &plain(v));
    }
    out
}

fn plain(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Path params arrive as text, stored ids are numbers. Compare as text.
fn same(a: Option<&Value>, b: &Value) -> bool {
    a.map(|a| plain(a) == plain(b)).unwrap_or(false)
}

fn guess(v: &str) -> Value {
    serde_json::from_str(v).unwrap_or_else(|_| Value::String(v.to_string()))
}

fn serve_file(dir: &Path, url_path: &str) -> Option<Reply> {
    let rel = Path::new(url_path.trim_start_matches('/'));
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
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
