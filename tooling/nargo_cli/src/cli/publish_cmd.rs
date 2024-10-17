use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use crate::cli::package::flock::Filesystem;
use crate::cli::NargoConfig;
use crate::errors::CliError;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_frontend::graph::CrateName;
use once_cell::sync::Lazy;
use reqwest::multipart::{Form, Part};
use reqwest::{Body, Client};
use crate::cli::source::DEFAULT_REGISTRY_INDEX;

/// Add dependencies to a Nargo.toml manifest file
#[derive(Debug, Clone, Args)]
pub(crate) struct PublishCommand {
    /// The name of the package to compile
    // #[clap(long, conflicts_with = "workspace")]
    #[clap(long)]
    package: Option<CrateName>,
}

pub(crate) fn run(args: PublishCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection = PackageSelection::DefaultOrAll;
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;
    let target_dir = workspace.target_directory_path().join("package");

    // Create a new Filesystem instance that points to the 'package' directory.
    let tarball = Filesystem::new(Utf8PathBuf::from(target_dir.to_str().unwrap()));

    for package in &workspace {
        // Build the packed file path
        let packed_file_path = tarball.path_unchecked().join(Utf8PathBuf::from(package.name.to_string()));

        // Check if the packed file exists
        if !packed_file_path.exists() {
            eprintln!("Packed file does not exist: {}", packed_file_path);
            continue; // Skip this package if the file doesn't exist
        }

        // Open the file synchronously
        let mut file = std::fs::File::open(&packed_file_path)
            .context("Failed to open packed file").unwrap();

        let mut buffer = Vec::new();
        // Read the file into the buffer
        file.read_to_end(&mut buffer)
            .context("Failed to read packed file").unwrap();

        let length = buffer.len();


        // // Create the output file at the specified path
        // let output_file_path = target_dir.join(format!("dupa.zst"));
        // let output_file = File::create(&output_file_path).unwrap(); // Handle the error
        //
        // // Create a buffered writer for the file
        // let mut writer = BufWriter::new(output_file);
        //
        // writer.write_all(&buffer).unwrap();
        // extract_package(&output_file_path, &loc)?;
        // Create a Body from the buffer
        let file_part = reqwest::blocking::multipart::Part::bytes(buffer)
            .file_name(format!("{}_{}", package.name, package.version.as_ref().unwrap()))
            .mime_str("application/zstd") // Set MIME type to application/zstd
            .expect("Failed to set MIME type"); // Error handling for setting MIME type

        let form = reqwest::blocking::multipart::Form::new().part("file", file_part);
        println!("Buffer length: {}", length);

        // Send the request synchronously
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/api/v1", DEFAULT_REGISTRY_INDEX))
            .multipart(form)
            .send()
            .context("Failed to send request").unwrap();

        // Optionally, check the response here
        if response.status().is_success() {
            println!("Successfully uploaded package: {}", package.name);
        } else {
            eprintln!("Failed to upload package: {}. Status: {}", package.name, response.status());
        }
    }

    Ok(())
}


// pub(crate) async fn run(args: PublishCommand, config: NargoConfig) -> Result<(), CliError> {
//     let toml_path = get_package_manifest(&config.program_dir)?;
//     let default_selection = PackageSelection::DefaultOrAll;
//     let selection = args.package.map_or(default_selection, PackageSelection::Selected);
//
//     let workspace = resolve_workspace_from_toml(
//         &toml_path,
//         selection,
//         Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
//     )?;
//     let target_dir = workspace.target_directory_path().join("package");
//
//     // Create a new Filesystem instance that points to the 'package' directory.
//     let tarball = Filesystem::new(Utf8PathBuf::from(target_dir.to_str().unwrap()));
//
//     for package in &workspace {
//         // Build the packed file path
//         let packed_file_path = tarball.path_unchecked().join(Utf8PathBuf::from(package.name.to_string()));
//
//         // Check if the packed file exists
//         if !packed_file_path.exists() {
//             eprintln!("Packed file does not exist: {}", packed_file_path);
//             continue; // Skip this package if the file doesn't exist
//         }
//
//         // Open the file asynchronously
//         let mut file = tokio::fs::File::open(&packed_file_path)
//             .await
//             .context("Failed to open packed file").unwrap();
//
//         let mut buffer = Vec::new();
//         // Read the file into the buffer
//         file.read_to_end(&mut buffer)
//             .await
//             .context("Failed to read packed file").unwrap();
//
//         let length = buffer.len();
//         // Create a Body from the buffer
//         let file_part = Part::stream(Body::from(buffer))
//             .file_name(format!("{}_{}", package.name, package.version.as_ref().unwrap()));
//
//         let form = Form::new().part("file", file_part);
//         println!("Buffer length: {}", length);
//
//         // Send the request asynchronously
//         let response = HTTP_CLIENT
//             .post(format!("{}/api/v1", DEFAULT_REGISTRY_INDEX))
//             .multipart(form)
//             .send()
//             .await // Await the response
//             .context("Failed to send request").unwrap();
//
//         // Optionally, check the response here
//         if response.status().is_success() {
//             println!("Successfully uploaded package: {}", package.name);
//         } else {
//             eprintln!("Failed to upload package: {}. Status: {}", package.name, response.status());
//         }
//     }
//
//     Ok(())
// }

// Create a static Lazy instance that holds the reqwest Client
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .build()
        .expect("Failed to create HTTP client")
});