use rustc_version::{version, Version};

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("7.0").unwrap(),
        "The minimal supported rustc version is 1.67.0."
    );
}

fn main() {
    check_rustc_version();
}
