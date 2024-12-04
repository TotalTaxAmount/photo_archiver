use core::fmt;
use std::{
  borrow::BorrowMut,
  collections::{BTreeMap, HashMap},
  error::Error,
  sync::Arc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use archive_config::CONFIG;
use archive_database::{database::SharedDatabase, entities::users, structs::User};
use async_trait::async_trait;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use dashmap::DashMap;
use hmac::{Hmac, Mac};
use jwt::{token::Signed, Header, SignWithKey, Token, VerifyWithKey};
use log::{debug, error, trace};
use serde_json::{json, Value};
use sha2::Sha256;
use tokio::{sync::Mutex, time::{interval, sleep}};
use webrs::{api::ApiMethod, request::Request, response::Response, server::WebrsHttp};

use super::oauth::OAuthFlow;

pub type SharedUserManager = Arc<Mutex<UserManager>>;

const AUTH_HEADER: &str = "authorization";

#[derive(Debug)]
pub enum UserManagerError {
  TokenError(String),
  AuthenticationError(String),
}

impl fmt::Display for UserManagerError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Self::AuthenticationError(m) => write!(f, "Authentication Error: {}", m),
      Self::TokenError(m) => write!(f, "Token Error: {}", m),
    }
  }
}

impl Error for UserManagerError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::TokenError(_) | Self::AuthenticationError(_) => None,
    }
  }
}

impl UserManagerError {
  pub fn get_message(&self) -> String {
    match self {
      Self::AuthenticationError(m) | Self::TokenError(m) => m.clone(),
    }
  }
}

#[derive(Clone)]
pub struct UserManager {
  database: SharedDatabase,
  http_server: Arc<WebrsHttp>,
  active_users: DashMap<i32, User>,
  oauth_flows: HashMap<String, (i32, OAuthFlow, u64)>,
}

impl UserManager {
  pub fn new(http_server: Arc<WebrsHttp>, database: SharedDatabase) -> SharedUserManager {
    let user_manager =
      Arc::new(Mutex::new(Self { http_server, database, active_users: DashMap::new(), oauth_flows: HashMap::new() }));

    let cleanup = Arc::clone(&user_manager);
    tokio::spawn(async move {
      let max_time: u64 = 600;
      let mut interval = interval(Duration::from_secs(60));
      loop {
        interval.tick().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut user_manager = cleanup.lock().await;

        user_manager.oauth_flows.retain(|state, (id, _, timestamp)| {
          if now - *timestamp > max_time {
            trace!("Removing expired OAuth flow: state = {}, id = {}", state, id);
            false
          } else {
            true
          }
        });
      }
    });

    user_manager
  }

  pub async fn init(&self) {}

  #[inline]
  pub fn get_active_users(&self) -> &DashMap<i32, User> {
    &self.active_users
  }

  pub fn generate_session_token(
    user: User,
  ) -> Result<Token<Header, BTreeMap<String, String>, Signed>, UserManagerError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(&CONFIG.auth.jwt_secret.as_bytes()).map_err(|e| {
      error!("Failed to create HMAC key: {}", e);
      UserManagerError::TokenError(e.to_string())
    })?;

    let mut claims: BTreeMap<String, String> = BTreeMap::new();
    claims.insert("id".to_string(), user.get_id().to_string());
    claims.insert("exp".to_string(), (chrono::Utc::now() + chrono::Duration::days(1)).timestamp().to_string());

    let header = Header::default();

