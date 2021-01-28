/// XXX:
// FIXME:
// Why is this in it's own crate?
// Ideally this should be in the aztec_backend crate, as this is only needed for the Aztec backend.
// 
// The problem is that ACIR does not have a generic module to compute ECC operations.
// The merkle path and root for aztec is done using Pedersen.
// We rely on the WASM module to compute the PedersenHash for us
// Hence, we need (unfortunately) for now, that ACIR depends on the barretenberg backend
//


mod contract;
pub mod barretenberg_rs;

pub use barretenberg_rs::*;