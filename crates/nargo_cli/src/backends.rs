use acvm::{Backend, PartialWitnessGenerator, ProofSystemCompiler, SmartContract};
// pub(crate) use acvm_backend_barretenberg::Barretenberg as ConcreteBackend;

// #[cfg(not(any(feature = "plonk_bn254", feature = "plonk_bn254_wasm")))]
// compile_error!("please specify a backend to compile with");

// #[cfg(all(feature = "plonk_bn254", feature = "plonk_bn254_wasm"))]
// compile_error!(
//     "feature \"plonk_bn254\"  and feature \"plonk_bn254_wasm\" cannot be enabled at the same time"
// );

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

impl PartialWitnessGenerator for ConcreteBackend {
    fn solve_black_box_function_call(
        &self,
        _initial_witness: &mut std::collections::BTreeMap<
            acvm::acir::native_types::Witness,
            acvm::FieldElement,
        >,
        _func_call: &acvm::acir::circuit::opcodes::BlackBoxFuncCall,
    ) -> Result<acvm::OpcodeResolution, acvm::OpcodeResolutionError> {
        todo!("partial witness generation is not implemented")
    }
}

impl ProofSystemCompiler for ConcreteBackend {
    type Error = Errors;

    fn np_language(&self) -> acvm::Language {
        acvm::Language::PLONKCSat { width: 3 }
    }

    fn black_box_function_supported(&self, _opcode: &acvm::acir::BlackBoxFunc) -> bool {
        true
    }

    fn get_exact_circuit_size(
        &self,
        _circuit: &acvm::acir::circuit::Circuit,
    ) -> Result<u32, Self::Error> {
        todo!()
    }

    fn preprocess(
        &self,
        _circuit: &acvm::acir::circuit::Circuit,
    ) -> Result<(Vec<u8>, Vec<u8>), Self::Error> {
        Ok((Vec::new(), Vec::new()))
    }

    fn prove_with_pk(
        &self,
        _circuit: &acvm::acir::circuit::Circuit,
        _witness_values: std::collections::BTreeMap<
            acvm::acir::native_types::Witness,
            acvm::FieldElement,
        >,
        _proving_key: &[u8],
    ) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn verify_with_vk(
        &self,
        _proof: &[u8],
        _public_inputs: std::collections::BTreeMap<
            acvm::acir::native_types::Witness,
            acvm::FieldElement,
        >,
        _circuit: &acvm::acir::circuit::Circuit,
        _verification_key: &[u8],
    ) -> Result<bool, Self::Error> {
        todo!()
    }
}

impl SmartContract for ConcreteBackend {
    type Error = Errors;

    fn eth_contract_from_vk(&self, _verification_key: &[u8]) -> Result<String, Self::Error> {
        todo!()
    }
}
