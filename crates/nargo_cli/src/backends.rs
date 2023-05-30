#[cfg(feature = "bb_js")]
use acvm::{
    acir::circuit::opcodes::BlackBoxFuncCall, Backend, CommonReferenceString,
    PartialWitnessGenerator, ProofSystemCompiler, SmartContract,
};

#[cfg(any(feature = "plonk_bn254", feature = "plonk_bn254_wasm"))]
pub(crate) use acvm_backend_barretenberg::Barretenberg as ConcreteBackend;

#[cfg(not(any(feature = "plonk_bn254", feature = "plonk_bn254_wasm", feature = "bb_js")))]
compile_error!("please specify a backend to compile with");

#[cfg(all(feature = "plonk_bn254", feature = "plonk_bn254_wasm"))]
compile_error!(
    "feature \"plonk_bn254\"  and feature \"plonk_bn254_wasm\" cannot be enabled at the same time"
);

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ConcreteBackend;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Errors;

impl std::fmt::Display for Errors {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Errors")
    }
}

impl std::error::Error for Errors {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Backend for ConcreteBackend {}

impl CommonReferenceString for ConcreteBackend {
    type Error = Errors;

    fn generate_common_reference_string<'life0, 'life1, 'async_trait>(
        &'life0 self,
        circuit: &'life1 acvm::acir::circuit::Circuit,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<Vec<u8>, Self::Error>> + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn update_common_reference_string<'life0, 'life1, 'async_trait>(
        &'life0 self,
        common_reference_string: Vec<u8>,
        circuit: &'life1 acvm::acir::circuit::Circuit,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<Vec<u8>, Self::Error>> + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}

impl PartialWitnessGenerator for ConcreteBackend {
    fn schnorr_verify(
        &self,
        initial_witness: &mut acvm::acir::native_types::WitnessMap,
        public_key_x: &acvm::acir::circuit::opcodes::FunctionInput,
        public_key_y: &acvm::acir::circuit::opcodes::FunctionInput,
        signature: &[acvm::acir::circuit::opcodes::FunctionInput],
        message: &[acvm::acir::circuit::opcodes::FunctionInput],
        output: &acvm::acir::native_types::Witness,
    ) -> Result<acvm::pwg::OpcodeResolution, acvm::pwg::OpcodeResolutionError> {
        todo!()
    }

    fn pedersen(
        &self,
        initial_witness: &mut acvm::acir::native_types::WitnessMap,
        inputs: &[acvm::acir::circuit::opcodes::FunctionInput],
        domain_separator: u32,
        outputs: &[acvm::acir::native_types::Witness],
    ) -> Result<acvm::pwg::OpcodeResolution, acvm::pwg::OpcodeResolutionError> {
        todo!()
    }

    fn fixed_base_scalar_mul(
        &self,
        initial_witness: &mut acvm::acir::native_types::WitnessMap,
        input: &acvm::acir::circuit::opcodes::FunctionInput,
        outputs: &[acvm::acir::native_types::Witness],
    ) -> Result<acvm::pwg::OpcodeResolution, acvm::pwg::OpcodeResolutionError> {
        todo!()
    }
}

impl ProofSystemCompiler for ConcreteBackend {
    type Error = Errors;

    fn np_language(&self) -> acvm::Language {
        acvm::Language::PLONKCSat { width: 3 }
    }

    fn get_exact_circuit_size(
        &self,
        _circuit: &acvm::acir::circuit::Circuit,
    ) -> Result<u32, Self::Error> {
        todo!()
    }

    fn supports_opcode(&self, opcode: &acvm::acir::circuit::Opcode) -> bool {
        if !matches!(
            opcode,
            acvm::acir::circuit::Opcode::BlackBoxFuncCall(
                BlackBoxFuncCall::RecursiveAggregation { .. }
            )
        ) {
            todo!()
        } else {
            true
        }
    }

    fn proof_as_fields(
        &self,
        proof: &[u8],
        public_inputs: acvm::acir::native_types::WitnessMap,
    ) -> Result<Vec<acvm::FieldElement>, Self::Error> {
        todo!()
    }

    fn vk_as_fields(
        &self,
        common_reference_string: &[u8],
        verification_key: &[u8],
    ) -> Result<(Vec<acvm::FieldElement>, acvm::FieldElement), Self::Error> {
        todo!()
    }

    fn preprocess(
        &self,
        common_reference_string: &[u8],
        circuit: &acvm::acir::circuit::Circuit,
    ) -> Result<(Vec<u8>, Vec<u8>), Self::Error> {
        todo!()
    }

    fn prove_with_pk(
        &self,
        common_reference_string: &[u8],
        circuit: &acvm::acir::circuit::Circuit,
        witness_values: acvm::acir::native_types::WitnessMap,
        proving_key: &[u8],
        is_recursive: bool,
    ) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn verify_with_vk(
        &self,
        common_reference_string: &[u8],
        proof: &[u8],
        public_inputs: acvm::acir::native_types::WitnessMap,
        circuit: &acvm::acir::circuit::Circuit,
        verification_key: &[u8],
        is_recursive: bool,
    ) -> Result<bool, Self::Error> {
        todo!()
    }
}

impl SmartContract for ConcreteBackend {
    type Error = Errors;

    fn eth_contract_from_vk(
        &self,
        common_reference_string: &[u8],
        verification_key: &[u8],
    ) -> Result<String, Self::Error> {
        todo!()
    }
}
