use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

pub struct ExistsCommand {
    keys: Vec<String>,
}

impl ExistsCommand {
    pub fn new(keys: Vec<String>) -> Self {
        ExistsCommand { keys }
    }

    pub fn execute(&self, db: &Db) -> String {
        let db = db.lock().unwrap();
        let mut count = 0;

        for key in &self.keys {
            if db.contains_key(key) {
                count += 1;
            }
        }

        format!(":{}\r\n", count)
    }
}