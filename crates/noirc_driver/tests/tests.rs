use noirc_driver::Driver;
use std::path::PathBuf;

#[test]
fn fail() {
    let mut fail_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fail_dir.push("tests/fail/");

    let paths = std::fs::read_dir(fail_dir).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        assert!(!Driver::file_compiles(&path), "path: {}", path.display())
    }
}
#[test]
#[cfg(feature = "std")]
fn pass() {
    let mut pass_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pass_dir.push("tests/pass/");

    let paths = std::fs::read_dir(pass_dir).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        assert!(Driver::file_compiles(&path), "path: {}", path.display())
    }
}
