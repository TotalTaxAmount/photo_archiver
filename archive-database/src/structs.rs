use tokio_postgres::Row;

#[derive(Debug)]
pub struct User {
  id: i32,
  username: String,
  password_hash: String,
  created_at: i64,
  gapi_token: Option<String>,
  session_token: Option<String>,
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

impl TryFrom<Row> for User {
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

    Ok(User {
      id,
      username,
      password_hash,
      created_at,
      gapi_token: None,
      session_token: None,
    })
  }
}
