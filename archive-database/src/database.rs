use std::{process::exit, sync::Arc, time::Duration};

use archive_config::{DatabaseConfig, CONFIG};
use log::{debug, error, info};
use tokio::{sync::Mutex, time::timeout};
use tokio_postgres::{Client, NoTls, Row};

use crate::structs::{DatabaseError, User};

pub type SharedDatabase = Arc<Mutex<Database>>;

pub struct Database {
  config: DatabaseConfig,
  client: Option<Client>,
}

impl Database {
  pub fn new(config: DatabaseConfig) -> SharedDatabase {
    Arc::new(Mutex::new(Self {
      config,
      client: None,
    }))
  }

  pub async fn init(&mut self) -> Result<(), tokio_postgres::Error> {
    debug!("Initializing database connection");
    let connection_string = format!(
      "hostaddr={} port={} user={} password={} dbname={}",
      self.config.ip,
      self.config.port,
      self.config.username,
      self.config.password,
      self.config.dbname
    );

    let res = timeout(
      Duration::from_secs(CONFIG.database.timeout.into()),
      tokio_postgres::connect(&connection_string, NoTls),
    )
    .await;

    let (client, connection) = match res {
      Ok(r) => r.unwrap_or_else(|e| {
        error!("Failed to connect to database: {}", e);
        exit(1)
      }),
      Err(_) => {
        error!("Connection to database timed out");
        exit(1)
      }
    };

    tokio::spawn(async move {
      if let Err(e) = connection.await {
        eprintln!("Connection error: {}", e);
      }
    });

    info!(
      "Connected to database at {}:{}",
      self.config.ip, self.config.port
    );

    self.client = Some(client);

    Ok(())
  }

  pub async fn get_users(&self) -> Result<Vec<User>, DatabaseError> {
    if self.client.is_none() {
      error!("Database is not initialized");
      return Err(DatabaseError::new("Database not initialized"));
    }

    let c = self.client.as_ref().unwrap();
    let rows: Vec<Row> = match c.query("SELECT * FROM USERS", &[]).await {
      Ok(r) => r,
      Err(e) => {
        error!("Failed to get users from database: {}", e);
        return Err(DatabaseError::new("Failed to get users from database"));
      }
    };

    let mut res: Vec<User> = Vec::new();

    for row in rows {
      res.insert(res.len(), row.try_into()?);
    }

    Ok(res)
  }
}
