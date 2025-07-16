#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};

// Temporary hack, this ensure that we always use a M31 field here
// without polluting the feature flags of the `acir_field` crate.
type FieldElement = acir::acir_field::M31FieldElement;

#[derive(Default)]
// pedantic_solving: bool
pub struct M31BlackBoxSolver(pub bool);

impl BlackBoxFunctionSolver<FieldElement> for M31BlackBoxSolver {
    fn pedantic_solving(&self) -> bool {
        self.0
    }

    fn multi_scalar_mul(
        &self,
        _points: &[FieldElement],
        _scalars_lo: &[FieldElement],
        _scalars_hi: &[FieldElement],
    ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::MultiScalarMul,
            "unsupported".to_string(),
        ))
    }

    fn ec_add(
        &self,
        _input1_x: &FieldElement,
        _input1_y: &FieldElement,
        _input1_infinite: &FieldElement,
        _input2_x: &FieldElement,
        _input2_y: &FieldElement,
        _input2_infinite: &FieldElement,
    ) -> Result<(FieldElement, FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::EmbeddedCurveAdd,
            "unsupported".to_string(),
        ))
    }

    fn poseidon2_permutation(
        &self,
        _inputs: &[FieldElement],
        _len: u32,
    ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
        Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::Poseidon2Permutation,
            "unsupported".to_string(),
        ))
    }
}