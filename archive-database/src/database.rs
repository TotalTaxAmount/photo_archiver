use std::{process::exit, sync::Arc, time::Duration};

use archive_config::{DatabaseConfig, CONFIG};
use log::{debug, error, info};
use sea_orm::{
  ActiveModelTrait, ColumnTrait, ConnectOptions, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use serde::de::value::Error;
use tokio::sync::Mutex;

use crate::{
  entities::users,
  structs::{DatabaseError, User},
};

pub type SharedDatabase = Arc<Mutex<PhotoArchiverDatabase>>;

pub struct PhotoArchiverDatabase {
  config: DatabaseConfig,
  client: Option<DatabaseConnection>,
}

impl PhotoArchiverDatabase {
  pub fn new(config: DatabaseConfig) -> SharedDatabase {
    Arc::new(Mutex::new(Self { config, client: None }))
  }

  pub async fn init(&mut self) -> Result<(), Error> {
    debug!("Initializing database connection");
    let connection_string = format!(
      "postgres://{}:{}@{}:{}/{}",
      self.config.username, self.config.password, self.config.ip, self.config.port, self.config.dbname
    );

    let mut options = ConnectOptions::new(connection_string);
    options.connect_timeout(Duration::from_secs(CONFIG.database.timeout));

    let client = match sea_orm::Database::connect(options).await {
      Ok(r) => r,
      Err(e) => {
        error!("Failed to connect to database at {}:{}: {}", CONFIG.database.ip, CONFIG.database.port, e);
        exit(1)
      }
    };

    info!("Connected to database at {}:{}", self.config.ip, self.config.port);

    self.client = Some(client);

    Ok(())
  }

  /// Get a  Vec of all the users in the database
  ///
  /// Returns Vec<User> if getting users was successful or a DatabaseError if it
  /// was not
  pub async fn get_all_users(&self) -> Result<Vec<User>, DatabaseError> {
    if self.client.is_none() || !self.client.as_ref().unwrap().ping().await.is_ok() {
      error!("Database is not initialized or the connection is invalid");
      return Err(DatabaseError::new("Database is not initialized or the connection is invalid"));
    }

    let db = self.client.as_ref().unwrap();
    let models = users::Entity::find().all(db).await.unwrap();

    let res = models.iter().map(|m| m.clone().into()).collect();
    Ok(res)
  }

  pub async fn get_user_by<V>(&self, field: users::Column, value: V) -> Result<User, DatabaseError>
  where
    V: Into<sea_orm::Value>,
  {
    if self.client.is_none() || !self.client.as_ref().unwrap().ping().await.is_ok() {
      error!("Database is not initialized or the connection is invalid");
      return Err(DatabaseError::new("Database is not initialized or the connection is invalid"));
    }

    let db = self.client.as_ref().unwrap();
    let user = users::Entity::find().filter(field.eq(value.into())).one(db).await.map_err(|e| {
      error!("Error querying that database: {}", e);
      DatabaseError::new("Failed to query the database")
    })?;

    match user {
      Some(m) => Ok(m.into()),
      None => Err(DatabaseError::new("User not found")),
    }
  }

  /// Updates an existing user
  ///
  /// Returns Ok(()) if the user was successfully modified or a DatabaseError if
  /// the operation failed
  pub async fn update_user(&self, id: i32, username: String, password_hash: String) -> Result<(), DatabaseError> {
    if self.client.is_none() || !self.client.as_ref().unwrap().ping().await.is_ok() {
      error!("Database is not initialized or the connection is invalid");
      return Err(DatabaseError::new("Database is not initialized or the connection is invalid"));
    }

    let db = self.client.as_ref().unwrap();

    let user = users::Entity::find_by_id(id).one(db).await.map_err(|e| {
      error!("Failed to fetch user: {}", e);
      DatabaseError::new("Failed to fetch user")
    })?;

    let mut user_mut = match user {
      Some(u) => u.into_active_model(),
      None => return Err(DatabaseError::new("User not found")),
    };

    user_mut.username = Set(username);
    user_mut.password_hash = Set(password_hash);

    let _ = user_mut.update(db).await.map_err(|e| {
      error!("Failed to update user: {}", e);
      DatabaseError::new("Failed to update user")
    })?;

    Ok(())
  }

  /// Creates an new user
  ///
  /// Returns Ok(()) if the user was successfully created or a DatabaseError if
  /// the operation failed
  pub async fn new_user(&self, user: User) -> Result<(), DatabaseError> {
    if self.client.is_none() || !self.client.as_ref().unwrap().ping().await.is_ok() {
      error!("Database is not initialized or the connection is invalid");
      return Err(DatabaseError::new("Database is not initialized or the connection is invalid"));
    }
    let db = self.client.as_ref().unwrap();

    let new_user = users::ActiveModel {
      username: Set(user.get_username()),
      password_hash: Set(user.get_password_hash()),
      ..Default::default()
    };

    let _ = new_user.insert(db).await.map_err(|e| {
      error!("Error inserting new user: {}", e);
      return DatabaseError::new("Failed to insert new user");
    })?;

    Ok(())
  }

  /// Delate an existing user
  ///
  /// Returns Ok(()) if the user was deleted successfully or a DatabaseError if
  /// the operation failed
  pub async fn delate_user(&self, user_id: i32) -> Result<(), DatabaseError> {
    if self.client.is_none() || !self.client.as_ref().unwrap().ping().await.is_ok() {
      error!("Database is not initialized or the connection is invalid");
      return Err(DatabaseError::new("Database is not initialized or the connection is invalid"));
    }

    let db = self.client.as_ref().unwrap();

    let _ = users::Entity::delete_by_id(user_id).exec(db).await.map_err(|e| {
      error!("Failed to delete user: {}", e);
      DatabaseError::new("Failed to delete user")
    })?;

    Ok(())
  }
}
