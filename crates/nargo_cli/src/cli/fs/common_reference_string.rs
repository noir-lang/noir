use std::{env, path::PathBuf};

use acvm::{acir::circuit::Circuit, CommonReferenceString};

use super::{create_named_dir, write_to_file};

// TODO: pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";
const TRANSCRIPT_NAME: &str = "common-reference-string.bin";

fn common_reference_string_location() -> PathBuf {
    let cache_dir = match env::var("BACKEND_CACHE_DIR") {
        Ok(cache_dir) => PathBuf::from(cache_dir),
        Err(_) => dirs::home_dir().unwrap().join(".nargo").join("backends"),
    };
    cache_dir.join(BACKEND_IDENTIFIER).join(TRANSCRIPT_NAME)
}

pub(crate) fn get_common_reference_string<B: CommonReferenceString>(
    backend: &B,
    circuit: &Circuit,
) -> Result<Vec<u8>, B::Error> {
    use tokio::runtime::Builder;

    let crs_path = common_reference_string_location();

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    // TODO: Implement retries
    let crs = match std::fs::read(&crs_path) {
        // If the read data is empty, we don't have a CRS and need to generate one
        Ok(common_reference_string) if !common_reference_string.is_empty() => runtime
            .block_on(backend.update_common_reference_string(common_reference_string, circuit))?,
        Ok(_) | Err(_) => runtime.block_on(backend.generate_common_reference_string(circuit))?,
    };

    create_named_dir(crs_path.parent().unwrap(), "crs");

    write_to_file(crs.as_slice(), &crs_path);

    Ok(crs)
}
