use std::sync::Arc;

use serde_json::from_str;
use tokio::sync::OwnedSemaphorePermit;

use crate::{error::DownloaderError, Downloader, DownloaderPool};

pub struct DownloaderGuard {
  pub(crate) downloader: Option<Downloader>,
  pub(crate) pool: Arc<DownloaderPool>,
  pub(crate) _permit: OwnedSemaphorePermit,
}

impl DownloaderGuard {
  pub fn get(&mut self) -> &mut Downloader {
    self.downloader.as_mut().unwrap()
  }
}

impl Drop for DownloaderGuard {
  fn drop(&mut self) {
    if let Some(d) = self.downloader.take() {
      let pool = self.pool.clone();
      tokio::spawn(async move { pool.return_to_pool(d).await });
    }
  }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Photo {
  #[serde(rename = "apertureFNumber")]
  pub aperture_f_number: f64,
  #[serde(rename = "cameraMake")]
  pub camera_make: String,
  #[serde(rename = "cameraModel")]
  pub camera_model: String,
  #[serde(rename = "exposureTime")]
  pub exposure_time: String,
  #[serde(rename = "focalLength")]
  pub focal_length: f64,
  #[serde(rename = "isoEquivalent")]
  pub iso_equivalent: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaMetadata {
  #[serde(rename = "creationTime")]
  pub creation_time: String,
  pub height: String,
  pub width: String,
  pub photo: Photo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaItem {
  #[serde(rename = "baseUrl")]
  pub base_url: String,
  pub filename: String,
  pub id: String,
  #[serde(rename = "mediaMetadata")]
  pub media_metadata: MediaMetadata,
  #[serde(rename = "mimeType")]
  pub mime_type: String,
  #[serde(rename = "productUrl")]
  pub product_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaItemsResponse {
  #[serde(rename = "mediaItems")]
  pub media_items: Vec<MediaItem>,
  #[serde(rename = "nextPageToken")]
  pub next_page_token: String,
}

impl TryFrom<String> for MediaItemsResponse {
  type Error = serde_json::Error;
  fn try_from(value: String) -> Result<Self, Self::Error> {
    from_str::<Self>(&value)
  }
}
