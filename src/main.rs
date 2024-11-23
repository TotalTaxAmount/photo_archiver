mod user;

use std::env::{set_var, var};

use archive_config::CONFIG;
use archive_database::{database::Database, structs::User};
use log::{info, trace};
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

  let mut database = Database::new(CONFIG.database.clone());
  let user_manager = UserManager::new(http_server.clone(), database.clone());
  user_manager.lock().await.init().await;
  let _ = database.lock().await.init().await;
  http_server.register_method(user_manager.clone()).await;
  database.lock().await.new_user(User::new("foobar", "unique_hash")).await.unwrap();
  let users = database.lock().await.get_users().await;
  info!("{:?}", users);

  let http_server_clone = http_server.clone();

  tokio::spawn(async move {
    let s = http_server.clone();
    s.start().await
  });

  loop {
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
