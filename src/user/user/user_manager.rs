use core::fmt;
use std::{
  borrow::BorrowMut,
  collections::{BTreeMap, HashMap},
  fmt::write,
  process::exit,
  sync::Arc,
};

use archive_config::CONFIG;
use archive_database::{database::SharedDatabase, entities::users, structs::User};
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use jwt::{claims, token::Signed, Claims, Header, SignWithKey, Token, VerifyWithKey};
use log::{debug, error};
use rand::Rng;
use serde_json::{json, Value};
use sha2::{digest::InvalidLength, Digest, Sha256};
use tokio::sync::Mutex;
use webrs::{api::ApiMethod, request::Request, response::Response, server::WebrsHttp};

use crate::user::oauth::{oauth_api::OAuthMethod, OAuthParameters};

pub type SharedUserManager = Arc<Mutex<UserManager>>;

#[derive(Debug)]
pub enum UserManagerError {
  TokenError(String),
}

impl fmt::Display for UserManagerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UserManagerError::TokenError(msg) => write!(f, "Token Error: {}", msg),
    }
  }
}

impl std::error::Error for UserManagerError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      UserManagerError::TokenError(_) => None,
    }
  }
}

#[derive(Clone)]
pub struct UserManager {
  database: SharedDatabase,
  http_server: Arc<WebrsHttp>,
  oauth: Arc<Mutex<OAuthMethod>>,
  active_users: HashMap<i32, User>,
}

impl UserManager {
  pub fn new(http_server: Arc<WebrsHttp>, database: SharedDatabase) -> SharedUserManager {
    Arc::new(Mutex::new(Self {
      http_server,
      database,
      oauth: Arc::new(Mutex::new(OAuthMethod::new(
        OAuthParameters::parse(&CONFIG.server.client_secret_path).unwrap_or_else(|e| {
          error!("Failed to parse client secret: {} Exiting...", e);
          exit(1)
        }),
      ))),
      active_users: HashMap::new(),
    }))
  }

  pub async fn init(&self) {
    self.http_server.register_method(self.oauth.clone()).await;
  }

  #[inline]
  pub fn get_oauth(&self) -> Arc<Mutex<OAuthMethod>> {
    self.oauth.clone()
  }

  #[inline]
  pub fn get_active_users(&self) -> &HashMap<i32, User> {
    &self.active_users
  }

  pub fn generate_session_token(
    user: User,
  ) -> Result<Token<Header, BTreeMap<String, String>, Signed>, UserManagerError> {
    let key: Hmac<Sha256> =
      Hmac::new_from_slice(&CONFIG.auth.jwt_secret.as_bytes()).map_err(|e| {
        error!("Failed to create HMAC key: {}", e);
        UserManagerError::TokenError(e.to_string())
      })?;

    let mut claims: BTreeMap<String, String> = BTreeMap::new();
    claims.insert("id".to_string(), user.get_id().to_string());
    claims.insert(
      "exp".to_string(),
      (chrono::Utc::now() + chrono::Duration::days(1))
        .timestamp()
        .to_string(),
    );

    let header = Header::default();

    Token::new(header, claims).sign_with_key(&key).map_err(|e| {
      error!("Error signing token with key: {}", e);
      UserManagerError::TokenError(e.to_string())
    })
  }

