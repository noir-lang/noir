use std::path::PathBuf;

use acvm_backend_barretenberg::backends_directory;
pub(crate) use acvm_backend_barretenberg::Backend;

fn active_backend_file_path() -> PathBuf {
    backends_directory().join(".selected_backend")
}

pub(crate) const ACVM_BACKEND_BARRETENBERG: &str = "acvm-backend-barretenberg";

pub(crate) fn clear_active_backend() {
    let active_backend_file = active_backend_file_path();
    if active_backend_file.is_file() {
        std::fs::remove_file(active_backend_file_path())
            .expect("should delete active backend file");
    }
}

pub(crate) fn set_active_backend(backend_name: &str) {
    std::fs::create_dir_all(
        active_backend_file_path().parent().expect("active backend file should have parent"),
    )
    .unwrap();
    std::fs::write(active_backend_file_path(), backend_name.as_bytes()).unwrap();
}

pub(crate) fn get_active_backend() -> String {
    let active_backend_file = active_backend_file_path();

    if !active_backend_file.is_file() {
        set_active_backend(ACVM_BACKEND_BARRETENBERG);
        return ACVM_BACKEND_BARRETENBERG.to_string();
    }

    std::fs::read_to_string(active_backend_file).unwrap()
}
