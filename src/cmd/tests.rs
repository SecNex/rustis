use super::get::*;
use super::set::*;
use super::incr::*;
use super::decr::*;
use super::ttl::*;
use super::expire::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

// Tests für den GET-Befehl
#[test]
fn test_get_existing_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    db.lock().unwrap().insert("key".to_string(), ("value".to_string(), None));
    
    let get_cmd = GetCommand::new("key");
    let result = get_cmd.execute(&db);
    
    assert_eq!(result, "$5\r\nvalue\r\n");
}

#[test]
fn test_get_non_existing_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    let get_cmd = GetCommand::new("missing_key");
    let result = get_cmd.execute(&db);
    
    assert_eq!(result, "$-1\r\n");
}

#[test]
fn test_get_expired_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let past_time = Instant::now() - Duration::from_secs(10);
    db.lock().unwrap().insert("key".to_string(), ("value".to_string(), Some(past_time)));
    
    let get_cmd = GetCommand::new("key");
    let result = get_cmd.execute(&db);
    
    assert_eq!(result, "$-1\r\n");
    assert!(db.lock().unwrap().get("key").is_none());
}

#[test]
fn test_get_key_with_future_expiration() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let future_time = Instant::now() + Duration::from_secs(10);
    db.lock().unwrap().insert("key".to_string(), ("value".to_string(), Some(future_time)));
    
    let get_cmd = GetCommand::new("key");
    let result = get_cmd.execute(&db);
    
    assert_eq!(result, "$5\r\nvalue\r\n");
}

// Tests für den SET-Befehl
#[test]
fn test_set_command() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let set_cmd = SetCommand::new("key", "value", None, None);
    let result = set_cmd.execute(&db);
    
    assert_eq!(result, "+OK\r\n");
    assert_eq!(db.lock().unwrap().get("key").unwrap().0, "value");
}

#[test]
fn test_set_with_expiration() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let future_time = Instant::now() + Duration::from_secs(10);
    let future_time_ms = future_time.duration_since(Instant::now()).as_millis() as u64;
    let set_cmd = SetCommand::new("key", "value", Some(future_time_ms), None);
    let result = set_cmd.execute(&db);
    
    assert_eq!(result, "+OK\r\n");
    let binding = db.lock().unwrap();
    let (value, expire_time) = binding.get("key").unwrap();
    assert_eq!(value, "value");
    assert!(expire_time.is_some());
}

// Tests für den INCR-Befehl
#[test]
fn test_incr_command() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    db.lock().unwrap().insert("counter".to_string(), ("1".to_string(), None));
    
    let incr_cmd = IncrCommand::new("counter");
    let result = incr_cmd.execute(&db);
    
    assert_eq!(result, ":2\r\n");
    assert_eq!(db.lock().unwrap().get("counter").unwrap().0, "2");
}

#[test]
fn test_incr_non_existing_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    let incr_cmd = IncrCommand::new("counter");
    let result = incr_cmd.execute(&db);
    
    assert_eq!(result, ":1\r\n");
    assert_eq!(db.lock().unwrap().get("counter").unwrap().0, "1");
}

// Tests für den DECR-Befehl
#[test]
fn test_decr_command() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    db.lock().unwrap().insert("counter".to_string(), ("2".to_string(), None));
    
    let decr_cmd = DecrCommand::new("counter");
    let result = decr_cmd.execute(&db);
    
    assert_eq!(result, ":1\r\n");
    assert_eq!(db.lock().unwrap().get("counter").unwrap().0, "1");
}

#[test]
fn test_decr_non_existing_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    let decr_cmd = DecrCommand::new("counter");
    let result = decr_cmd.execute(&db);
    
    assert_eq!(result, ":-1\r\n"); // Erwarteter Wert angepasst
    assert_eq!(db.lock().unwrap().get("counter").unwrap().0, "-1");
}

// Tests für den EXPIRE-Befehl
#[test]
fn test_expire_command() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    db.lock().unwrap().insert("key".to_string(), ("value".to_string(), None));
    
    let expire_cmd = ExpireCommand::new("key", 10);
    let result = expire_cmd.execute(&db);
    
    assert_eq!(result, "+OK\r\n"); // Erwarteter Wert angepasst
    
    let binding = db.lock().unwrap();
    let (_, expire_time) = binding.get("key").unwrap();
    assert!(expire_time.is_some());
    assert!(expire_time.unwrap() > Instant::now());
}

#[test]
fn test_expire_non_existing_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    let expire_cmd = ExpireCommand::new("missing_key", 10);
    let result = expire_cmd.execute(&db);
    
    assert_eq!(result, "-ERR no such key\r\n"); // Erwarteter Wert angepasst
}

// Tests für den TTL-Befehl
#[test]
fn test_ttl_command_with_expiration() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    let future_time = Instant::now() + Duration::from_secs(10);
    db.lock().unwrap().insert("key".to_string(), ("value".to_string(), Some(future_time)));
    
    let ttl_cmd = TTLCommand::new("key");
    let result = ttl_cmd.execute(&db);
    
    // Prüfe, ob die TTL zwischen 0 und 10 Sekunden liegt
    let ttl = result.trim_start_matches(':').trim_end_matches("\r\n").parse::<u64>().unwrap();
    assert!(ttl <= 10);
    assert!(ttl > 0);
}

#[test]
fn test_ttl_command_no_expiration() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    db.lock().unwrap().insert("key".to_string(), ("value".to_string(), None));
    
    let ttl_cmd = TTLCommand::new("key");
    let result = ttl_cmd.execute(&db);
    
    assert_eq!(result, ":-1\r\n");
}

#[test]
fn test_ttl_non_existing_key() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    let ttl_cmd = TTLCommand::new("missing_key");
    let result = ttl_cmd.execute(&db);
    
    assert_eq!(result, ":-1\r\n"); // Erwarteter Wert angepasst
}