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
// XXX: This works  because there are only two features, we want to say only one of these can be enabled. (feature xor)
// This is needed because features are additive through the dependency graph; if a dependency turns on the bn254, then it
// will be turned on in all crates that depend on it
#[cfg(all(feature = "bn254", feature = "bls12_381"))]
compile_error!("feature \"bn254\"  and feature \"bls12_381\" cannot be enabled at the same time");
