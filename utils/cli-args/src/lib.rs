#[derive(Copy, Clone, Debug, clap::ValueEnum, PartialEq, Eq)]
pub enum UnstableFeature {
    Enums,
    ArrayOwnership,
}

impl std::fmt::Display for UnstableFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Note: clap::ValueEnum uses kebab-case by default so
        // we need to replicate that here.
        match self {
            Self::Enums => write!(f, "enums"),
            Self::ArrayOwnership => write!(f, "array-ownership"),
        }
    }
}

/// CLI options that need to be passed to the compiler frontend (the elaborator)
#[derive(Copy, Clone)]
pub struct FrontendOptions<'a> {
    /// The scope of --debug-comptime, or None if unset
    pub debug_comptime_in_file: Option<&'a str>,

    /// Use pedantic ACVM solving
    pub pedantic_solving: bool,

    /// Unstable compiler features that were explicitly enabled. Any unstable features
    /// that are not in this list result in an error when used.
    pub enabled_unstable_features: &'a [UnstableFeature],
}

impl<'a> FrontendOptions<'a> {
    pub fn test_default() -> FrontendOptions<'static> {
        FrontendOptions {
            debug_comptime_in_file: None,
            pedantic_solving: true,
            enabled_unstable_features: &[UnstableFeature::Enums],
        }
    }
}
