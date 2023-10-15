use acvm::BlackBoxFunctionSolver;

// This is a struct that wraps a dynamically dispatched `BlackBoxFunctionSolver`
// where we proxy the unimplemented stuff to the wrapped backend, but it
// allows us to avoid changing function signatures to include the `Box`
pub(super) struct WrapperSolver(pub(super) Box<dyn BlackBoxFunctionSolver>);

impl BlackBoxFunctionSolver for WrapperSolver {
    fn schnorr_verify(
        &self,
        public_key_x: &acvm::FieldElement,
        public_key_y: &acvm::FieldElement,
        signature: &[u8],
        message: &[u8],
    ) -> Result<bool, acvm::BlackBoxResolutionError> {
        self.0.schnorr_verify(public_key_x, public_key_y, signature, message)
    }

    fn pedersen(
        &self,
        inputs: &[acvm::FieldElement],
        domain_separator: u32,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        self.0.pedersen(inputs, domain_separator)
    }

    fn fixed_base_scalar_mul(
        &self,
        low: &acvm::FieldElement,
        high: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        self.0.fixed_base_scalar_mul(low, high)
    }
}

// We also have a mocked implementation of the `BlackBoxFunctionSolver` trait for use in tests

#[cfg(test)]
pub(crate) struct MockBackend;

#[cfg(test)]
impl BlackBoxFunctionSolver for MockBackend {
    fn schnorr_verify(
        &self,
        _public_key_x: &acvm::FieldElement,
        _public_key_y: &acvm::FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, acvm::BlackBoxResolutionError> {
        unimplemented!()
    }

    fn pedersen(
        &self,
        _inputs: &[acvm::FieldElement],
        _domain_separator: u32,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        unimplemented!()
    }

    fn fixed_base_scalar_mul(
        &self,
        _low: &acvm::FieldElement,
        _high: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        unimplemented!()
    }
}
