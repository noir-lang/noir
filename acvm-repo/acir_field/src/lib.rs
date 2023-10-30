#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use num_bigint::BigUint;
use num_traits::Num;

cfg_if::cfg_if! {
    if #[cfg(feature = "bn254")] {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bn254::Fr>;
        pub const CHOSEN_FIELD : FieldOptions = FieldOptions::BN254;

    } else if #[cfg(feature = "bls12_381")] {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bls12_381::Fr>;
        pub const CHOSEN_FIELD : FieldOptions = FieldOptions::BLS12_381;
    } else {
        compile_error!("please specify a field to compile with");
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FieldOptions {
    BN254,
    BLS12_381,
}

impl FieldOptions {
    pub fn to_string(&self) -> &str {
        match self {
            FieldOptions::BN254 => "bn254",
            FieldOptions::BLS12_381 => "bls12_381",
        }
    }

    pub fn is_native_field(str: &str) -> bool {
        let big_num = if let Some(hex) = str.strip_prefix("0x") {
            BigUint::from_str_radix(hex, 16)
        } else {
            BigUint::from_str_radix(str, 10)
        };
        if let Ok(big_num) = big_num {
            big_num == FieldElement::modulus()
        } else {
            CHOSEN_FIELD.to_string() == str
        }
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
