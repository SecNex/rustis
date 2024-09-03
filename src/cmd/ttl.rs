use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant};

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

pub struct TTLCommand<'a> {
    key: &'a str,
}

impl<'a> TTLCommand<'a> {
    pub fn new(key: &'a str) -> Self {
        TTLCommand { key }
    }

    pub fn execute(&self, db: &Db) -> String {
        let db = db.lock().unwrap();
        if let Some((_, Some(expire_time))) = db.get(self.key) {
            let ttl = expire_time.saturating_duration_since(Instant::now()).as_secs();
            format!(":{}\r\n", ttl)
        } else {
            ":-1\r\n".to_string() // -1 indicates that the key does not have an associated expire time
        }
    }
}