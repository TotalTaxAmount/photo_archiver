pub mod error;
pub mod structs;

use std::{collections::VecDeque, sync::Arc, task};

use error::DownloaderError;
use log::trace;
use reqwest::{Client, Response};
use serde_json::{from_str, to_string, Value};
use structs::{DownloaderGuard, MediaItemsResponse};
use tokio::sync::{
  oneshot::{channel, Sender},
  Mutex, OwnedSemaphorePermit, Semaphore,
};
use uid::IdU8;

pub struct DownloaderPool {
  pool: Mutex<VecDeque<Downloader>>,
  semaphore: Arc<Semaphore>,
  pending_tasks: Mutex<VecDeque<DownloadTask>>,
}

type DownloadTask = Sender<Result<DownloaderGuard, DownloaderError>>;

impl DownloaderPool {
  pub fn new(pool_size: usize) -> Arc<Self> {
    let mut downloaders = VecDeque::with_capacity(pool_size);
    for _ in 0..pool_size {
      downloaders.push_back(Downloader::new());
    }
    Arc::new(Self {
      pool: Mutex::new(downloaders),
      semaphore: Arc::new(Semaphore::new(pool_size)),
      pending_tasks: Mutex::new(VecDeque::new()),
    })
  }

  pub async fn acquire(self: Arc<Self>) -> Result<DownloaderGuard, DownloaderError> {
    let permit = self.semaphore.clone().try_acquire_owned();

    if let Ok(p) = permit {
      let downloader = {
        let mut pool = self.pool.lock().await;
        pool.pop_front()
      };

      if downloader.is_some() {
        return Ok(DownloaderGuard { downloader, pool: self.clone(), _permit: p });
      }
    }

    let (tx, rx) = channel();
    {
      let mut pending_tasks = self.pending_tasks.lock().await;
      pending_tasks.push_back(tx);
    }

    rx.await.map_err(|_| DownloaderError::PoolError("Task queue cancelled".to_owned()))?
  }

  async fn return_to_pool(self: Arc<Self>, downloader: Downloader) {
    let mut pool = self.pool.lock().await;
    pool.push_back(downloader);

    let mut pending = self.pending_tasks.lock().await;
    if let Some(task) = pending.pop_front() {
      let p = self.semaphore.clone().acquire_owned().await.unwrap();
      let downloader = pool.pop_front().unwrap();
      let _ = task.send(Ok(DownloaderGuard { downloader: Some(downloader), pool: self.clone(), _permit: p }));
    }
  }
}

#[derive(Debug, Clone)]
pub struct Downloader {
  access_token: Option<String>,
  id: IdU8<Self>,
}

impl Downloader {
  pub fn new() -> Self {
    Self { access_token: None, id: IdU8::<Self>::new() }
  }

  pub fn set_token<S: ToString>(&mut self, token: S) {
    self.access_token = Some(token.to_string());
  }

  pub fn get_token(&self) -> Option<String> {
    self.access_token.clone()
  }

  pub fn get_id(&self) -> u8 {
    self.id.clone().get()
  }

  pub async fn list_photos(&self, next_page_token: Option<String>) -> Result<MediaItemsResponse, DownloaderError> {
    if self.access_token.is_none() {
      todo!()
    }
    let token = self.access_token.as_ref().unwrap();

    let client = Client::new();
    let res = client
      .get("https://photoslibrary.googleapis.com/v1/mediaItems")
      .bearer_auth(token)
      .send()
      .await
      .map_err(|e| DownloaderError::RequestError(e.to_string()))?;

    let text = res.text().await.unwrap();
    MediaItemsResponse::try_from(text)
      .map_err(|e| DownloaderError::ApiError(format!("Bad json from gAPI: {}", e.to_string())))
  }
}
