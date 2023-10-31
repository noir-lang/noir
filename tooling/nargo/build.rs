use rustc_version::{version, Version};

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("1.71.1").unwrap(),
        "The minimal supported rustc version is 1.71.1."
    );
}

fn main() {
    check_rustc_version();
}
