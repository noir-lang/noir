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

    fn pedersen_commitment(
        &self,
        inputs: &[acvm::FieldElement],
        domain_separator: u32,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        self.0.pedersen_commitment(inputs, domain_separator)
    }

    fn fixed_base_scalar_mul(
        &self,
        low: &acvm::FieldElement,
        high: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        self.0.fixed_base_scalar_mul(low, high)
    }

    fn pedersen_hash(
        &self,
        inputs: &[acvm::FieldElement],
        domain_separator: u32,
    ) -> Result<acvm::FieldElement, acvm::BlackBoxResolutionError> {
        self.0.pedersen_hash(inputs, domain_separator)
    }

    fn ec_add(
        &self,
        input1_x: &acvm::FieldElement,
        input1_y: &acvm::FieldElement,
        input2_x: &acvm::FieldElement,
        input2_y: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        self.0.ec_add(input1_x, input1_y, input2_x, input2_y)
    }

    fn ec_double(
        &self,
        input_x: &acvm::FieldElement,
        input_y: &acvm::FieldElement,
    ) -> Result<(acvm::FieldElement, acvm::FieldElement), acvm::BlackBoxResolutionError> {
        self.0.ec_double(input_x, input_y)
    }
}
