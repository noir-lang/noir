use dep::protocol_types::{
    abis::{
        block_header::BlockHeader,
        call_context::CallContext,
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        public_circuit_public_inputs::PublicCircuitPublicInputs,
    },
    contrakt::deployment_data::ContractDeploymentData,
    hash::hash_args,
};

// docs:start:private-global-variables
struct PrivateGlobalVariables {
    chain_id: Field,
    version: Field,
}
// docs:end:private-global-variables

impl PrivateGlobalVariables {
    fn serialize(self) -> [Field; 2] {
        [self.chain_id, self.version]
    }
}

// docs:start:public-global-variables
struct PublicGlobalVariables {
    chain_id: Field,
    version: Field,
    block_number: Field,
    timestamp: Field,
}
// docs:end:public-global-variables

impl PublicGlobalVariables {
    fn serialize(self) -> [Field; 4] {
        [self.chain_id, self.version, self.block_number, self.timestamp]
    }
}

// PrivateContextInputs are expected to be provided to each private function
// docs:start:private-context-inputs
struct PrivateContextInputs {
    call_context : CallContext,
    block_header: BlockHeader,
    contract_deployment_data: ContractDeploymentData,
    private_global_variables: PrivateGlobalVariables,
}
// docs:end:private-context-inputs

// PublicContextInputs are expected to be provided to each public function
// docs:start:public-context-inputs
struct PublicContextInputs {
    call_context: CallContext,
    block_header: BlockHeader,

    public_global_variables: PublicGlobalVariables,
}
// docs:end:public-context-inputs

struct Hasher {
    fields: [Field],
}

impl Hasher {
    pub fn new()-> Self {
        Self { fields: [] }
    }

    pub fn add(&mut self, field: Field) {
        self.fields = self.fields.push_back(field);
    }

    pub fn add_multiple<N>(&mut self, fields: [Field; N]) {
        for i in 0..N {
            self.fields = self.fields.push_back(fields[i]);
        }
    }

    pub fn hash(self) -> Field {
        hash_args(self.fields)
    }
}
