mod oauth;

use std::{
  env::{set_var, var},
  process::exit,
  sync::Arc,
  time::Duration,
};

use gphotos_downloader::GPhotosDownloader;
use log::{error, info};
use oauth::{oauth::OAuth, OAuthParameters};
use photo_archiver::Config;
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

  let config: Config = Config::parse(var("CONFIG_PATH").unwrap());

  let oauth_params = match OAuthParameters::parse(config.client_secret_path) {
    Ok(o) => o,
    Err(e) => {
      error!("Failed to read secrets file: {}", e);
      exit(-1);
    }
  };
  let oauth_method = OAuth::new(oauth_params);

  let (auth_url, pkce_verifier) = oauth_method.generate_auth_url();

  info!("Open this URL:\n{}", auth_url);

  *oauth_method.pkce_verifier.lock().unwrap() = Some(pkce_verifier);

  let http_server = WebrsHttp::new(
    vec![Arc::new(Mutex::new(oauth_method))],
    config.port,
    (
      config.compression.zstd,
      config.compression.br,
      config.compression.gzip,
    ),
    config.content_dir,
  );

  http_server.start().await?;
  Ok(())
}
