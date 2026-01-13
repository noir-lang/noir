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
