use acvm::BlackBoxFunctionSolver;

pub struct MockBackend;

impl BlackBoxFunctionSolver for MockBackend {
    fn schnorr_verify(
        &self,
        _public_key_x: &acvm::FieldElement,
        _public_key_y: &acvm::FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, acvm::BlackBoxResolutionError> {
        unimplemented!(
            "schnorr_verify blackbox function is not currently supported in the Wasm LSP"
        )
    }

    fn pedersen(
        &self,
        _inputs: &[acvm::FieldElement],
        _domain_separator: u32,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        unimplemented!("pedersen blackbox function is not currently supported in the Wasm LSP")
    }

    fn fixed_base_scalar_mul(
        &self,
        _low: &acvm::FieldElement,
        _high: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        unimplemented!(
            "fixed_base_scalar_mul blackbox function is not currently supported in the Wasm LSP"
        )
    }
}