  pub fn hash_password<S: ToString>(password: S) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.to_string());
    hasher
      .finalize()
      .iter()
      .map(|b| format!("{:02x}", b))
      .collect::<String>()
  }

  fn validate_token(&self, token_str: &str) -> Result<bool, UserManagerError> {
    let key: Hmac<Sha256> =
      Hmac::new_from_slice(&CONFIG.auth.jwt_secret.as_bytes()).map_err(|e| {
        error!("Failed to create HMAC key: {}", e);
        UserManagerError::TokenError(e.to_string())
      })?;

    let token: Token<Header, BTreeMap<String, String>, _> =
      token_str.verify_with_key(&key).map_err(|e| {
        error!("Failed to verify token with key: {}", e);
        UserManagerError::TokenError(e.to_string())
      })?;

    let claims = token.claims();
    let id = claims.get("id").and_then(|v| v.parse::<i32>().ok());
    let exp = claims.get("exp").and_then(|v| v.parse::<i64>().ok());
    if let (Some(id), Some(exp)) = (id, exp) {
      if exp < chrono::Utc::now().timestamp() {
        return Ok(false);
      }
      match self.get_active_users().get(&id) {
        Some(u) if u.get_session_token() == Some(token_str.to_string()) => Ok(true),
        None | Some(_) => {
          error!("User is not active or token mismatch");
          Ok(false)
        }
      }
    } else {
      error!("Invalid JWT token");
      Err(UserManagerError::TokenError(
        "Invalid JWT Token".to_string(),
      ))
    }
  }

  async fn handle_new_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    let json: Value = match serde_json::from_slice(&req.get_data()) {
      Ok(j) => j,
      Err(e) => {
        error!("Failed to parse request json: {}", e);
        return Some(
          Response::from_json(
            400,
            json!({
              "error": "Failed to parse request json"
            }),
          )
          .unwrap(),
        );
      }
    };

    let username: &str = json["username"].as_str()?;
    let password: &str = json["password"].as_str()?;

    if username.len() < 3 {
      return Some(
        Response::from_json(
          400,
          json!({
            "error": "Username is too short"
          }),
        )
        .unwrap(),
      );
    }

    if password.len() < 8 {
      return Some(
        Response::from_json(
          400,
          json!({
            "error": "Password is too short"
          }),
        )
        .unwrap(),
      );
    }

    match self
      .database
      .lock()
      .await
      .new_user(User::new(username, &Self::hash_password(password)))
      .await
    {
      Ok(_) => {
        debug!("Added new user");
        Some(
          Response::from_json(
            200,
            json!({
              "success": "New user created successfully"
            }),
          )
          .unwrap(),
        )
      }
      Err(e) => {
        error!("Failed to add new user to database: {}", e);
        Some(
          Response::from_json(
            500,
            json!({
              "error": format!("User {} already exists", username)
            }),
          )
          .unwrap(),
        )
      }
    }
  }

  async fn handle_user_login<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> {
    let json: Value = match serde_json::from_slice(&req.get_data()) {
      Ok(j) => j,
      Err(e) => {
        error!("Failed to parse request json: {}", e);
        return Some(
          Response::from_json(
            400,
            json!({
              "error": "Failed to parse request json"
            }),
          )
          .unwrap(),
        );
      }
    };

    let username = json["username"].as_str()?;
    let password = json["password"].as_str()?;

    if let Ok(mut u) = {
      self
        .database
        .lock()
        .await
        .get_user_by(users::Column::Username, username)
        .await
    } {
      if Self::hash_password(password) == u.get_password_hash() {
        let session_token = Self::generate_session_token(u.clone());
        let session_token = session_token.unwrap();
        u.borrow_mut().set_session_token(session_token.as_str());

        self.active_users.insert(u.get_id(), u);
        return Some(
          Response::from_json(
            200,
            json!({
              "token": session_token.as_str()
            }),
          )
          .unwrap(),
        );
      } else {
        return Some(
          Response::from_json(
            401,
            json!({
              "error": "Invalid username or password"
            }),
          )
          .unwrap(),
        );
      }
    } else {
      return Some(
        Response::from_json(
          401,
          json!({
            "error": "Invalid username or password"
          }),
        )
        .unwrap(),
      );
    }
  }

  // These to need auth
  async fn handle_delete_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    todo!()
  }

  async fn handle_modify_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    todo!()
  }
}

#[async_trait]
impl ApiMethod for UserManager {
  fn get_endpoint(&self) -> &str {
    "/users"
  }

  async fn handle_get<'s, 'r>(&'s mut self, _req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    None
  }

  async fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    match req.get_endpoint().rsplit("/").next() {
      Some("new") => return self.handle_new_user(req).await,
      Some("delete") => return self.handle_delete_user(req).await,
      Some("modify") => return self.handle_modify_user(req).await,
      Some("login") => return self.handle_user_login(req).await,
      Some("validate") => {
        let json: Value = match serde_json::from_slice(&req.get_data()) {
          Ok(j) => j,
          Err(e) => {
            error!("Failed to parse request json: {}", e);
            return Some(Response::from_json(400, json!({ "error": "failed to parse request json" })).unwrap());
          }
        };

        let t = match json["token"].as_str() {
          Some(t) => t,
          None => {
            error!("No token in request");
            return Some(Response::from_json(400, json!({ "error": "No token in json" })).unwrap());
          }
        };

        match self.validate_token(t) {
          Ok(v) => {
            return if v {
              Some(Response::from_json(200, json!({"success": "Token is valid"})).unwrap())
            } else {
              Some(Response::from_json(401, json!({"error": "Token is invalid"})).unwrap())
            };
          }
          Err(e) => Some(Response::from_json(500, json!({ "error": format!("{}", e) })).unwrap()),
        }
      }
      Some(_) | None => {
        return Some(Response::basic(404, "Not Found"));
      }
    }
  }
}
