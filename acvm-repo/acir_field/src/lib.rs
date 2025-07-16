#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

mod field_element;
mod generic_ark;

use ark_ff::fields::{Fp64, MontBackend, MontConfig};
pub use generic_ark::AcirField;

// TODO : Shift it to external crate
#[derive(MontConfig)]
#[modulus = "2147483647"]
#[generator = "3"]
#[small_subgroup_base = "3"]
#[small_subgroup_power = "1"]
pub struct M31Config;
pub type M31 = Fp64<MontBackend<M31Config, 1>>;
pub type M31FieldElement = field_element::FieldElement<M31>;

/// Temporarily exported generic field to aid migration to `AcirField`
pub use field_element::FieldElement as GenericFieldElement;

cfg_if::cfg_if! {
    if #[cfg(feature = "bls12_381")] {
        pub type FieldElement = field_element::FieldElement<ark_bls12_381::Fr>;
    } else {
        // pub type FieldElement = field_element::FieldElement<ark_bn254::Fr>;
        pub type FieldElement = M31FieldElement;
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
