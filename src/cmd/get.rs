use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

pub struct GetCommand<'a> {
    key: &'a str,
}

impl<'a> GetCommand<'a> {
    pub fn new(key: &'a str) -> Self {
        GetCommand { key }
    }

    pub fn execute(&self, db: &Db) -> String {
        let mut db = db.lock().unwrap();
        if let Some((value, expire_time)) = db.get(self.key) {
            if let Some(expire_time) = expire_time {
                if Instant::now() > *expire_time {
                    db.remove(self.key);
                    return "$-1\r\n".to_string(); // Key not found
                }
            }
            format!("${}\r\n{}\r\n", value.len(), value)
        } else {
            "$-1\r\n".to_string() // Key not found
        }
    }
}