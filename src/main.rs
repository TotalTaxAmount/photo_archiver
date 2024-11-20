mod oauth;

use std::{
  env::{set_var, var},
  process::exit,
  sync::Arc,
  time::Duration,
};

use gphotos_downloader::GPhotosDownloader;
use lazy_static::lazy_static;
use log::{error, info};
use oauth::{oauth::OAuth, OAuthParameters};
use photo_archiver::config::Config;
use serde_json::error;
use tokio::sync::Mutex;
use webrs::server::WebrsHttp;

lazy_static! {
  pub static ref CONFIG: Config = Config::parse(var("CONFIG_PATH").unwrap());
}

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
  let oauth_method = OAuth::new(oauth_params);

  let http_server = WebrsHttp::new(
    vec![Arc::new(Mutex::new(oauth_method))],
    CONFIG.server.port,
    (
      CONFIG.server.compression.zstd,
      CONFIG.server.compression.br,
      CONFIG.server.compression.gzip,
    ),
    CONFIG.server.content_dir.clone(),
  );

  http_server.start().await?;
  Ok(())
}
