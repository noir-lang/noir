cfg_if::cfg_if! {
    if #[cfg(feature = "plonk")] {
        // CSAT_3_PLONK_AZTEC

        pub mod csat_3_plonk_aztec;
        pub use csat_3_plonk_aztec::Plonk as ConcreteBackend;

    } else if #[cfg(feature = "marlin")] {
        // R1CS_MARLIN_ARKWORKS
        pub mod r1cs_marlin_arkworks;
        pub use r1cs_marlin_arkworks::Marlin as ConcreteBackend;
    } else {
        compile_error!("please specify a backend to compile with");
    }
}
// XXX: This works  because there are only two features, we want to say only one of these can be enabled. (feature xor)
#[cfg(all(feature = "plonk", feature = "marlin"))]
compile_error!("feature \"plonk\"  and feature \"marlin\" cannot be enabled at the same time");
