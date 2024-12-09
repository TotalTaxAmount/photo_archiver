pub mod error;

use std::{collections::VecDeque, sync::Arc, task};

use error::DownloaderError;
use tokio::sync::{oneshot::{channel, Sender}, Mutex, OwnedSemaphorePermit, Semaphore};
use uid::IdU8;

pub struct DownloaderPool {
  pool: Mutex<VecDeque<Downloader>>,
  semaphore: Arc<Semaphore>,
  pending_tasks : Mutex<VecDeque<DownloadTask>>
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
      pending_tasks: Mutex::new(VecDeque::new())
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
        return Ok(DownloaderGuard {
          downloader,
          pool: self.clone(),
          _permit: p
        });
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
      let _ = task.send(Ok(DownloaderGuard {
        downloader: Some(downloader),
        pool: self.clone(),
        _permit: p
      }));
    }
  }
}

pub struct DownloaderGuard {
  downloader: Option<Downloader>,
  pool: Arc<DownloaderPool>,
  _permit: OwnedSemaphorePermit
}

impl DownloaderGuard {
  pub fn get(&mut self) -> &mut Downloader {
    self.downloader.as_mut().unwrap()
  }
}

impl Drop for DownloaderGuard {
  fn drop(&mut self) {
    if let Some(d) = self.downloader.take() {
      let pool = self.pool.clone();
      tokio::spawn(async move {
        pool.return_to_pool(d).await
      });
    }
  }
}

#[derive(Debug, Clone)]
pub struct Downloader {
  access_token: Option<String>,
  id: IdU8<Self>
}

impl Downloader {
  pub fn new() -> Self{
    Self { 
      access_token: None,
      id: IdU8::<Self>::new()
    }
  }

  pub fn set_token(&mut self, token: String) {
    self.access_token = Some(token);
  }

  pub fn get_token(&self) -> Option<String> {
    self.access_token.clone()
  } 

  pub fn get_id(&self) -> u8 {
    self.id.clone().get()
  }
} 

