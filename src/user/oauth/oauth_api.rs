use std::{
  str::FromStr,
  sync::{Arc, Mutex},
};

use archive_config::CONFIG;
use async_trait::async_trait;
use log::info;
use oauth2::{
  basic::BasicClient, reqwest::async_http_client, url::Url, AuthUrl, AuthorizationCode, ClientId,
  ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
  TokenUrl,
};
use serde_json::json;
use webrs::{api::ApiMethod, request::Request, response::Response};

use super::OAuthParameters;

pub struct OAuthMethod {
  oauth_client: BasicClient,
  access_token: Option<String>,
  pub pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
}

impl OAuthMethod {
  pub fn new(oauth_params: OAuthParameters) -> Self {
    let oauth_client = BasicClient::new(
      ClientId::new(oauth_params.client_id),
      Some(ClientSecret::new(oauth_params.client_secret)),
      AuthUrl::from_url(Url::from_str(&oauth_params.auth_uri).unwrap()),
      Some(TokenUrl::from_url(
        Url::from_str(&oauth_params.token_uri).unwrap(),
      )),
    )
    .set_redirect_uri(
      RedirectUrl::new(format!(
        "http://localhost:{}/api/oauth/callback",
        CONFIG.server.port
      ))
      .unwrap(),
    );

    Self {
      oauth_client,
      access_token: None,
      pkce_verifier: Arc::new(Mutex::new(None)),
    }
  }

  pub fn get_access_code(&self) -> Option<String> {
    self.access_token.clone()
  }

  pub fn generate_auth_url(&self) -> (String, PkceCodeVerifier) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let auth_url = self
      .oauth_client
      .authorize_url(CsrfToken::new_random)
      .add_scope(Scope::new(
        "https://www.googleapis.com/auth/photoslibrary.readonly".to_string(),
      ))
      .set_pkce_challenge(pkce_challenge)
      .url();

    (auth_url.0.to_string(), pkce_verifier)
  }

  async fn handle_callback<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>> {
    let query = req.get_url_params();
    let code = query.get("code")?.to_string();

    let auth_code = AuthorizationCode::new(code);

    let pkce_verifier = self.pkce_verifier.lock().unwrap().take()?;

    let token_res = self
      .oauth_client
      .exchange_code(auth_code)
      .set_pkce_verifier(pkce_verifier)
      .request_async(async_http_client)
      .await
      .ok()?;

    let access_token = token_res.access_token().secret().to_string();
    let hidden = {
      let (f, l) = access_token.split_at(4);
      format!("{}{}", f, "*".repeat(l.len()))
    };
    info!("Access token: {}", hidden);
    self.access_token = Some(access_token);

    let mut res = Response::basic(302, "Found");
    res.add_header("location".to_string(), "/");

    Some(res)
  }

  async fn handle_new_url<'s, 'r>(&'s mut self) -> Option<Response<'r>> {
    let (url, pkce_code) = self.generate_auth_url();
    *self.pkce_verifier.lock().unwrap() = Some(pkce_code);
    Some(
      Response::from_json(
        200,
        json!({
          "oauth_url": url,
        }),
      )
      .unwrap(),
    )
  }
}

#[async_trait]
impl ApiMethod for OAuthMethod {
  fn get_endpoint(&self) -> &str {
    "/oauth"
  }

  async fn handle_get<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    match req.get_endpoint().rsplit("/").next() {
      Some("callback") => return self.handle_callback(req).await,
      Some("new") => return self.handle_new_url().await,
      Some(_) | None => return Some(Response::basic(400, "Bad Request")),
    }
  }

  async fn handle_post<'s, 'r>(&'s mut self, _req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    None
  }
}
