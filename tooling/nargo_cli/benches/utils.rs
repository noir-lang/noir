use std::path::PathBuf;

#[allow(unused)]
fn get_selected_tests() -> Vec<PathBuf> {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap(),
    };

    let test_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_programs")
        .join("execution_success");

    let selected_tests = vec!["struct", "eddsa", "regression"];
    let mut selected_tests =
        selected_tests.into_iter().map(|t| test_dir.join(t)).collect::<Vec<_>>();

    let test_dir = test_dir.parent().unwrap().join("benchmarks");
    selected_tests.extend(test_dir.read_dir().unwrap().filter_map(|e| e.ok()).map(|e| e.path()));

    selected_tests
}
