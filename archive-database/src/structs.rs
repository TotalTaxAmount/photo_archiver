use core::fmt;
use std::{cell::RefCell, path::Display};

use jwt::{token::Signed, Claims, Header, Token};
use serde::{de::value::Error, Deserialize, Serialize};
use tokio_postgres::{types::Type, Row};

#[derive(Debug)]
pub enum UserFields {
  Id,
  Username,
  PasswordHash,
  CreatedAt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWrapper {
  id: i32,
  created_at: i64,
  user: RefCell<User>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
  username: String,
  password_hash: String,
  gapi_token: Option<String>,
  session_token: Option<String>,
}

impl User {
  pub fn new<S>(username: S, password_hash: S) -> Self
  where
    S: ToString,
  {
    Self {
      username: username.to_string(),
      password_hash: password_hash.to_string(),
      gapi_token: None,
      session_token: None,
    }
  }

  pub fn get_username(&self) -> String {
    self.username.clone()
  }

  pub fn get_password_hash(&self) -> String {
    self.password_hash.clone()
  }

  pub fn get_gapi_token(&self) -> Option<String> {
    self.gapi_token.clone()
  }

  pub fn get_session_token(&self) -> Option<String> {
    self.session_token.clone()
  }

  pub fn set_username<S>(&mut self, new_username: S)
  where
    S: ToString,
  {
    self.username = new_username.to_string()
  }

  pub fn set_password_hash<S>(&mut self, new_password_hash: S)
  where
    S: ToString,
  {
    self.password_hash = new_password_hash.to_string()
  }

  pub fn set_gapi_token<S>(&mut self, gapi_token: S)
  where
    S: ToString,
  {
    self.gapi_token = Some(gapi_token.to_string())
  }

  pub fn set_session_token<S>(&mut self, session_token: S)
  where
    S: ToString,
  {
    self.session_token = Some(session_token.to_string())
  }
}

#[derive(Debug)]
pub struct DatabaseError {
  message: String,
}

impl fmt::Display for DatabaseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.get_message())
  }
}

impl DatabaseError {
  pub fn new<S>(message: S) -> Self
  where
    S: ToString,
  {
    Self {
      message: message.to_string(),
    }
  }

  pub fn get_message(&self) -> String {
    self.message.clone()
  }
}

impl UserWrapper {
  pub fn get_id(&self) -> i32 {
    self.id
  }

  pub fn get_created_at(&self) -> i64 {
    self.created_at
  }

  pub fn get_inner_user(&self) -> RefCell<User> {
    self.user.clone()
  }
}

impl TryFrom<Row> for UserWrapper {
  type Error = DatabaseError;

  fn try_from(value: Row) -> Result<Self, Self::Error> {
    let id = value
      .try_get("id")
      .map_err(|_| Self::Error::new("Failed to get 'id' from row"))?;
    let username: String = value
      .try_get("username")
      .map_err(|_| Self::Error::new("Failed to get 'username' from row"))?;
    let password_hash: String = value
      .try_get("password_hash")
      .map_err(|_| Self::Error::new("Failed to get 'hashed_password' from row"))?;
    let created_at = value
      .try_get("created_at")
      .map_err(|_| Self::Error::new("Failed to get 'created_at' from row"))?;

    Ok(UserWrapper {
      id,
      created_at,
      user: RefCell::new(User::new(username, password_hash)),
    })
  }
}