    Token::new(header, claims).sign_with_key(&key).map_err(|e| {
      error!("Error signing token with key: {}", e);
      UserManagerError::TokenError(e.to_string())
    })
  }

  fn hash_password<S: ToString>(password: S) -> String {
    hash(password.to_string(), DEFAULT_COST).unwrap_or_else(|_| "".to_string())
  }

  fn verify_password(password: &str, hashed_password: &str) -> Result<bool, BcryptError> {
    verify(password, hashed_password)
  }

  fn validate_token(&self, token_str: &str) -> Result<(bool, i32), UserManagerError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(&CONFIG.auth.jwt_secret.as_bytes()).map_err(|e| {
      error!("Failed to create HMAC key: {}", e);
      UserManagerError::TokenError(e.to_string())
    })?;

    let token: Token<Header, BTreeMap<String, String>, _> = token_str.verify_with_key(&key).map_err(|e| {
      error!("Failed to verify token with key: {}", e);
      UserManagerError::TokenError(e.to_string())
    })?;

    let claims = token.claims();
    let id = claims.get("id").and_then(|v| v.parse::<i32>().ok());
    let exp = claims.get("exp").and_then(|v| v.parse::<i64>().ok());
    if let (Some(id), Some(exp)) = (id, exp) {
      if exp < chrono::Utc::now().timestamp() {
        return Ok((false, id));
      }
      match self.get_active_users().get(&id) {
        Some(u) if u.get_session_token() == Some(token_str.to_string()) => Ok((true, id)),
        None | Some(_) => {
          error!("User is not active or token mismatch");
          Ok((false, id))
        }
      }
    } else {
      error!("Invalid JWT token");
      Err(UserManagerError::TokenError("Invalid JWT Token".to_string()))
    }
  }

  pub async fn validate_request<'s, 'r>(&'s self, req: &Request<'r>) -> Result<i32, UserManagerError> {
    let headers = req.get_headers();
    let auth_header =
      headers.get(AUTH_HEADER).ok_or(UserManagerError::AuthenticationError("No 'authorization' header".to_owned()))?;

    if !auth_header.starts_with("Bearer ") {
      return Err(UserManagerError::AuthenticationError("Invalid header format".to_owned()));
    }

    let (valid, id) = self.validate_token(&auth_header[7..])?;
    if !valid {
      return Err(UserManagerError::AuthenticationError("Invalid token".to_owned()));
    }

    Ok(id)
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

    match self.database.lock().await.new_user(User::new(username, &Self::hash_password(password))).await {
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

    if let Ok(mut u) = { self.database.lock().await.get_user_by(users::Column::Username, username).await } {
      if let Ok(_) = UserManager::verify_password(password, &u.get_password_hash()) {
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

  async fn handle_verify_token<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    match self.validate_request(&req).await {
      Ok(_) => {
        return Some(Response::from_json(200, json!({ "success": "Token is valid"})).unwrap());
      }
      Err(e) => {
        return Some(Response::from_json(401, json!({ "error": e.get_message() })).unwrap());
      }
    }
  }

  // These to need auth
  async fn handle_delete_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    todo!()
  }

  async fn handle_modify_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    todo!()
  }

  async fn handle_new_oauth_url<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> {
    let id = match self.validate_request(&req).await {
      Ok(id) => id,
      Err(e) => {
        return Some(Response::from_json(401, json!({ "error": format!("{}", e.get_message()) })).unwrap());
      }
    };

    let mut flow = match OAuthFlow::new(id) {
      Ok(f) => f,
      Err(e) =>
        return Some(
          Response::from_json(401, json!({ "error": format!("Failed to create new oauth flow: {}", e.to_string()) }))
            .unwrap(),
        ),
    };

    let (url, state) = flow.generate_auth_url();

    if self.active_users.get(&id).is_none() {
      return Some(Response::from_json(401, json!({ "error": "User is not logged in or does not exist"})).unwrap());
    }

    trace!("New OAuth flow added, state = {}, id = {}", state, id);
    let curr_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    self.oauth_flows.retain(|_, (i, _, _)| {
      if id != *i {
        return true;
      } else {
        trace!("User {} created new oauth flow, removing old", id);
        return false;
      }
    });
    self.oauth_flows.insert(state, (id, flow, curr_time));
    return Some(Response::from_json(200, json!({ "oauth_url": url })).unwrap());
  }

  async fn handle_oauth_callback<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> {
    let params = req.get_url_params();
    let state = if let Some(s) = params.get("state") {
      s
    } else {
      return Some(Response::from_json(400, json!({ "error": "No state param" })).unwrap());
    };
    let code = if let Some(s) = params.get("code") {
      s
    } else {
      return Some(Response::from_json(400, json!({ "error": "No code param" })).unwrap());
    };

    let (id, flow, _) = if let Some(f) = self.oauth_flows.get_mut(*state) {
      f
    } else {
      return Some(Response::from_json(401, json!({ "error": "No flow for state" })).unwrap());
    };

    let mut u = match self.active_users.get_mut(&id) {
      Some(u) => u,
      None => return Some(Response::from_json(401, json!({ "error": "User is not active" })).unwrap()),
    }; // Should always be active here

    if flow.get_user_id() != *id {
      return Some(Response::from_json(401, json!({ "error": "Invalid id" })).unwrap());
    }

    let mut res: Option<Response> = None;
    match flow.process(code.to_string()).await {
      Ok(t) => u.set_gapi_token(t),
      Err(e) => {
        res = Some(Response::from_json(501, json!({ "error": e.to_string() })).unwrap());
      }
    };

    trace!("Removed OAuth flow: state = {}, id = {}", &state, &id);
    self.oauth_flows.remove(*state);

    if res.is_none() {
      let mut temp = Response::basic(301, "Found");
      temp.add_header("location".to_string(), "/");
      res = Some(temp);
    }

    res
  }
}

#[async_trait]
impl ApiMethod for UserManager {
  fn get_endpoint(&self) -> &str {
    "/users"
  }

  async fn handle_get<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    match req.get_endpoint().rsplit("users/").next() {
      Some("validate") => self.handle_verify_token(req).await,
      Some("oauth/url") => self.handle_new_oauth_url(req).await,
      Some("oauth/callback") => self.handle_oauth_callback(req).await,
      _ => Some(Response::basic(404, "Not Found")),
    }
  }

  async fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    match req.get_endpoint().rsplit("/").next() {
      Some("new") => self.handle_new_user(req).await,
      Some("delete") => self.handle_delete_user(req).await,
      Some("modify") => self.handle_modify_user(req).await,
      Some("login") => self.handle_user_login(req).await,
      _ => Some(Response::basic(404, "Not Found")),
    }
  }
}
