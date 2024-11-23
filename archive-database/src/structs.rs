use tokio_postgres::Row;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWrapper {
  id: i32,
  created_at: i64,
  user: User,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
  username: String,
  password_hash: String,
  gapi_token: Option<String>,
  session_token: Option<String>,
}

impl User {
  pub fn new<S>(username: S, password_hash: S) -> Self
  where 
    S: ToString
  {
    Self {
      username: username.to_string(),
      password_hash: password_hash.to_string(),
      gapi_token: None,
      session_token: None,
    }
  }

  pub fn get_username(&self) -> &str {
    &self.username
  }

  pub fn get_password_hash(&self) -> &str {
    &self.password_hash
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

  pub fn set_gapi_token<S>(&mut self, gapi_token: Option<S>)
  where
    S: ToString,
  {
    self.gapi_token = gapi_token.map(|s| s.to_string());
  }

  pub fn set_session_token<S>(&mut self, session_token: Option<S>)
  where
    S: ToString,
  {
    self.session_token = session_token.map(|s| s.to_string());
  }
}

#[derive(Debug)]
pub struct DatabaseError {
  message: String,
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
      user: User::new(username, password_hash),
    })
  }
}
