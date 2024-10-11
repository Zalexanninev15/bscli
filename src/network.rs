use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use reqwest;
use serde_json::Value;
use indicatif::{ ProgressBar, ProgressStyle };
use futures_util::StreamExt;

use crate::main::REPO_URL;

pub async fn fetch_repo_data() -> Result<Value, Box<dyn std::error::Error>> {
    let resp = reqwest::get(REPO_URL).await?.text().await?;
    Ok(serde_json::from_str(&resp)?)
}

pub async fn download_file(
    url: &str,
    file_name: &str
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    let total_size = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )?
            .progress_chars("#>-")
    );

    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(file_name);
    let mut file = File::create(&temp_path)?;

    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message("Download completed");

    Ok(temp_path)
}
