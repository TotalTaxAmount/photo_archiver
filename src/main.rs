mod oauth;

use std::{
  env::{set_var, var},
  process::exit,
  sync::Arc,
  time::Duration,
};

use archive_config::CONFIG;
use gphotos_downloader::GPhotosDownloader;
use log::{error, info};
use oauth::{oauth::OAuth, OAuthParameters};
use serde_json::error;
use tokio::sync::Mutex;
use webrs::server::WebrsHttp;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  if let Err(_) = var("LOGLEVEL") {
    set_var("LOGLEVEL", "info");
  }
  pretty_env_logger::formatted_timed_builder()
    .parse_env("LOGLEVEL")
    .format_timestamp_millis()
    .init();

  if let Err(_) = var("CONFIG_PATH") {
    set_var("CONFIG_PATH", "archive_config.toml");
  }

  let oauth_params = match OAuthParameters::parse(&CONFIG.server.client_secret_path) {
    Ok(o) => o,
    Err(e) => {
      error!("Failed to read secrets file: {}", e);
      exit(-1);
    }
  };
  let oauth_method = Arc::new(Mutex::new(OAuth::new(oauth_params)));

  let http_server = WebrsHttp::new(
    CONFIG.server.port,
    (
      CONFIG.server.compression.zstd,
      CONFIG.server.compression.br,
      CONFIG.server.compression.gzip,
    ),
    CONFIG.server.content_dir.clone(),
  );

  http_server
    .register_method(oauth_method.clone())
    .await;

  let http_server_clone = http_server.clone();
  let http_server_task = tokio::spawn(async move {
    let s = http_server.clone();
    s.start().await
  });

  
  loop {
    if let Some(t) = oauth_method.lock().await.get_access_code() {
      info!("Yessir: {}", t);
      break;
    }
  }

  http_server_clone.stop().await;
  let _ = http_server_task.await;

  info!("Shutting down...");
  Ok(())
}
