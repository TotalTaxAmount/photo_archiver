use std::{
  borrow::BorrowMut,
  error::Error,
  fs::read_to_string,
  path::Path,
  str::FromStr,
  sync::{Arc, Mutex},
};

use archive_config::CONFIG;
use log::info;
use oauth2::{
  basic::BasicClient, reqwest::async_http_client, url::Url, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
  CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;
use webrs::{request::Request, response::Response};

use crate::user::user_manager::UserManagerError;

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

#[derive(Clone)]
pub struct OAuthFlow {
  user_id: i32,
  oauth_client: BasicClient,
  pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
}

impl OAuthFlow {
  pub fn new(user_id: i32) -> Result<Self, Box<dyn Error>> {
    let oauth_params = OAuthParameters::parse(&CONFIG.server.client_secret_path)?;
    let oauth_client = BasicClient::new(
      ClientId::new(oauth_params.client_id),
      Some(ClientSecret::new(oauth_params.client_secret)),
      AuthUrl::from_url(Url::from_str(&oauth_params.auth_uri).unwrap()),
      Some(TokenUrl::from_url(Url::from_str(&oauth_params.token_uri).unwrap())),
    )
    .set_redirect_uri(
      RedirectUrl::new(format!("http://localhost:{}/api/users/oauth/callback", CONFIG.server.port)).unwrap(),
    );

    Ok(Self { user_id, oauth_client, pkce_verifier: Arc::new(Mutex::new(None)) })
  }


  #[inline]
  pub fn get_user_id(&self) -> i32 {
    self.user_id
  }

  pub fn generate_auth_url(&mut self) -> (String, String) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let state: String = rand::thread_rng().sample_iter(&Alphanumeric).take(32).map(char::from).collect();

    let csrf_token = CsrfToken::new(state.clone());

    let auth_url = self
      .oauth_client
      .authorize_url(|| csrf_token)
      .add_scope(Scope::new("https://www.googleapis.com/auth/photoslibrary.readonly".to_string()))
      .set_pkce_challenge(pkce_challenge)
      .url();

    if let Ok(mut v) = self.pkce_verifier.lock() {
      *v = Some(pkce_verifier);
    }

    (auth_url.0.to_string(), state)
  }

  pub async fn process(&mut self, code: String) -> Result<String, Box<dyn Error>> {
    let auth_code = AuthorizationCode::new(code);

    let pkce_verifier = match self.pkce_verifier.lock().unwrap().take() {
      Some(v) => v,
      None => return Err(Box::new(UserManagerError::AuthenticationError("Internal Server Error".to_owned()))),
    };

    let token_res = self
      .oauth_client
      .exchange_code(auth_code)
      .set_pkce_verifier(pkce_verifier)
      .request_async(async_http_client)
      .await?;

    let access_token = token_res.access_token().secret().to_string();
    let hidden = {
      let (f, l) = access_token.split_at(4);
      format!("{}{}", f, "*".repeat(l.len()))
    };
    info!("Access token: {}", hidden);
    Ok(access_token)
  }
}
