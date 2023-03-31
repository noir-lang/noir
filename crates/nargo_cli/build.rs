use rustc_version::{version, Version};

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("1.66.0").unwrap(),
        "The minimal supported rustc version is 1.66.0."
    );
}

fn main() {
    check_rustc_version();

    build_data::set_GIT_COMMIT();
    build_data::set_GIT_DIRTY();
    build_data::no_debug_rebuilds();
}
