use std::path::PathBuf;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../noir_stdlib/src"]
#[cfg_attr(not(target_os = "windows"), prefix = "std/")]
#[cfg_attr(target_os = "windows", prefix = r"std\")] // Note reversed slash direction
struct StdLibAssets;

// Returns a vector of tuples containing the path to a stdlib file in the std lib crate
// along with the source code of that file.
//
// This is needed because when we preload the file manager, it needs to know where
// the source code for the stdlib is. The stdlib is treated special because it comes with
// the compiler and is never included as a dependency like other user defined crates.
pub fn stdlib_paths_with_source() -> Vec<(String, String)> {
    StdLibAssets::iter()
        .map(|path| {
            let source = std::str::from_utf8(StdLibAssets::get(&path).unwrap().data.as_ref())
                .unwrap()
                .to_string();
            (path.to_string(), source)
        })
        .collect()
}

/// Returns the contents of the Nargo.toml file for the standard library as a string.
pub fn stdlib_nargo_toml_source() -> String {
    include_str!("../../../noir_stdlib/Nargo.toml").to_string()
}

/// Returns the absolute path to the `noir_stdlib/src` directory on disk, when the
/// running binary was built from the monorepo (debug build only) and the directory
/// is still reachable from where the binary was compiled. Returns `None` for
/// release builds or when the source tree is no longer present (e.g. a distributed
/// `nargo`), so callers must fall back to the embedded stdlib copy.
pub fn stdlib_disk_path() -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../noir_stdlib/src").canonicalize().ok()
}
