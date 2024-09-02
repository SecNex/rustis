use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant};

pub struct TTLCommand<'a> {
    key: &'a str,
}

impl<'a> TTLCommand<'a> {
    pub fn new(key: &'a str) -> Self {
        TTLCommand { key }
    }

    pub fn execute(&self, db: &Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>) -> String {
        let db = db.lock().unwrap();
        if let Some((_, Some(expire_time))) = db.get(self.key) {
            let ttl = expire_time.saturating_duration_since(Instant::now()).as_secs();
            format!(":{}\r\n", ttl)
        } else {
            ":-1\r\n".to_string() // -1 indicates that the key does not have an associated expire time
        }
    }
}