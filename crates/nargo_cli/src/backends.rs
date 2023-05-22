pub(crate) use acvm_backend_barretenberg::Barretenberg as ConcreteBackend;

#[cfg(not(any(feature = "plonk_bn254", feature = "plonk_bn254_wasm")))]
compile_error!("please specify a backend to compile with");

#[cfg(all(feature = "plonk_bn254", feature = "plonk_bn254_wasm"))]
compile_error!(
    "feature \"plonk_bn254\"  and feature \"plonk_bn254_wasm\" cannot be enabled at the same time"
);
