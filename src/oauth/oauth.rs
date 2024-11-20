use std::{
  fmt::format, str::FromStr, sync::{Arc, LazyLock, Mutex}
};

use async_trait::async_trait;
use log::info;
use oauth2::{
  basic::BasicClient, reqwest::async_http_client, url::Url, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl
};
use serde_json::{json, to_string};
use webrs::{api::ApiMethod, request::Request, response::Response};

use super::OAuthParameters;

pub struct OAuth {
  oauth_client: BasicClient,
  pub pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
}

impl OAuth {
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
      RedirectUrl::new("http://localhost:8080/api/oauth/callback".to_string()).unwrap(),
    );

    Self {
      oauth_client,
      pkce_verifier: Arc::new(Mutex::new(None)),
    }
  }

  pub fn generate_auth_url(&self) -> (String, PkceCodeVerifier) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let auth_url = self
      .oauth_client
      .authorize_url(|| CsrfToken::new_random())
      .add_scope(Scope::new(
        "https://www.googleapis.com/auth/photoslibrary.readonly".to_string(),
      ))
      .set_pkce_challenge(pkce_challenge)
      .url();

    (auth_url.0.to_string(), pkce_verifier)
  }
}

#[async_trait]
impl ApiMethod for OAuth {
  fn get_endpoint(&self) -> &str {
    "/oauth/callback"
  }

  async fn handle_get<'s, 'r>(&'s self, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    let query = req.get_url_params();
    let code = query.get("code")?.to_string();

    let auth_code = AuthorizationCode::new(code);

    let pkce_verifier = self.pkce_verifier.lock().unwrap().take()?;

    let token_res = self.oauth_client
      .exchange_code(auth_code)
      .set_pkce_verifier(pkce_verifier)
      .request_async(async_http_client)
      .await
      .ok()?;

    let access_token = token_res.access_token().secret().to_string();

    info!("Access token: {}", access_token);

    let mut res = Response::new(200, "application/json");
    res.set_data(to_string(&json!({
      "success": "ok"
    })).unwrap().into_bytes());
    Some(res)
  }

  async fn handle_post<'s, 'r>(&'s mut self, req: Request<'r>) -> Option<Response<'r>>
  where
    'r: 's,
  {
    None
  }
}
