use std::{env, path::PathBuf};

use acvm::{acir::circuit::Circuit, CommonReferenceString};

use super::{create_named_dir, write_to_file};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";
const TRANSCRIPT_NAME: &str = "common-reference-string.bin";

fn common_reference_string_location() -> PathBuf {
    let cache_dir = match env::var("NARGO_BACKEND_CACHE_DIR") {
        Ok(cache_dir) => PathBuf::from(cache_dir),
        Err(_) => dirs::home_dir().unwrap().join(".nargo").join("backends"),
    };
    cache_dir.join(BACKEND_IDENTIFIER).join(TRANSCRIPT_NAME)
}

pub(crate) fn read_cached_common_reference_string() -> Vec<u8> {
    let crs_path = common_reference_string_location();

    // TODO(#1390): Implement checksum
    match std::fs::read(crs_path) {
        Ok(common_reference_string) => common_reference_string,
        Err(_) => vec![],
    }
}

pub(crate) fn update_common_reference_string<B: CommonReferenceString>(
    backend: &B,
    mut common_reference_string: Vec<u8>,
    circuit: &Circuit,
) -> Result<(), B::Error> {
    use tokio::runtime::Builder;

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    // TODO(#1391): Implement retries
    // If the read data is empty, we don't have a CRS and need to generate one
    let fut = if common_reference_string.is_empty() {
        backend.generate_common_reference_string(circuit)
    } else {
        backend.update_common_reference_string(common_reference_string.to_vec(), circuit)
    };

    runtime.block_on(async {
        common_reference_string = fut.await?;
        Ok(())
    })
}

pub(crate) fn write_cached_common_reference_string(common_reference_string: &[u8]) {
    let crs_path = common_reference_string_location();

    create_named_dir(crs_path.parent().unwrap(), "crs");

    write_to_file(common_reference_string, &crs_path);
}
