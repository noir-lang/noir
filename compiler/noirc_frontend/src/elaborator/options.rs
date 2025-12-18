//! Compiler frontend options and unstable feature flags.

use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnstableFeature {
    Enums,
    Ownership,
}

impl std::fmt::Display for UnstableFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Enums => write!(f, "enums"),
            Self::Ownership => write!(f, "ownership"),
        }
    }
}

impl FromStr for UnstableFeature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enums" => Ok(Self::Enums),
            "ownership" => Ok(Self::Ownership),
            other => Err(format!("Unknown unstable feature '{other}'")),
        }
    }
}

/// Generic options struct meant to resolve to ElaboratorOptions below when
/// we can resolve a file path to a file id later. This generic struct is used
/// so that FrontendOptions doesn't need to duplicate fields and methods with ElaboratorOptions.
#[derive(Copy, Clone)]
pub struct GenericOptions<'a, T> {
    /// The scope of --debug-comptime, or None if unset
    pub debug_comptime_in_file: Option<T>,

    /// Use pedantic ACVM solving
    pub pedantic_solving: bool,

    /// Unstable compiler features that were explicitly enabled. Any unstable features
    /// that are not in this vector result in an error when used.
    pub enabled_unstable_features: &'a [UnstableFeature],

    /// Deny crates from requiring unstable features.
    pub disable_required_unstable_features: bool,
}

/// Options from nargo_cli that need to be passed down to the elaborator
pub(crate) type ElaboratorOptions<'a> = GenericOptions<'a, fm::FileId>;

/// This is the unresolved version of `ElaboratorOptions`
/// CLI options that need to be passed to the compiler frontend (the elaborator).
pub type FrontendOptions<'a> = GenericOptions<'a, &'a str>;

impl<T> GenericOptions<'_, T> {
    /// A sane default of frontend options for running tests
    pub fn test_default() -> GenericOptions<'static, T> {
        GenericOptions {
            debug_comptime_in_file: None,
            pedantic_solving: true,
            enabled_unstable_features: &[UnstableFeature::Enums],
            disable_required_unstable_features: true,
        }
    }
}
