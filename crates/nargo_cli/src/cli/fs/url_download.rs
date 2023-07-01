use thiserror::Error;
use std::{cmp::min, io::Write};

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::IntoUrl;

#[derive(Debug, Error)]
pub(crate) enum NargoDownloadError {
    #[error("{0}")]
    Generic(String),
}

impl From<reqwest::Error> for NargoDownloadError {
    fn from(value: reqwest::Error) -> Self {
        NargoDownloadError::Generic(value.to_string())
    }
}
impl From<std::io::Error> for NargoDownloadError {
    fn from(value: std::io::Error) -> Self {
        NargoDownloadError::Generic(value.to_string())
    }
}

pub(crate) async fn download_from_url<T>(url: T, name: &str) -> Result<(), NargoDownloadError>
where
    T: IntoUrl,
{
    let mut resp = reqwest::get(url).await?;
    resp.content_length();
    let mut f = std::fs::File::create(name)?;
    let mut downloaded = 0;
    let total_size = resp
        .content_length()
        .expect("Cannot determine size of the content!");
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap()
            .progress_chars("#>-"));
    while let Some(chunk) = resp.chunk().await? {
        let new = min(downloaded + chunk.len() as u64, total_size);
        downloaded = new;
        pb.set_position(new);
        f.write_all(&chunk[..])?;
    }
    Ok(())
}