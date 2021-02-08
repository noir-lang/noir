pub mod types;

mod pairings_bn254;
mod polynomial_eval;
mod transcript;
mod turbo_plonk;

pub const fn cryptography_libraries() -> &'static str {
    concat!(
        crate::TURBOPLONK_LIBRARY!(),
        crate::TYPES_LIBRARY!(),
        crate::PAIRINGSBN254_LIBRARY!(),
        crate::POLYNOMIALEVAL_LIBRARY!(),
        crate::TRANSCRIPT_LIBRARY!()
    )
}
