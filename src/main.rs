pub mod cmd;
mod config;
mod db;
mod handler;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::net::TcpListener;
use tokio::net::TcpStream as TokioTcpStream;

use config::Settings;
use db::connection::DbConnection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().expect("Failed to load settings");
    let address = settings.server.address.unwrap_or("127.0.0.1".to_string());
    let port = settings.server.port.unwrap_or(6379);
    let address_listener = format!("{}:{}", address, port);

    // Initialisiere die Datenbankverbindung
    let db_conn = Arc::new(
        DbConnection::new(
            settings.database.host.as_deref().unwrap_or("localhost"),
            settings.database.port.unwrap_or(5432),
            settings.database.user.as_deref().unwrap_or("user"),
            settings.database.password.as_deref().unwrap_or("password"),
            settings.database.dbname.as_deref().unwrap_or("dbname"),
        ).await.expect("Failed to connect to the database"),
    );

    // Prüfen Sie die Verbindung
    match db_conn.ping().await {
        Ok(_) => println!("Database connection is active"),
        Err(e) => eprintln!("Failed to ping database: {}", e),
    }

    let listener = TcpListener::bind(address_listener.clone())?;
    println!("Server is running on {}", address_listener);

    let db: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let db = Arc::clone(&db);
        let db_conn = Arc::clone(&db_conn);

        tokio::spawn(async move {
            let stream = TokioTcpStream::from_std(stream).unwrap(); // Konvertieren von std::net::TcpStream zu tokio::net::TcpStream
            handler::handle_client(stream, db, db_conn).await;
        });
    }

    Ok(())
}