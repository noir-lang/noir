use crate::git::nargo_crates;
use anyhow::{ensure, Context, Result};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use semver::Version;
use std::error::Error;
// Import necessary traits
use futures::{StreamExt, TryFutureExt};
// Import StreamExt for streaming
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use tar::Archive;
use zstd::Decoder;

pub const DEFAULT_REGISTRY_INDEX: &str = "http://localhost:3000";

// package_name - e.g. asdf@1.0.0
pub fn fetch_from_registry(package_name: &String, version: &String) -> Result<String, Box<dyn Error>> {
    let package = format!("{}_{}", package_name, version);
    let version = version.parse::<Version>()?;
    let loc = nargo_crates().join("registry").join(&package);
    if loc.join(&package_name).join("Nargo.toml").exists() {
        return Ok(loc.to_str().unwrap().to_string());
    }
    fs::create_dir_all(&loc)?;
    let url = format!("{}/api/v1/{}/{}", DEFAULT_REGISTRY_INDEX, package_name, version);
    let response = HTTP_CLIENT.get(&url)
        .send()
        .unwrap();

    // Check if the request was successful
    if response.status().is_success() {
        // // Create the output file at the specified path
        let output_file_path = loc.join(package);
        let output_file = File::create(&output_file_path)
            .map_err(|e| Box::new(e) as Box<dyn Error>)?; // Handle the error

        // Create a buffered writer for the file
        let mut writer = BufWriter::new(output_file);

        // Write the response body to the file
        let content = response.bytes()?;
        writer.write_all(&content)?;
        writer.flush().unwrap();

        extract_package(&output_file_path, &loc)?;
        println!("Successfully extracted package to: {}", loc.to_str().unwrap());
        Ok(loc.to_str().unwrap().to_string())
    } else {
        Err(Box::new(io::Error::new(io::ErrorKind::Other, "Failed to download package")))
    }
}

// Create a static Lazy instance that holds the reqwest Client
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .build()
        .expect("Failed to create HTTP client")
});

fn extract_package(package_path: &Path, extract_to: &Path) -> Result<()> {
    // Open the downloaded package file
    let mut archive = File::open(package_path)
        .with_context(|| format!("Failed to open package file: {:?}", package_path))?;

    // Seek to the start of the file
    archive.seek(SeekFrom::Start(0))
        .with_context(|| "Failed to seek to start of the archive")?;

    // Decode the zstd-compressed tarball
    let zst_decoder = Decoder::new(&archive)
        .with_context(|| "Failed to create zstd decoder")?;

    // Create a tar archive from the decoded content
    let mut tar = Archive::new(zst_decoder);

    // Iterate through the tar entries
    for entry in tar.entries()? {
        let mut entry = entry
            .with_context(|| "Failed to iterate over archive entries")?;

        // Extract the entry path and clone it to avoid holding a borrow
        let entry_path = entry
            .path()
            .with_context(|| "Failed to read entry path")?
            .to_path_buf();  // Convert to `PathBuf`, an owned value

        // Skip the OK-file if necessary
        if entry_path.file_name().unwrap_or_default() == "OK_FILE" {
            continue;
        }

        // Now that we no longer hold an immutable borrow on `entry`,
        // we can safely mutably borrow it for unpacking
        entry.unpack_in(extract_to)
            .with_context(|| format!("Failed to extract: {:?}", entry_path))?;
    }

    Ok(())
}