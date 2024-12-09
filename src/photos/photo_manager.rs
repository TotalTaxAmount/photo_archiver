use std::sync::Arc;

use archive_config::CONFIG;
use async_trait::async_trait;
use gphotos_downloader::DownloaderPool;
use log::trace;
use tokio::sync::Mutex;
use webrs::{api::ApiMethod, request::Request, response::Response};

use crate::user::user_manager::{self, SharedUserManager};

pub type SharedPhotoManager = Arc<Mutex<PhotoManager>>;

pub struct PhotoManager {
  user_manager: SharedUserManager,
  pool: Arc<DownloaderPool>
}

impl PhotoManager {
  pub fn new(user_manager: SharedUserManager) -> SharedPhotoManager {
    Arc::new(Mutex::new(Self { 
      user_manager,
      pool: DownloaderPool::new(CONFIG.downloader.pool_size)
    }))
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
    let id = match self.user_manager.lock().await.validate_request(&req).await {
        Ok(id) => id,
        Err(_) => return Some(Response::basic(401, "Unauthorized")),
    };

    Some(Response::basic(200, &txt))
  }

  async fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> 
  where 
    'r: 's,
  {
    let id = match self.user_manager.lock().await.validate_request(&req).await {
      Ok(id) => id,
      Err(_) => return Some(Response::basic(401, "Unauthorized")),
    };
    None
  }
}
