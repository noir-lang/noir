use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use const_format::formatcp;

const DEST_FOLDER: &str = ".nargo/backends/acvm-backend-barretenberg";
const BINARY_NAME: &str = "backend_binary";

pub(crate) fn get_binary_path() -> PathBuf {
    match std::env::var("BB_BINARY_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => dirs::home_dir().unwrap().join(formatcp!("{}/{}", DEST_FOLDER, BINARY_NAME)),
    }
}

const USERNAME: &str = "AztecProtocol";
const REPO: &str = "barretenberg";
const VERSION: &str = "0.4.6";
const TAG: &str = formatcp!("barretenberg-v{}", VERSION);

const API_URL: &str =
    formatcp!("https://github.com/{}/{}/releases/download/{}", USERNAME, REPO, TAG);

fn get_bb_download_url() -> String {
    if let Ok(path) = std::env::var("BB_BINARY_URL") {
        return path;
    }

    let target_os = env!("TARGET_OS");
    let target_arch = env!("TARGET_ARCH");

    let archive_name = match target_os {
        "linux" => "barretenberg-x86_64-linux-gnu.tar.gz",
        "macos" => match target_arch {
            "aarch64" => "barretenberg-aarch64-apple-darwin.tar.gz",
            "x86_64" => "barretenberg-x86_64-apple-darwin.tar.gz",
            arch => panic!("unsupported arch {arch}"),
        },
        os => panic!("Unsupported OS {os}"),
    };

    format!("{API_URL}/{archive_name}")
}

pub(crate) fn download_bb_binary(binary_path: &Path) {
    use flate2::read::GzDecoder;
    use tar::Archive;
    use tempfile::tempdir;

    // Create directory to place binary in.
    std::fs::create_dir_all(binary_path.parent().unwrap()).unwrap();

    // Download sources
    let compressed_file: Cursor<Vec<u8>> = download_binary_from_url(&get_bb_download_url())
        .unwrap_or_else(|error| panic!("\n\nDownload error: {error}\n\n"));

    // Unpack the tarball
    let gz_decoder = GzDecoder::new(compressed_file);
    let mut archive = Archive::new(gz_decoder);

    let temp_directory = tempdir().expect("could not create a temporary directory");
    archive.unpack(&temp_directory).unwrap();
    let temp_binary_path = temp_directory.path().join("bb");

    // Rename the binary to the desired name
    std::fs::copy(temp_binary_path, binary_path).unwrap();

    drop(temp_directory);
}

/// Try to download the specified URL into a buffer which is returned.
fn download_binary_from_url(url: &str) -> Result<Cursor<Vec<u8>>, String> {
    let response = reqwest::blocking::get(url).map_err(|error| error.to_string())?;

    let bytes = response.bytes().unwrap();

    // TODO: Check SHA of downloaded binary

    Ok(Cursor::new(bytes.to_vec()))
}
