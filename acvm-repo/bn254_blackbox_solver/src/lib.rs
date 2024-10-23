#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};

mod embedded_curve_ops;
mod generator;
mod pedersen;
mod poseidon2;
mod schnorr;

pub use embedded_curve_ops::{embedded_curve_add, multi_scalar_mul};
pub use generator::generators::derive_generators;
pub use poseidon2::{
    field_from_hex, poseidon2_permutation, poseidon_hash, Poseidon2Config, Poseidon2Sponge,
    POSEIDON2_CONFIG,
};

// Temporary hack, this ensure that we always use a bn254 field here
// without polluting the feature flags of the `acir_field` crate.
type FieldElement = acir::acir_field::GenericFieldElement<ark_bn254::Fr>;

#[derive(Default)]
pub struct Bn254BlackBoxSolver;

impl BlackBoxFunctionSolver<FieldElement> for Bn254BlackBoxSolver {
    fn schnorr_verify(
        &self,
        public_key_x: &FieldElement,
        public_key_y: &FieldElement,
        signature: &[u8; 64],
        message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError> {
        let sig_s: [u8; 32] = signature[0..32].try_into().unwrap();
        let sig_e: [u8; 32] = signature[32..64].try_into().unwrap();
        Ok(schnorr::verify_signature(
            public_key_x.into_repr(),
            public_key_y.into_repr(),
            sig_s,
            sig_e,
            message,
        ))
    }

    fn multi_scalar_mul(
        &self,
        points: &[FieldElement],
        scalars_lo: &[FieldElement],
        scalars_hi: &[FieldElement],
    ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
        multi_scalar_mul(points, scalars_lo, scalars_hi)
    }

    fn ec_add(
        &self,
        input1_x: &FieldElement,
        input1_y: &FieldElement,
        input1_infinite: &FieldElement,
        input2_x: &FieldElement,
        input2_y: &FieldElement,
        input2_infinite: &FieldElement,
    ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
        embedded_curve_add(
            [*input1_x, *input1_y, *input1_infinite],
            [*input2_x, *input2_y, *input2_infinite],
        )
    }

    fn poseidon2_permutation(
        &self,
        inputs: &[FieldElement],
        len: u32,
    ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
        poseidon2_permutation(inputs, len)
    }
}
