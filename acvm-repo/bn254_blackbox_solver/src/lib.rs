#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use acir::FieldElement;
use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};

mod embedded_curve_ops;
mod generator;
mod pedersen;
mod poseidon2;
mod schnorr;

use ark_ec::AffineRepr;
pub use embedded_curve_ops::{embedded_curve_add, multi_scalar_mul};
pub use poseidon2::poseidon2_permutation;

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

    fn pedersen_commitment(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        let inputs: Vec<grumpkin::Fq> = inputs.iter().map(|input| input.into_repr()).collect();
        let result = pedersen::commitment::commit_native_with_index(&inputs, domain_separator);
        let res_x =
            FieldElement::from_repr(*result.x().expect("should not commit to point at infinity"));
        let res_y =
            FieldElement::from_repr(*result.y().expect("should not commit to point at infinity"));
        Ok((res_x, res_y))
    }

    fn pedersen_hash(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<FieldElement, BlackBoxResolutionError> {
        let inputs: Vec<grumpkin::Fq> = inputs.iter().map(|input| input.into_repr()).collect();
        let result = pedersen::hash::hash_with_index(&inputs, domain_separator);
        let result = FieldElement::from_repr(result);
        Ok(result)
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
