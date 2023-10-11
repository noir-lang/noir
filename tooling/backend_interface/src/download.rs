use std::{
    io::{Cursor, ErrorKind},
    path::Path,
    process::{Command, Stdio},
};

/// Downloads a zipped archive and unpacks the backend binary to `destination_path`.
///
/// # Backend Requirements
///
/// In order for a backend to be compatible with this function:
/// - `backend_url` must serve a gzipped tarball.
/// - The tarball must only contain the backend's binary.
/// - The binary file must be located at the archive root.
pub fn download_backend(backend_url: &str, destination_path: &Path) -> std::io::Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    use tempfile::tempdir;

    // Download sources
    let compressed_file: Cursor<Vec<u8>> = download_binary_from_url(backend_url)
        .map_err(|_| std::io::Error::from(ErrorKind::Other))?;

    // Unpack the tarball
    let gz_decoder = GzDecoder::new(compressed_file);
    let mut archive = Archive::new(gz_decoder);

    let temp_directory = tempdir()?;
    archive.unpack(&temp_directory)?;

    // Assume that the archive contains a single file which is the backend binary.
    let mut archive_files = std::fs::read_dir(&temp_directory)?;
    let temp_binary_path = archive_files.next().unwrap()?.path();

    // Create directory to place binary in.
    std::fs::create_dir_all(destination_path.parent().unwrap())?;

    // Rename the binary to the desired name
    std::fs::copy(temp_binary_path, destination_path)?;

    drop(temp_directory);

    Ok(())
}

pub fn run_backend_installation_script(
    installation_script_path: &Path,
    destination_path: &Path,
) -> std::io::Result<()> {
    let mut cmd = Command::new("bash");

    // TODO: Pass ACIR version to check for compatibility.
    cmd.arg(installation_script_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("NARGO_OS", env!("NARGO_OS"))
        .env("NARGO_ARCHITECTURE", env!("NARGO_ARCHITECTURE"))
        .env("NARGO_BACKEND_DESTINATION_PATH", destination_path);

    cmd.output().map(|_| ())?;

    if destination_path.is_file() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Installation script did not place backend at expected location",
        ))
    }
}

/// Try to download the specified URL into a buffer which is returned.
fn download_binary_from_url(url: &str) -> Result<Cursor<Vec<u8>>, reqwest::Error> {
    let response = reqwest::blocking::get(url)?;

    let bytes = response.bytes()?;

    // TODO: Check SHA of downloaded binary

    Ok(Cursor::new(bytes.to_vec()))
}
