use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use webrs::{api::ApiMethod, request::Request, response::Response};

use crate::user::user_manager::{self, SharedUserManager};

pub type SharedPhotoManager = Arc<Mutex<PhotoManager>>;

pub struct PhotoManager {
  user_manager: SharedUserManager,
}

impl PhotoManager {
  pub fn new(user_manager: SharedUserManager) -> SharedPhotoManager {
    Arc::new(Mutex::new(Self { user_manager }))
  }
}

#[async_trait]
impl ApiMethod for PhotoManager {
  fn get_endpoint(&self) -> &str {
    "/photos"
  }

  async fn handle_get<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> 
  where
    'r: 's,
  {
    let id = match __self.user_manager.lock().await.validate_request(&req).await {
        Ok(id) => id,
        Err(e) => return Some(Response::basic(401, "Unauthorized")),
    };
    None
  }

  async fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> 
  where 
    'r: 's,
  {
    let id = match __self.user_manager.lock().await.validate_request(&req).await {
      Ok(id) => id,
      Err(e) => return Some(Response::basic(401, "Unauthorized")),
    };
    None
  }
}
