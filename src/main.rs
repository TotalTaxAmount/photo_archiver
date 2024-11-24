mod user;

use std::{
  env::{set_var, var},
  process::exit,
};

use archive_config::CONFIG;
use archive_database::database::PhotoArchiverDatabase;
use log::{error, info};
use reqwest::Client;
use user::user::user_manager::UserManager;
use webrs::server::WebrsHttp;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  if var("LOGLEVEL").is_err() {
    set_var("LOGLEVEL", "info");
  }
  pretty_env_logger::formatted_timed_builder()
    .parse_env("LOGLEVEL")
    .format_timestamp_millis()
    .init();

  if var("CONFIG_PATH").is_err() {
    set_var("CONFIG_PATH", "archive_config.toml");
  }

  let http_server = WebrsHttp::new(
    CONFIG.server.port,
    (
      CONFIG.server.compression.zstd,
      CONFIG.server.compression.br,
      CONFIG.server.compression.gzip,
    ),
    CONFIG.server.content_dir.clone(),
  );

  let database = PhotoArchiverDatabase::new(CONFIG.database.clone());
  let user_manager = UserManager::new(http_server.clone(), database.clone());

  database.lock().await.init().await.unwrap_or_else(|e| {
    error!("Failed to initialize database: {}", e);
    exit(1)
  });
  user_manager.lock().await.init().await;
  http_server.register_method(user_manager.clone()).await;

  let http_server_clone = http_server.clone();

  tokio::spawn(async move {
    let s = http_server.clone();
    s.start().await
  });

  loop {
    println!("{:?}", user_manager.lock().await.get_active_users());
    if let Some(t) = user_manager
      .lock()
      .await
      .get_oauth()
      .lock()
      .await
      .get_access_code()
    {
      let client = Client::new();

      let res = client
        .get("https://photoslibrary.googleapis.com/v1/mediaItems")
        .bearer_auth(t)
        .send()
        .await
        .unwrap();

      if res.status().is_success() {
        let res_text = res.text().await.unwrap();
        info!("Items: {}", res_text);
      }

      break;
    }
  }

  http_server_clone.stop().await;
  info!("Shutting down...");
  Ok(())
}
