use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::io::{Read, Write};
use std::net::{TcpStream, TcpListener};

mod cmd;

fn parse_resp_bulk_string(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let parts: Vec<&str> = input.split("\r\n").collect();

    if parts.is_empty() {
        return args;
    }

    if parts[0].starts_with('*') {
        let count = parts[0][1..].parse::<usize>().unwrap_or(0);
        for i in 0..count {
            if i * 2 + 1 < parts.len() {
                if parts[i * 2 + 1].starts_with('$') {
                    let len = parts[i * 2 + 1][1..].parse::<usize>().unwrap_or(0);
                    if i * 2 + 2 < parts.len() && parts[i * 2 + 2].len() == len {
                        args.push(parts[i * 2 + 2].to_string());
                    }
                }
            }
        }
    }
    args
}

fn handle_client(mut stream: TcpStream, db: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>) {
    let mut buffer = [0; 1024];
    
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => return, // Verbindung geschlossen
            Ok(n) => n,
            Err(_) => return, // Fehler beim Lesen
        };

        let input = String::from_utf8_lossy(&buffer[..bytes_read]);

        let args = parse_resp_bulk_string(&input);

        if args.is_empty() {
            let _ = stream.write_all(b"-ERR no command received\r\n");
            continue;
        }

        let response = match args.get(0).map(|s| s.to_uppercase()) {
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
                cmd::set::SetCommand::new(key, value, expire_seconds, expire_milliseconds).execute(&db)
            }
            Some(command) if command == "GET" && args.len() == 2 => {
                println!("Executing GET with key: '{}'", args[1]);
                cmd::get::GetCommand::new(&args[1]).execute(&db)
            }
            Some(command) if command == "EXPIRE" && args.len() == 3 => {
                if let Ok(seconds) = args[2].parse::<u64>() {
                    println!("Executing EXPIRE with key: '{}' and seconds: '{}'", args[1], seconds);
                    cmd::expire::ExpireCommand::new(&args[1], seconds).execute(&db)
                } else {
                    "-ERR invalid expire time\r\n".to_string()
                }
            }
            Some(command) if command == "TTL" && args.len() == 2 => {
                println!("Executing TTL with key: '{}'", args[1]);
                cmd::ttl::TTLCommand::new(&args[1]).execute(&db)
            }
            _ => "-ERR unknown command or wrong number of arguments\r\n".to_string(),
        };

        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    println!("Server is running on 127.0.0.1:6379");

    let db: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let db = Arc::clone(&db);

        thread::spawn(move || {
            handle_client(stream, db);
        });
    }

    Ok(())
}