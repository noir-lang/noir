use acvm::BlackBoxFunctionSolver;

pub(crate) struct MockBackend;

impl BlackBoxFunctionSolver for MockBackend {
    fn schnorr_verify(
        &self,
        _public_key_x: &acvm::FieldElement,
        _public_key_y: &acvm::FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, acvm::BlackBoxResolutionError> {
        Err(acvm::BlackBoxResolutionError::Unsupported(acvm::acir::BlackBoxFunc::SchnorrVerify))
    }

    fn pedersen_commitment(
        &self,
        _inputs: &[acvm::FieldElement],
        _domain_separator: u32,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        Err(acvm::BlackBoxResolutionError::Unsupported(
            acvm::acir::BlackBoxFunc::PedersenCommitment,
        ))
    }

    fn pedersen_hash(
        &self,
        _inputs: &[acvm::FieldElement],
        _domain_separator: u32,
    ) -> Result<acvm::FieldElement, acvm::BlackBoxResolutionError> {
        Err(acvm::BlackBoxResolutionError::Unsupported(acvm::acir::BlackBoxFunc::PedersenHash))
    }

    fn fixed_base_scalar_mul(
        &self,
        _low: &acvm::FieldElement,
        _high: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        Err(acvm::BlackBoxResolutionError::Unsupported(
            acvm::acir::BlackBoxFunc::FixedBaseScalarMul,
        ))
    }
}
