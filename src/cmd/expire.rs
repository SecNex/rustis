use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

pub struct ExpireCommand<'a> {
    key: &'a str,
    seconds: u64,
}

impl<'a> ExpireCommand<'a> {
    pub fn new(key: &'a str, seconds: u64) -> Self {
        ExpireCommand { key, seconds }
    }

    pub fn execute(&self, db: &Db) -> String {
        let mut db = db.lock().unwrap();
        if let Some((value, _)) = db.get(self.key).cloned() {
            let expire_time = Instant::now() + Duration::from_secs(self.seconds);
            db.insert(self.key.to_string(), (value, Some(expire_time)));
            "+OK\r\n".to_string()
        } else {
            "-ERR no such key\r\n".to_string()
        }
    }
}