#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

cfg_if::cfg_if! {
    if #[cfg(feature = "bn254")] {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bn254::Fr>;
    } else if #[cfg(feature = "bls12_381")] {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bls12_381::Fr>;
    } else {
        compile_error!("please specify a field to compile with");
    }
}

// This is needed because features are additive through the dependency graph; if a dependency turns on the bn254, then it
// will be turned on in all crates that depend on it
#[macro_export]
macro_rules! assert_unique_feature {
    () => {};
    ($first:tt $(,$rest:tt)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_feature!($($rest),*);
    }
}
// https://internals.rust-lang.org/t/mutually-exclusive-feature-flags/8601/7
// If another field/feature is added, we add it here too
assert_unique_feature!("bn254", "bls12_381");
