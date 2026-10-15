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
        let body = serde_json::to_string_pretty(rows).unwrap();
        fs::create_dir_all(&self.dir)?;
        let tmp = self.path(name).with_extension("tmp");
        fs::write(&tmp, body)?;
        fs::rename(tmp, self.path(name))?;
        Ok(saved)
    }
}
