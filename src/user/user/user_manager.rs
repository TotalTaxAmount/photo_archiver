use std::{collections::HashMap, process::exit, sync::Arc};

use archive_config::CONFIG;
use async_trait::async_trait;
use log::error;
use tokio::sync::Mutex;
use webrs::{
  api::ApiMethod, request::Request, response::Response, server::{WebrsHttp}
};

use crate::user::oauth::{oauth_api::OAuthMethod, OAuthParameters};

struct User {
  username: String,
  gapi_token: Option<String>,
  session_token: Option<String>,
}

pub struct UserManager {
  users: HashMap<String, User>,
  server: Arc<WebrsHttp>,
  oauth: Arc<Mutex<OAuthMethod>>,
}

impl UserManager {
  pub fn new(server: Arc<WebrsHttp>) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self {
      server,
      users: HashMap::new(),
      oauth: Arc::new(Mutex::new(OAuthMethod::new(
        OAuthParameters::parse(&CONFIG.server.client_secret_path).unwrap_or_else(|e| {
          error!("Failed to parse client secret: {} Exiting...", e);
          exit(1)
        }),
      ))),
    }))
  }

  pub async fn init(&self) {
    self.server.register_method(self.oauth.clone()).await;
  }

  pub fn get_oauth(&self) -> Arc<Mutex<OAuthMethod>> {
    self.oauth.clone()
  }

  async fn handle_new_user<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    None
  }

  async fn handle_user_login<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>> {
    None
  }
}

#[async_trait]
impl ApiMethod for UserManager {
  fn get_endpoint(&self) -> &str {
    "/users"
  }

  async fn handle_get<'s, 'r>(&'s mut self, _req: Request<'r>) -> Option<Response<'r>> 
  where 
    'r: 's
  {
    None
  }

  async fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>>
  where 
    'r: 's
  {
    match req.get_endpoint().rsplit("/").next() {
      Some(e) if e == "new" => return self.handle_new_user(req).await,
      Some(e) if e == "login" => return self.handle_user_login(req).await, 
      Some(_) | None => {
        return Some(Response::basic(404,"Not Found"));
      }
    }
  }
}
