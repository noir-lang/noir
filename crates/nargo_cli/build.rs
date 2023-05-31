use rustc_version::{version, Version};

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("1.66.0").unwrap(),
        "The minimal supported rustc version is 1.66.0."
    );
}

const GIT_COMMIT: &&str = &"GIT_COMMIT";

fn main() {
    check_rustc_version();

    // Only use build_data if the environment variable isn't set
    // The environment variable is always set when working via Nix
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }
}
