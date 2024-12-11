use core::fmt;

use serde::{Deserialize, Serialize};

use crate::entities::users;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
  model: users::Model,
  guser: Option<GUser>,
  session_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GUser {
  auth_token: String,
  name: String,
  pfp_url: String,
}

impl User {
  pub fn new<S: ToString>(username: S, password_hash: S) -> Self {
    Self {
      model: users::Model {
        username: username.to_string(),
        password_hash: password_hash.to_string(),
        id: -1,
        created_at: None,
      },
      guser: None,
      session_token: None,
    }
  }

  #[inline]
  pub fn get_username(&self) -> String {
    self.model.username.clone()
  }

  #[inline]
  pub fn get_password_hash(&self) -> String {
    self.model.password_hash.clone()
  }

  #[inline]
  pub fn get_id(&self) -> i32 {
    self.model.id
  }

  #[inline]
  pub fn get_created_at(&self) -> Option<i64> {
    self.model.created_at
  }

  #[inline]
  pub fn get_guser(&self) -> Option<GUser> {
    self.guser.clone()
  }

  #[inline]
  pub fn get_session_token(&self) -> Option<String> {
    self.session_token.clone()
  }

  #[inline]
  pub fn set_username<S: ToString>(&mut self, new_username: S) {
    self.model.username = new_username.to_string()
  }

  #[inline]
  pub fn set_password_hash<S: ToString>(&mut self, new_password_hash: S) {
    self.model.password_hash = new_password_hash.to_string()
  }

  #[inline]
  pub fn set_guser(&mut self, guser: GUser) {
    self.guser = Some(guser);
  }

  #[inline]
  pub fn set_session_token<S: ToString>(&mut self, session_token: S) {
    self.session_token = Some(session_token.to_string())
  }
}

impl GUser {
  pub fn new(auth_token: String, username: String, pfp_url: String) -> Self {
    Self { auth_token, name: username, pfp_url }
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }

  pub fn get_auth_token(&self) -> &str {
    &self.auth_token
  }

  pub fn get_pfp_url(&self) -> &str {
    &self.pfp_url
  }

  pub fn set_auth_token<S: ToString>(&mut self, auth_token: S) {
    self.auth_token = auth_token.to_string();
  }

  pub fn set_name<S: ToString>(&mut self, name: S) {
    self.name = name.to_string();
  }

  pub fn set_pfp_url<S: ToString>(&mut self, pfp_url: S) {
    self.pfp_url = pfp_url.to_string();
  }
}

impl From<users::Model> for User {
  fn from(value: users::Model) -> Self {
    Self { model: value, guser: None, session_token: None }
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
    Self { message: message.to_string() }
  }

  #[inline]
  pub fn get_message(&self) -> String {
    self.message.clone()
  }
}
