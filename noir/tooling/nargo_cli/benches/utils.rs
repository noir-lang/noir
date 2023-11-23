use std::path::PathBuf;

#[allow(unused)]
fn get_selected_tests() -> Vec<PathBuf> {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap().join("crates").join("nargo_cli"),
    };
    let test_dir = manifest_dir.join("tests").join("execution_success");

    let selected_tests =
        vec!["8_integration", "sha256_blocks", "struct", "eddsa", "regression", "regression_2099"];
    selected_tests.into_iter().map(|t| test_dir.join(t)).collect()
}
