use acvm::BlackBoxFunctionSolver;

// This is a struct that wraps a dynamically dispatched `BlackBoxFunctionSolver`
// where we proxy the unimplemented stuff to the wrapped backend, but it
// allows us to avoid changing function signatures to include the `Box`
pub(super) struct WrapperSolver(pub(super) Box<dyn BlackBoxFunctionSolver<acvm::FieldElement>>);

impl BlackBoxFunctionSolver<acvm::FieldElement> for WrapperSolver {
    fn schnorr_verify(
        &self,
        public_key_x: &acvm::FieldElement,
        public_key_y: &acvm::FieldElement,
        signature: &[u8; 64],
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

    fn multi_scalar_mul(
        &self,
        points: &[acvm::FieldElement],
        scalars_lo: &[acvm::FieldElement],
        scalars_hi: &[acvm::FieldElement],
    ) -> Result<
        (acvm::FieldElement, acvm::FieldElement, acvm::FieldElement),
        acvm::BlackBoxResolutionError,
    > {
        self.0.multi_scalar_mul(points, scalars_lo, scalars_hi)
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
        input1_infinite: &acvm::FieldElement,
        input2_x: &acvm::FieldElement,
        input2_y: &acvm::FieldElement,
        input2_infinite: &acvm::FieldElement,
    ) -> Result<
        (acvm::FieldElement, acvm::FieldElement, acvm::FieldElement),
        acvm::BlackBoxResolutionError,
    > {
        self.0.ec_add(input1_x, input1_y, input1_infinite, input2_x, input2_y, input2_infinite)
    }

    fn poseidon2_permutation(
        &self,
        inputs: &[acvm::FieldElement],
        len: u32,
    ) -> Result<Vec<acvm::FieldElement>, acvm::BlackBoxResolutionError> {
        self.0.poseidon2_permutation(inputs, len)
    }
}
