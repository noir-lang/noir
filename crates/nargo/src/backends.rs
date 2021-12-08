cfg_if::cfg_if! {
    if #[cfg(feature = "plonk_bn254")] {

        pub use aztec_backend::Plonk as ConcreteBackend;

    } else if #[cfg(feature = "marlin")] {
        // R1CS_MARLIN_ARKWORKS
        compile_error!("marlin backend has not been configured yet");

    } else {
        compile_error!("please specify a backend to compile with");
    }
}
// XXX: This works  because there are only two features, we want to say only one of these can be enabled. (feature xor)
#[cfg(all(feature = "plonk", feature = "marlin"))]
compile_error!("feature \"plonk\"  and feature \"marlin\" cannot be enabled at the same time");
