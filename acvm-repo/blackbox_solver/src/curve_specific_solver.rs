use acir::{BlackBoxFunc, FieldElement};

use crate::BlackBoxResolutionError;

/// This component will generate outputs for Blackbox function calls where the underlying [`acir::BlackBoxFunc`]
/// doesn't have a canonical Rust implementation.
///
/// Returns an [`BlackBoxResolutionError`] if the backend does not support the given [`acir::BlackBoxFunc`].
pub trait BlackBoxFunctionSolver {
    fn schnorr_verify(
        &self,
        public_key_x: &FieldElement,
        public_key_y: &FieldElement,
        signature: &[u8],
        message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError>;
    fn pedersen_commitment(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError>;
    fn pedersen_hash(
        &self,
        inputs: &[FieldElement],
        domain_separator: u32,
    ) -> Result<FieldElement, BlackBoxResolutionError>;
    fn fixed_base_scalar_mul(
        &self,
        low: &FieldElement,
        high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError>;
    fn ec_add(
        &self,
        input1_x: &FieldElement,
        input1_y: &FieldElement,
        input2_x: &FieldElement,
        input2_y: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError>;
}

pub struct StubbedBlackBoxSolver;

impl StubbedBlackBoxSolver {
    fn fail(black_box_function: BlackBoxFunc) -> BlackBoxResolutionError {
        BlackBoxResolutionError::Failed(
            black_box_function,
            format!("{} is not supported", black_box_function.name()),
        )
    }
}

impl BlackBoxFunctionSolver for StubbedBlackBoxSolver {
    fn schnorr_verify(
        &self,
        _public_key_x: &FieldElement,
        _public_key_y: &FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::SchnorrVerify))
    }
    fn pedersen_commitment(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::PedersenCommitment))
    }
    fn pedersen_hash(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<FieldElement, BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::PedersenHash))
    }
    fn fixed_base_scalar_mul(
        &self,
        _low: &FieldElement,
        _high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::FixedBaseScalarMul))
    }
    fn ec_add(
        &self,
        _input1_x: &FieldElement,
        _input1_y: &FieldElement,
        _input2_x: &FieldElement,
        _input2_y: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(Self::fail(BlackBoxFunc::EmbeddedCurveAdd))
    }
}
