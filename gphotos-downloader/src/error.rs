#[derive(Debug, Clone)]
pub enum DownloaderError {
  PoolError(String),
  RequestError(String)
}