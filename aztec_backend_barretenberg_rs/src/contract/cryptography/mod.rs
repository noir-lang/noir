pub mod types;

mod turbo_plonk;
mod pairings_bn254;
mod polynomial_eval;
mod transcript;

pub const fn cryptography_libraries() -> &'static str {
    concat!(crate::TURBOPLONK_LIBRARY!(), crate::TYPES_LIBRARY!(),crate::PAIRINGSBN254_LIBRARY!(),crate::POLYNOMIALEVAL_LIBRARY!(),crate::TRANSCRIPT_LIBRARY!())
}