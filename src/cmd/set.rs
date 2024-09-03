use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

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

    pub fn execute(&self, db: &Db) -> String {
        let mut db = db.lock().unwrap();
        let expire_time = self.expire_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds))
            .or_else(|| self.expire_milliseconds.map(|milliseconds| Instant::now() + Duration::from_millis(milliseconds)));
        db.insert(self.key.to_string(), (self.value.to_string(), expire_time));
        "+OK\r\n".to_string()
    }
}