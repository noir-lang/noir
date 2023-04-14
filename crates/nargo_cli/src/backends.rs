cfg_if::cfg_if! {
    if #[cfg(feature = "plonk_bn254")] {
        pub(crate) use aztec_backend::Plonk as ConcreteBackend;
    } else if #[cfg(feature = "plonk_bn254_wasm")] {
        pub(crate) use aztec_wasm_backend::Plonk as ConcreteBackend;
    } else {
        compile_error!("please specify a backend to compile with");
    }
}

// As we have 3 feature flags we must test all 3 potential pairings to ensure they're mutually exclusive.
#[cfg(all(feature = "plonk_bn254", feature = "plonk_bn254_wasm"))]
compile_error!(
    "feature \"plonk_bn254\"  and feature \"plonk_bn254_wasm\" cannot be enabled at the same time"
);
