use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct SetCommand<'a> {
    key: &'a str,
    value: &'a str,
    expire_seconds: Option<u64>,
    expire_milliseconds: Option<u64>,
}

impl<'a> SetCommand<'a> {
    pub fn new(key: &'a str, value: &'a str, expire_seconds: Option<u64>, expire_milliseconds: Option<u64>) -> Self {
        SetCommand { key, value, expire_seconds, expire_milliseconds }
    }

    pub fn execute(&self, db: &Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>) -> String {
        let mut db = db.lock().unwrap();
        let expire_time = if let Some(seconds) = self.expire_seconds {
            Some(Instant::now() + Duration::from_secs(seconds))
        } else if let Some(milliseconds) = self.expire_milliseconds {
            Some(Instant::now() + Duration::from_millis(milliseconds))
        } else {
            None
        };
        db.insert(self.key.to_string(), (self.value.to_string(), expire_time));
        "+OK\r\n".to_string()
    }
}