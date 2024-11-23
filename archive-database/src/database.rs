use std::{
  any::Any, borrow::Borrow, f32::consts::E, fmt::Display, process::exit, sync::Arc, time::Duration,
};

use archive_config::{DatabaseConfig, CONFIG};
use log::{debug, error, info};
use tokio::{sync::Mutex, time::timeout};
use tokio_postgres::{row, types::Field, Client, NoTls, Row};

use crate::structs::{DatabaseError, User, UserFields, UserWrapper};

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

  /// Get a  Vec of all the users in the database
  ///
  /// Returns Vec<UserWrapper> if getting users was successful or a DatabaseError if it was not
  pub async fn get_all_users(&self) -> Result<Vec<UserWrapper>, DatabaseError> {
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

    let mut res: Vec<UserWrapper> = Vec::new();

    for row in rows {
      res.insert(res.len(), row.try_into()?);
    }

    Ok(res)
  }

  pub async fn get_user_by<V>(
    &self,
    field: UserFields,
    value: V,
  ) -> Result<UserWrapper, DatabaseError>
  where
    V: ToString,
  {
    if self.client.is_none() {
      error!("Database is not initialized");
      return Err(DatabaseError::new("Database not initialized"));
    }

    let users = self.get_all_users().await?;

    let value_string = value.to_string();

    let u = users.iter().find(|user| match field {
      UserFields::Id => value_string.parse::<i32>() == Ok(user.get_id()),
      UserFields::Username => value_string == user.get_inner_user().borrow().get_username(),
      UserFields::PasswordHash => {
        value_string == user.get_inner_user().borrow().get_password_hash()
      }
      UserFields::CreatedAt => value_string.parse::<i64>() == Ok(user.get_created_at()),
    });

    match u {
      Some(u) => UserWrapper::try_from(u.clone()).map_err(|e| DatabaseError::new(e)),
      None => {
        let err_msg = format!(
          "Failed to get user by {:?} with value {}",
          field,
          value.to_string()
        );
        error!("{}", err_msg);
        Err(DatabaseError::new(err_msg))
      }
    }
  }

  /// Updates an existing user
  ///
  /// Returns Ok(()) if the user was successfully modified or a DatabaseError if the operation failed
  pub async fn update_user(
    &self,
    id: i32,
    username: String,
    password_hash: String,
  ) -> Result<u64, DatabaseError> {
    // TODO: We need to get the user id so we need full wrapper
    if self.client.is_none() {
      error!("Database is not initialized");
      return Err(DatabaseError::new("Database not initialized"));
    }

    let c = self.client.as_ref().unwrap();

    let res = c
      .execute(
        "UPDATE USERS SET username=$1, password_hash=$2 WHERE id = $3",
        &[&username, &password_hash, &id],
      )
      .await
      .map_err(|e| DatabaseError::new(e));

    res
  }

  /// Creates an new user
  ///
  /// Returns Ok(()) if the user was successfully created or a DatabaseError if the operation failed
  pub async fn new_user(&self, user: User) -> Result<u64, DatabaseError> {
    if self.client.is_none() {
      error!("Database is not initialized");
      return Err(DatabaseError::new("Database not initialized"));
    }
    let c = self.client.as_ref().unwrap();

    let res = c
      .execute(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        &[&user.get_username(), &user.get_password_hash()],
      )
      .await
      .map_err(|e| {
        if e
          .as_db_error()
          .map_or(false, |db_error| db_error.code().code() == "23505")
        {
          DatabaseError::new("User already exists")
        } else {
          DatabaseError::new(e)
        }
      });

    res
  }

  /// Delate an existing user
  ///
  /// Returns Ok(()) if the user was deleted successfully or a DatabaseError if the operation failed
  pub async fn delate_user(&self, user_id: i32) -> Result<u64, DatabaseError> {
    if self.client.is_none() {
      error!("Database is not initialized");
      return Err(DatabaseError::new("Database not initialized"));
    }

    let c = self.client.as_ref().unwrap();

    let res = c
      .execute("DELATE FROM users WHERE id = $1", &[&user_id])
      .await
      .map_err(|e| DatabaseError::new(e));
    res
  }
}
