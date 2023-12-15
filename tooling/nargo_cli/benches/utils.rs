use std::path::PathBuf;

#[allow(unused)]
fn get_selected_tests() -> Vec<PathBuf> {
    let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap(),
    }
    .parent()
    .unwrap()
    .parent()
    .unwrap()
    .join("test_programs");

    let test_dir = root_dir.join("execution_success");
    let selected_tests = vec!["sha256_byte", "struct", "eddsa", "regression", "regression_2099"];
    let selected_tests = selected_tests.into_iter().map(|t| test_dir.join(t));

    let benchmark_dir = root_dir.join("benchmarks");
    let benchmarks = vec!["large_loop"];
    let benchmarks = benchmarks.into_iter().map(|t| benchmark_dir.join(t));

    selected_tests.chain(benchmarks).collect()
}
