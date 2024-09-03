use tokio_postgres::{Client, NoTls, Error};

pub struct DbConnection {
    pub client: Client,
}

impl DbConnection {
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        dbname: &str,
    ) -> Result<Self, Error> {
        let connection_string = format!(
            "host={} port={} user={} password={} dbname={}",
            host, port, user, password, dbname
        );

        let (client, connection) =
            tokio_postgres::connect(&connection_string, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(DbConnection { client })
    }

    pub async fn query_users(&self) -> Result<Vec<(String, String, String)>, Error> {
        let rows = self.client.query("SELECT username, password, role FROM users", &[]).await?;
        let mut users = Vec::new();

        for row in rows {
            let username: String = row.get(0);
            let password: String = row.get(1);
            let role: String = row.get(2);
            users.push((username, password, role));
        }

        Ok(users)
    }

    pub async fn ping(&self) -> Result<(), Error> {
        self.client.execute("SELECT 1", &[]).await?;
        Ok(())
    }
}