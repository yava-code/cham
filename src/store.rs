//! A collection is a JSON array in data/<name>.json, kept in memory and
//! rewritten on every save. Fine for a shop with a few thousand orders;
//! swap it for SQLite when that stops being true.

use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

pub struct Store {
    dir: PathBuf,
    colls: HashMap<String, Vec<Value>>,
}

impl Store {
    pub fn open(dir: impl Into<PathBuf>) -> Store {
        Store { dir: dir.into(), colls: HashMap::new() }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.json"))
    }

    pub fn all(&mut self, name: &str) -> &mut Vec<Value> {
        if !self.colls.contains_key(name) {
            let rows = fs::read_to_string(self.path(name))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            self.colls.insert(name.to_string(), rows);
        }
        self.colls.get_mut(name).unwrap()
    }

    /// Fields that no earlier record in this collection had.
    pub fn new_fields(&mut self, name: &str, rec: &Map<String, Value>) -> Vec<String> {
        let known: BTreeSet<&String> = self
            .all(name)
            .iter()
            .filter_map(Value::as_object)
            .flat_map(|o| o.keys())
            .collect();
        if known.is_empty() {
            return Vec::new();
        }
        rec.keys().filter(|k| !known.contains(k)).cloned().collect()
    }

    pub fn insert(&mut self, name: &str, mut rec: Map<String, Value>) -> std::io::Result<Value> {
        let rows = self.all(name);
        let id = rows.iter().filter_map(|r| r["id"].as_u64()).max().unwrap_or(0) + 1;
        rec.insert("id".into(), id.into());
        rows.push(Value::Object(rec));
        let saved = rows.last().cloned().unwrap();
        self.flush(name)?;
        Ok(saved)
    }

    /// First record whose fields all equal `want` (compared as text, so the
    /// "7" from a URL matches the number 7 on disk).
    pub fn find(&mut self, name: &str, want: &Map<String, Value>) -> Option<Value> {
        self.all(name).iter().find(|r| matches(r, want)).cloned()
    }

    pub fn remove(&mut self, name: &str, want: &Map<String, Value>) -> std::io::Result<usize> {
        let rows = self.all(name);
        let before = rows.len();
        rows.retain(|r| !matches(r, want));
        let gone = before - rows.len();
        if gone > 0 {
            self.flush(name)?;
        }
        Ok(gone)
    }

    fn flush(&mut self, name: &str) -> std::io::Result<()> {
        let body = serde_json::to_string_pretty(self.all(name)).unwrap();
        fs::create_dir_all(&self.dir)?;
        let tmp = self.path(name).with_extension("tmp");
        fs::write(&tmp, body)?;
        fs::rename(tmp, self.path(name))
    }
}

fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn matches(row: &Value, want: &Map<String, Value>) -> bool {
    want.iter().all(|(k, v)| row.get(k).map(|r| text(r) == text(v)).unwrap_or(false))
}
