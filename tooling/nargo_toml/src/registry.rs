use std::error::Error;
use std::fs;
use semver::Version;
use once_cell::sync::Lazy;
use reqwest::Client;
use crate::git::nargo_crates;
use anyhow::{Context, Result};
use tokio::fs::File;                     // Ensure to use tokio::fs::File
use tokio::io::{self, AsyncWriteExt, BufWriter};        // Import necessary traits
use futures::StreamExt;                  // Import StreamExt for streaming


pub const DEFAULT_REGISTRY_INDEX: &str = "http://localhost:3000";

// package_name - e.g. asdf@1.0.0
pub fn fetch_from_registry(package: &String) -> Result<String, Box<dyn Error>> {
    let fut = async {
        let (package_name, version) = split_package_name_version(&package)?;
        let loc = nargo_crates().join("registry").join(&package);
        if loc.exists() {
            return Ok(loc.to_str().unwrap().to_string());
        }
        fs::create_dir_all(&loc)?;

        let response = HTTP_CLIENT.get(&format!("{}/api/v1/{}/{}", DEFAULT_REGISTRY_INDEX, package_name, version.to_string()))
            .send()
            .await
            .unwrap();

        // Create the output file at the specified path
        let output_file_path = loc.join(&package);
        let mut output_file = File::create(output_file_path)
            .await
            .context("failed to create output file")?;

        // Create a buffered writer for the file
        let mut writer = BufWriter::new(&mut output_file);

        // Stream the response bytes
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("failed to read response chunk")?;
            io::copy_buf(&mut &*chunk, &mut writer)
                .await
                .context("failed to save response chunk on disk")?;
        }

        // Ensure that all buffered data is written to the file
        writer.flush().await.context("failed to flush writer")?;

        // if response.status().is_success() {
        //     println!("Successfully downloaded the package: {}", package_name);
        // } else {
        //     eprintln!("Failed to upload package: {}. Status: {}", package_name, response.status());
        // }
        Ok(loc.to_str().unwrap().to_string())
    };
    tokio::runtime::Handle::current().block_on(fut)
}

pub fn split_package_name_version(input: &str) -> Result<(String, Version), Box<dyn Error>> {
    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() != 2 {
        return Err("Invalid input format, expected 'package@version'".into());
    }
    let package_name = parts[0].to_string();
    let version_str = parts[1];
    let version = version_str.parse::<Version>()?;
    Ok((package_name, version))
}

// Create a static Lazy instance that holds the reqwest Client
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .build()
        .expect("Failed to create HTTP client")
});