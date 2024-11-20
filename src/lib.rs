use core::error;
use std::{
  env::{set_var, var},
  fs::{read_to_string, File},
  io::Write,
  path::Path,
  process::exit,
};

use log::{error, warn};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
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

impl Default for Config {
  fn default() -> Self {
    Self {
      port: 8080,
      content_dir: "html".to_string(),
      client_secret_path: "secrets.json".to_string(),
      compression: Compression {
        zstd: true,
        br: true,
        gzip: true,
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
      Err(_) => todo!(),
    };

    if set_default {
      let toml_string = toml::to_string(&Config::default()).unwrap();
      config_file.write_all(toml_string.as_bytes()).unwrap();
      error!(
        "No config file! Creating and saving default to {}",
        path.to_string()
      );
      exit(0);
    }

    let config: Self = match read_to_string(path) {
      Ok(s) => toml::from_str(&s).unwrap(),
      Err(e) => {
        error!("Failed to read config file: {}", e);
        exit(0)
      }
    };

    config
  }
}
