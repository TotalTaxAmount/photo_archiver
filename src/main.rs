mod photos;
mod user;

use std::{
  env::{set_var, var},
  process::exit,
};

use archive_config::CONFIG;
use archive_database::database::PhotoArchiverDatabase;
use log::{error, info};
use photos::photo_manager::{self, PhotoManager};
use user::user_manager::UserManager;
use webrs::server::WebrsHttp;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  if var("LOGLEVEL").is_err() {
    set_var("LOGLEVEL", "info");
  }

  if var("CONFIG_PATH").is_err() {
    set_var("CONFIG_PATH", "archive_config.toml");
  }

  pretty_env_logger::formatted_timed_builder().parse_env("LOGLEVEL").format_timestamp_millis().init();

  let http_server = WebrsHttp::new(
    CONFIG.server.port,
    (CONFIG.server.compression.zstd, CONFIG.server.compression.br, CONFIG.server.compression.gzip),
    CONFIG.server.content_dir.clone(),
  );

  let database = PhotoArchiverDatabase::new(CONFIG.database.clone());
  let user_manager = UserManager::new(http_server.clone(), database.clone());
  let photo_manager = PhotoManager::new(user_manager.clone());

  database.lock().await.init().await.unwrap_or_else(|e| {
    error!("Failed to initialize database: {}", e);
    exit(1)
  });

  http_server.register_method(user_manager.clone()).await;
  http_server.register_method(photo_manager.clone()).await;

  // let http_server_clone = http_server.clone();
  let _ = http_server.start().await;
  Ok(())
}
