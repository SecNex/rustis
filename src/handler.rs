use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::db::connection::DbConnection;
use crate::cmd::{set, get, expire, ttl, incr, decr, exists, json::{SetJsonCommand, GetJsonCommand, DelJsonCommand}};

type Db = Arc<Mutex<HashMap<String, DbValue>>>;
type DbValue = (String, Option<Instant>);

pub fn parse_resp_bulk_string(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let parts: Vec<&str> = input.split("\r\n").collect();

    if parts.is_empty() {
        return args;
    }

    if let Some(count_str) = parts.first().and_then(|s| s.strip_prefix('*')) {
        if let Ok(count) = count_str.parse::<usize>() {
            for i in 0..count {
                if let (Some(len_str), Some(value)) = (parts.get(i * 2 + 1), parts.get(i * 2 + 2)) {
                    if let Some(len_str) = len_str.strip_prefix('$') {
                        if let Ok(len) = len_str.parse::<usize>() {
                            if value.len() == len {
                                args.push(value.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    args
}

pub async fn handle_client(mut stream: TcpStream, db: Db, db_conn: Arc<DbConnection>) {
    let peer_addr = stream.peer_addr().unwrap();
    println!("New connection from {}", peer_addr);
    
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("Connection closed by {}", peer_addr);
                return;
            }
            Ok(n) => n,
            Err(_) => {
                println!("Error reading from connection {}", peer_addr);
                return;
            }
        };

        let input = String::from_utf8_lossy(&buffer[..bytes_read]);

        let args = parse_resp_bulk_string(&input);

        if args.is_empty() {
            let _ = stream.write_all(b"-ERR no command received\r\n").await;
            continue;
        }

        let response = match args.first().map(|s| s.to_uppercase()) {
            Some(command) if command == "SET" => {
                let key = &args[1];
                let value = &args[2];
                let mut expire_seconds = None;
                let mut expire_milliseconds = None;

                if args.len() > 3 {
                    let mut i = 3;
                    while i < args.len() {
                        match args[i].as_str() {
                            "EX" => {
                                if i + 1 < args.len() {
                                    expire_seconds = args[i + 1].parse::<u64>().ok();
                                    i += 2;
                                } else {
                                    break;
                                }
                            }
                            "PX" => {
                                if i + 1 < args.len() {
                                    expire_milliseconds = args[i + 1].parse::<u64>().ok();
                                    i += 2;
                                } else {
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                }

                println!("Executing SET with key: '{}' and value: '{}'", key, value);
                set::SetCommand::new(key, value, expire_seconds, expire_milliseconds).execute(&db)
            }
            Some(command) if command == "GET" && args.len() == 2 => {
                println!("Executing GET with key: '{}'", args[1]);
                get::GetCommand::new(&args[1]).execute(&db)
            }
            Some(command) if command == "EXPIRE" && args.len() == 3 => {
                if let Ok(seconds) = args[2].parse::<u64>() {
                    println!("Executing EXPIRE with key: '{}' and seconds: '{}'", args[1], seconds);
                    expire::ExpireCommand::new(&args[1], seconds).execute(&db)
                } else {
                    "-ERR invalid expire time\r\n".to_string()
                }
            }
            Some(command) if command == "TTL" && args.len() == 2 => {
                println!("Executing TTL with key: '{}'", args[1]);
                ttl::TTLCommand::new(&args[1]).execute(&db)
            }
            Some(command) if command == "INCR" && args.len() == 2 => {
                println!("Executing INCR with key: '{}'", args[1]);
                incr::IncrCommand::new(&args[1]).execute(&db)
            }
            Some(command) if command == "DECR" && args.len() == 2 => {
                println!("Executing DECR with key: '{}'", args[1]);
                decr::DecrCommand::new(&args[1]).execute(&db)
            }
            Some(command) if command == "EXISTS" => {
                println!("Executing EXISTS with keys: {:?}", &args[1..]);
                exists::ExistsCommand::new(args[1..].to_vec()).execute(&db)
            }
            Some(command) if command == "USERS" => {
                println!("Executing USERS command");
                match db_conn.query_users().await {
                    Ok(users) => users.into_iter()
                        .map(|(username, password, role)| format!("{}:{}:{}", username, password, role))
                        .collect::<Vec<String>>()
                        .join("\n"),
                    Err(_) => "-ERR failed to query users\r\n".to_string(),
                }
            }
            Some(command) if command == "JSON.SET" && args.len() == 4 => {
                println!("Executing JSON.SET command");
                let key = &args[1];
                let path = &args[2];
                let value = &args[3];
                let json_cmd = SetJsonCommand::new(key, path, value);
                json_cmd.execute(&db)
            }
            Some(command) if command == "JSON.GET" && args.len() >= 2 => {
                println!("Executing JSON.GET command");
                let key = &args[1];
                let paths: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
                let json_cmd = GetJsonCommand::new(key, &paths);
                json_cmd.execute(&db)
            }
            Some(command) if command == "JSON.DEL" && args.len() == 2 => {
                println!("Executing JSON.DEL command");
                let key = &args[1];
                let json_cmd = DelJsonCommand::new(key);
                json_cmd.execute(&db)
            }
            _ => "-ERR unknown command or wrong number of arguments\r\n".to_string(),
        };

        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;
    }
}