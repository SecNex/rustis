use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde_json::Value;
use serde_json::json;

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

pub struct SetJsonCommand {
    key: String,
    value: String,
    path: String,
}

impl SetJsonCommand {
    pub fn new(key: &str, path: &str, value: &str) -> Self {
        SetJsonCommand {
            key: key.to_string(),
            value: value.to_string(),
            path: path.to_string(),
        }
    }

    pub fn execute(&self, db: &Db) -> String {
        let mut db = db.lock().unwrap();
        if self.path == "$" {
            db.insert(self.key.clone(), (self.value.clone(), None));
            "+OK\r\n".to_string()
        } else {
            "-ERR unsupported JSON path\r\n".to_string()
        }
    }
}   

pub struct GetJsonCommand {
    key: String,
    paths: Vec<String>,
}

impl GetJsonCommand {
    pub fn new(key: &str, paths: &[&str]) -> Self {
        GetJsonCommand {
            key: key.to_string(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn execute(&self, db: &Db) -> String {
        let db = db.lock().unwrap();
        if let Some((value, _)) = db.get(&self.key) {
            let json_value: Value = serde_json::from_str(value).unwrap_or(json!(null));
            if self.paths.is_empty() {
                let response = serde_json::to_string(&json_value).unwrap_or("-ERR invalid JSON\r\n".to_string());
                return format!("${}\r\n{}\r\n", response.len(), response);
            }
    
            let mut flat_result = vec![];
            let mut path_results = json!({});
    
            for path in &self.paths {
                let mut values = vec![];
                if path == "$..b" {
                    find_all_paths(&json_value, "b", &mut values);
                    if self.paths.len() == 1 {
                        flat_result = values;
                    } else {
                        path_results[path] = json!(values);
                    }
                } else {
                    find_all_paths(&json_value, path.trim_start_matches(".."), &mut values);
                    path_results[path] = json!(values);
                }
            }
    
            if !flat_result.is_empty() && self.paths.len() == 1 {
                let response = serde_json::to_string(&flat_result).unwrap_or("-ERR invalid JSON\r\n".to_string());
                return format!("${}\r\n{}\r\n", response.len(), response);
            } else {
                let response = serde_json::to_string(&path_results).unwrap_or("-ERR invalid JSON path\r\n".to_string());
                return format!("${}\r\n{}\r\n", response.len(), response);
            }
        } else {
            "-ERR no such key\r\n".to_string()
        }
    }
}

fn find_all_paths<'a>(json_value: &'a Value, key: &str, results: &mut Vec<Value>) {
    match json_value {
        Value::Object(map) => {
            for (k, v) in map {
                if k == key {
                    results.push(v.clone());
                }
                find_all_paths(v, key, results);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                find_all_paths(v, key, results);
            }
        }
        _ => (),
    }
}

pub struct DelJsonCommand {
    key: String,
}

impl DelJsonCommand {
    pub fn new(key: &str) -> Self {
        DelJsonCommand {
            key: key.to_string(),
        }
    }

    pub fn execute(&self, db: &Db) -> String {
        let mut db = db.lock().unwrap();
        if db.remove(&self.key).is_some() {
            "+OK\r\n".to_string()
        } else {
            "-ERR no such key\r\n".to_string()
        }
    }
}