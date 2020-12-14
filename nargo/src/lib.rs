/// Nargo is the package manager although right now, it is acting as a glorified crate for the compilation unit data structure

pub mod crate_unit;
pub mod crate_manager;

// XXX: We could argue that this should not be in nargo because compilers like
// rustc allow : `rustc main.rs foo.rs dep.rs`
pub use crate_unit::CrateUnit;

pub use crate_manager::CrateManager;

pub mod dir_util;