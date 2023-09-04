use acvm_backend_barretenberg::backends_directory;
pub(crate) use acvm_backend_barretenberg::Backend;

pub(crate) fn set_active_backend(backend_name: &str) {
    let backends_directory = backends_directory();
    let active_backend_file = backends_directory.join("./.selected_backend");

    std::fs::write(active_backend_file, backend_name.as_bytes()).unwrap();
}

pub(crate) fn get_active_backend() -> String {
    let backends_directory = backends_directory();
    let active_backend_file = backends_directory.join("./.selected_backend");

    if !active_backend_file.is_file() {
        let barretenberg = "acvm-backend-barretenberg";
        set_active_backend(barretenberg);
        return barretenberg.to_string();
    }

    std::fs::read_to_string(active_backend_file).unwrap()
}
