cfg_if::cfg_if! {
    if #[cfg(feature = "bn254")] {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bn254::Fr>;

    } else if #[cfg(feature = "bls12_381")] {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bls12_381::Fr>;
    } else {
        mod generic_ark;
        pub type FieldElement = generic_ark::FieldElement<ark_bn254::Fr>;
    }
}
#[cfg(all(feature = "bn254", feature = "bls12_381"))]
compile_error!("feature \"bn254\" and feature \"bls12_381\" cannot be enabled at the same time");
