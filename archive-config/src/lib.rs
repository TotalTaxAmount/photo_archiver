use std::{env::var, fs::File, io::Write, net::Ipv4Addr, path::Path, process::exit};

use lazy_static::lazy_static;
use log::error;
use serde::{Deserialize, Serialize};

lazy_static! {
  pub static ref CONFIG: Config = Config::parse(var("CONFIG_PATH").unwrap());
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
  pub port: u16,
  pub content_dir: String,
  pub client_secret_path: String,
  pub compression: Compression,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Compression {
  pub zstd: bool,
  pub br: bool,
  pub gzip: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DatabaseConfig {
  pub ip: Ipv4Addr,
  pub port: u16,
  pub timeout: u64,
  pub username: String,
  pub password: String,
  pub dbname: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
  pub jwt_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
  pub auth: AuthConfig,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      server: ServerConfig {
        port: 8080,
        content_dir: "html".to_string(),
        client_secret_path: "secret.json".to_string(),
        compression: Compression {
          zstd: true,
          br: true,
          gzip: true,
        },
      },
      database: DatabaseConfig {
        ip: Ipv4Addr::new(127, 0, 0, 1),
        port: 5432,
        timeout: 5,
        username: "username".to_string(),
        password: "password".to_string(),
        dbname: "photoarchiver".to_string(),
      },
      auth: AuthConfig {
        jwt_secret: "changeme".to_string(),
      },
    }
  }
}

impl Config {
  pub fn parse<P>(path: P) -> Self
  where
    P: AsRef<Path> + ToString,
  {
    let set_default = !Path::new(&path.to_string()).exists();

    let mut config_file = match File::options()
      .create(true)
      .write(true)
      .read(true)
      .append(true)
      .open(&path)
    {
      Ok(f) => f,
      Err(e) => {
        error!("Failed to open config file: {}", e);
        exit(1)
      }
    };

    if set_default {
      let toml_string = toml::to_string(&Config::default()).unwrap();
      config_file.write_all(toml_string.as_bytes()).unwrap();
      error!(
        "No config file! Creating and saving default to {}",
        path.to_string()
      );
      exit(1);
    }

    let config: Self = toml::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| {
      error!("Failed to read config file: {}", e);
      exit(1);
    }))
    .unwrap_or_else(|e| {
      error!("Failed to parse config: {}", e);
      exit(1);
    });

    config
  }
}
