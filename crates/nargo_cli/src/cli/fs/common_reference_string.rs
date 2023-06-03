use std::{env, path::PathBuf, sync::atomic::AtomicU32};

use acvm::{acir::circuit::Circuit, CommonReferenceString};

use super::{create_named_dir, write_to_file};

use crc::{Crc, CRC_32_ISCSI};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";
const TRANSCRIPT_NAME: &str = "common-reference-string.bin";
const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

static CRS_CHECKSUM: AtomicU32 = AtomicU32::new(0);

fn common_reference_string_location() -> PathBuf {
    let cache_dir = match env::var("NARGO_BACKEND_CACHE_DIR") {
        Ok(cache_dir) => PathBuf::from(cache_dir),
        Err(_) => dirs::home_dir().unwrap().join(".nargo").join("backends"),
    };
    cache_dir.join(BACKEND_IDENTIFIER).join(TRANSCRIPT_NAME)
}

pub(crate) fn read_cached_common_reference_string() -> Vec<u8> {
    let crs_path = common_reference_string_location();

    match std::fs::read(crs_path) {
        Ok(common_reference_string) => {
            let loaded_checksum = CRS_CHECKSUM.load(std::sync::atomic::Ordering::Acquire);
            let checksum = CASTAGNOLI.checksum(&common_reference_string);
            if loaded_checksum == 0 {
                CRS_CHECKSUM
                    .compare_exchange_weak(
                        loaded_checksum,
                        checksum,
                        std::sync::atomic::Ordering::Acquire,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                    .unwrap();
            } else if checksum != loaded_checksum {
                panic!("Common reference string checksum mismatch");
            }
            return common_reference_string;
        }
        Err(_) => vec![],
    }
}

pub(crate) fn update_common_reference_string<B: CommonReferenceString>(
    backend: &B,
    common_reference_string: &[u8],
    circuit: &Circuit,
) -> Result<Vec<u8>, B::Error> {
    use tokio::runtime::Builder;

    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    // TODO(#1391): Implement retries
    // If the read data is empty, we don't have a CRS and need to generate one
    let fut = if common_reference_string.is_empty() {
        backend.generate_common_reference_string(circuit)
    } else {
        backend.update_common_reference_string(common_reference_string.to_vec(), circuit)
    };

    match runtime.block_on(fut) {
        Ok(common_reference_string) => {
            let checksum = CASTAGNOLI.checksum(&common_reference_string);
            CRS_CHECKSUM.store(checksum, std::sync::atomic::Ordering::Release);
            return Ok(common_reference_string);
        }
        Err(e) => return Err(e),
    };
}

pub(crate) fn write_cached_common_reference_string(common_reference_string: &[u8]) {
    let crs_path = common_reference_string_location();

    create_named_dir(crs_path.parent().unwrap(), "crs");

    write_to_file(common_reference_string, &crs_path);
    let checksum = CASTAGNOLI.checksum(common_reference_string);
    CRS_CHECKSUM.store(checksum, std::sync::atomic::Ordering::Release);
}
