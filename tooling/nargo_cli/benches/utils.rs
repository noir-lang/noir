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
        .join("benchmarks");

        let dirs = std::fs::read_dir(&test_dir).unwrap();

    dirs.into_iter().map(|t| test_dir.join(t.unwrap().path())).collect()
}
