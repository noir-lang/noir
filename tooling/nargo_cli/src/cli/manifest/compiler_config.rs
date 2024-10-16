use serde::{Deserialize, Serialize};
use std::hash::Hash;
use crate::cli::manifest::profile::{DefaultForProfile, Profile};
use crate::cli::manifest::TomlCairo;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ManifestCompilerConfig {
    /// Replace all names in generated Sierra code with dummy counterparts, representing the
    /// expanded information about the named items.
    ///
    /// For libfuncs and types that would be recursively opening their generic arguments.
    /// For functions, that would be their original name in Cairo.
    /// For example, while the Sierra name be `[6]`, with this flag turned on it might be:
    /// - For libfuncs: `felt252_const<2>` or `unbox<Box<Box<felt252>>>`.
    /// - For types: `felt252` or `Box<Box<felt252>>`.
    /// - For user functions: `test::foo`.
    pub sierra_replace_ids: bool,
    /// Do not exit with error on compiler warnings.
    pub allow_warnings: bool,
    /// Enable auto gas withdrawal and gas usage check.
    pub enable_gas: bool,
    /// Add a mapping between sierra statement indexes and fully qualified paths of cairo functions
    /// to debug info. A statement index maps to a vector consisting of a function which caused the
    /// statement to be generated and all functions that were inlined or generated along the way.
    /// Used by [cairo-profiler](https://github.com/software-mansion/cairo-profiler).
    /// This feature is unstable and is subject to change.
    pub unstable_add_statements_functions_debug_info: bool,
    /// Add a mapping between sierra statement indexes and code location in cairo code
    /// to debug info. A statement index maps to a vector consisting of a code location which caused the
    /// statement to be generated and all code location that were inlined or generated along the way.
    /// Used by [cairo-coverage](https://github.com/software-mansion/cairo-coverage).
    /// This feature is unstable and is subject to change.
    pub unstable_add_statements_code_locations_debug_info: bool,
    // Inlining strategy.
    pub inlining_strategy: InliningStrategy,
}

#[derive(Debug, Default, Deserialize, Serialize, Eq, PartialEq, Hash, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum InliningStrategy {
    /// Do not override inlining strategy.
    #[default]
    Default,
    /// Inline only in the case of an `inline(always)` annotation.
    Avoid,
}

//todo fix for aztec
impl DefaultForProfile for ManifestCompilerConfig {
    fn default_for_profile(profile: &Profile) -> Self {
        Self {
            sierra_replace_ids: profile.is_dev(),
            allow_warnings: true,
            enable_gas: true,
            unstable_add_statements_functions_debug_info: false,
            unstable_add_statements_code_locations_debug_info: false,
            inlining_strategy: InliningStrategy::default(),
        }
    }
}

impl From<ManifestCompilerConfig> for TomlCairo {
    fn from(config: ManifestCompilerConfig) -> Self {
        Self {
            sierra_replace_ids: Some(config.sierra_replace_ids),
            allow_warnings: Some(config.allow_warnings),
            enable_gas: Some(config.enable_gas),
            unstable_add_statements_functions_debug_info: Some(
                config.unstable_add_statements_functions_debug_info,
            ),
            unstable_add_statements_code_locations_debug_info: Some(
                config.unstable_add_statements_code_locations_debug_info,
            ),
            inlining_strategy: Some(config.inlining_strategy),
        }
    }
}
