use std::sync::Arc;

use archive_config::CONFIG;
use async_trait::async_trait;
use gphotos_downloader::DownloaderPool;
use log::{error, trace};
use serde_json::json;
use tokio::sync::Mutex;
use webrs::{api::ApiMethod, request::Request, response::Response};

use crate::user::user_manager::{self, SharedUserManager};

pub type SharedPhotoManager = Arc<Mutex<PhotoManager>>;

pub struct PhotoManager {
  user_manager: SharedUserManager,
  pool: Arc<DownloaderPool>,
}

impl PhotoManager {
  pub fn new(user_manager: SharedUserManager) -> SharedPhotoManager {
    Arc::new(Mutex::new(Self { user_manager, pool: DownloaderPool::new(CONFIG.downloader.pool_size) }))
  }

  pub async fn handle_list_photos<'s, 'r>(&'s self, id: i32, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    let user_manager = self.user_manager.lock().await;
    let user = user_manager.get_active_users().get(&id).unwrap();

    if let Some(guser) = user.get_guser() {
      let token = guser.get_auth_token();
      let mut downloader_guard = self.pool.clone().acquire().await.unwrap();
      downloader_guard.get().set_token(token);
      let photos = downloader_guard.get().list_photos(None).await;
      trace!("{:?}", photos);
    } else {
      error!("User {} not logged into google", user.get_username());
      return Some(Response::from_json(401, json!({ "error": "User is not logged into google" })).unwrap());
    }

    None
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
    match req.get_endpoint().rsplit("photos/").next() {
      Some("list") => self.handle_list_photos(id, req).await,
      _ => return Some(Response::basic(404, "Not Found")),
    }
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
