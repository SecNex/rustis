use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

pub struct IncrCommand {
    key: String,
}

impl IncrCommand {
    pub fn new(key: &str) -> Self {
        IncrCommand {
            key: key.to_string(),
        }
    }

    pub fn execute(&self, db: &Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>) -> String {
        let mut db = db.lock().unwrap();
        let entry = db.entry(self.key.clone()).or_insert(("0".to_string(), None));

        match entry.0.parse::<i64>() {
            Ok(mut value) => {
                value += 1;
                entry.0 = value.to_string();
                format!(":{}\r\n", value)
            }
            Err(_) => "-ERR value is not an integer or out of range\r\n".to_string(),
        }
    }
}