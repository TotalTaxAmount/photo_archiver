use std::{cell::RefCell, collections::HashMap, fmt::format, process::exit, sync::Arc};

use archive_config::CONFIG;
use archive_database::{database::SharedDatabase, structs::User};
use async_trait::async_trait;
use jwt::{token::Signed, Claims, Header, Token};
use log::{debug, error};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use webrs::{api::ApiMethod, request::Request, response::Response, server::WebrsHttp};

use crate::user::oauth::{oauth_api::OAuthMethod, OAuthParameters};

pub type SharedUserManager = Arc<Mutex<UserManager>>;

#[derive(Clone)]
pub struct UserManager {
  database: SharedDatabase,
  http_server: Arc<WebrsHttp>,
  oauth: Arc<Mutex<OAuthMethod>>,
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
    }))
  }

  pub async fn init(&self) {
    self.http_server.register_method(self.oauth.clone()).await;
  }

  pub fn get_oauth(&self) -> Arc<Mutex<OAuthMethod>> {
    self.oauth.clone()
  }

  pub async fn generate_session_token(&self, user: RefCell<User>) -> Token<Header, Claims, Signed> {
    todo!()
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

  async fn handle_new_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    let json: Value = match serde_json::from_slice(&req.get_data()) {
        Ok(j) => j,
        Err(_) => todo!(),
    };

    let username: &str = json["username"].as_str()?;
    let password: &str = json["password"].as_str()?;
    match self.database.lock().await.new_user(User::new(username, &Self::hash_password(password))).await {
      Ok(r) => {
        debug!("Modified {} rows", r);
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
              "error": format!("Failed to add user to database: {}", e)
            }),
          )
          .unwrap(),
        )
      }
    }
  }

  async fn handle_user_login<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    let req_user = match serde_json::from_slice::<User>(&req.get_data()) {
      Ok(u) => u,
      Err(e) => todo!(),
    };

    if let Ok(u) = {
      let username = req_user.get_username();
      self
        .database
        .lock()
        .await
        .get_user_by(archive_database::structs::UserFields::Username, &username)
        .await
    } {
      let u = u.get_inner_user();
      if req_user.get_password_hash() == u.borrow().get_password_hash() {
        let session_token = self.generate_session_token(u.clone()).await;
        let session_token_string = session_token.as_str();
        u.borrow_mut().set_session_token(session_token_string);

        return Some(
          Response::from_json(
            200,
            json!({
              "token": session_token_string
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
      Some(_) | None => {
        return Some(Response::basic(404, "Not Found"));
      }
    }
  }
}