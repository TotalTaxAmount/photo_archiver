use std::{error::Error, fs::read_to_string, path::Path};

use serde::Deserialize;

pub mod oauth_api;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct OAuthParameters {
  client_id: String,
  project_id: String,
  auth_uri: String,
  token_uri: String,
  auth_provider_x509_cert_url: String,
  client_secret: String,
  redirect_uris: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct OAuthWrapper {
  installed: OAuthParameters,
}

impl OAuthParameters {
  #[inline]
  pub fn parse<P: AsRef<Path>>(creds_file_path: P) -> Result<Self, Box<dyn Error>> {
    let creds_contents = read_to_string(creds_file_path)?;
    let oauth: OAuthWrapper = serde_json::from_str(&creds_contents)?;

    Ok(oauth.installed)
  }
}
