cfg_if::cfg_if! {
    if #[cfg(feature = "bn254")] {
        mod bn254;
        pub use bn254::*;
    } else {
        mod bn254;
        pub use bn254::*;
    }
}
