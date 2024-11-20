mod oauth;

use std::{
  env::{set_var, var}, sync::Arc, time::Duration
};

use gphotos_downloader::GPhotosDownloader;
use log::info;
use oauth::{oauth::OAuth, OAuthParameters};
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

  let oauth = OAuthParameters::parse("secret.json").unwrap();
  let oauth_method = OAuth::new(oauth);

  let (auth_url, pkce_verifier) = oauth_method.generate_auth_url();

  info!("Open this URL:\n{}", auth_url);

  *oauth_method.pkce_verifier.lock().unwrap() = Some(pkce_verifier);


  let http_server = WebrsHttp::new(vec![Arc::new(Mutex::new(oauth_method))], 8080, (true, true, true), "html".to_string());

  http_server.start().await?;
  Ok(())
}
