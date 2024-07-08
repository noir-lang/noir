use dep::types::{
    header::Header, KernelCircuitPublicInputs, traits::Empty,
    recursion::{verification_key::VerificationKey, proof::RecursiveProof, traits::Verifiable}
};

struct EmptyNestedCircuitPublicInputs {
    proof: RecursiveProof,
    vk: VerificationKey,
}

impl Verifiable for EmptyNestedCircuitPublicInputs {
  fn verify(self) {
    std::verify_proof(
        self.vk.key.as_slice(),
        self.proof.fields.as_slice(),
        &[],
        self.vk.hash
    );
  }
}

impl Empty for EmptyNestedCircuitPublicInputs {
    fn empty() -> Self {
        Self {
            proof: RecursiveProof::empty(),
            vk: VerificationKey::empty(),
        }
    }
}

struct PrivateKernelEmptyPrivateInputs {
    empty_nested: EmptyNestedCircuitPublicInputs,
    historical_header: Header,
    chain_id: Field,
    version: Field,
    vk_tree_root: Field,
}

impl PrivateKernelEmptyPrivateInputs {
    pub fn execute(self) -> KernelCircuitPublicInputs {
        self.empty_nested.verify();

        let mut public_inputs = KernelCircuitPublicInputs::empty();
        public_inputs.constants.historical_header = self.historical_header;
        public_inputs.constants.tx_context.chain_id = self.chain_id;
        public_inputs.constants.tx_context.version = self.version;
        public_inputs.constants.vk_tree_root = self.vk_tree_root;

        public_inputs
    }
}

impl Empty for PrivateKernelEmptyPrivateInputs {
    fn empty() -> Self {
        PrivateKernelEmptyPrivateInputs {
            empty_nested: EmptyNestedCircuitPublicInputs::empty(),
            historical_header: Header::empty(),
            chain_id: 0,
            version: 0,
            vk_tree_root: 0
        }
    }
}

mod tests {
    use crate::private_kernel_empty::{PrivateKernelEmptyPrivateInputs, EmptyNestedCircuitPublicInputs};
    use dep::types::header::Header;

    #[test]
    unconstrained fn works() {
        let private_inputs = PrivateKernelEmptyPrivateInputs {
            empty_nested: EmptyNestedCircuitPublicInputs::empty(),
            historical_header: Header::empty(),
            chain_id: 1,
            version: 2,
            vk_tree_root: 3
        };
        let public_inputs = private_inputs.execute();
        assert_eq(public_inputs.constants.tx_context.chain_id, 1);
        assert_eq(public_inputs.constants.tx_context.version, 2);
        assert_eq(public_inputs.constants.vk_tree_root, 3);
    }
}
