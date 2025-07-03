use std::collections::BTreeMap;

use acvm::{
    FieldElement,
    acir::{
        circuit::{Circuit, ErrorSelector, Program, brillig::BrilligBytecode},
        native_types::Witness,
    },
};
use noirc_errors::debug_info::DebugInfo;

use crate::{ErrorType, errors::SsaReport};

#[derive(Default)]
pub struct SsaProgramArtifact {
    pub program: Program<FieldElement>,
    pub debug: Vec<DebugInfo>,
    pub warnings: Vec<SsaReport>,
    pub main_input_witnesses: Vec<Witness>,
    pub main_return_witnesses: Vec<Witness>,
    pub names: Vec<String>,
    pub brillig_names: Vec<String>,
    pub error_types: BTreeMap<ErrorSelector, ErrorType>,
}

impl SsaProgramArtifact {
    pub fn new(
        unconstrained_functions: Vec<BrilligBytecode<FieldElement>>,
        error_types: BTreeMap<ErrorSelector, ErrorType>,
    ) -> Self {
        let program = Program { functions: Vec::default(), unconstrained_functions };
        Self {
            program,
            debug: Vec::default(),
            warnings: Vec::default(),
            main_input_witnesses: Vec::default(),
            main_return_witnesses: Vec::default(),
            names: Vec::default(),
            brillig_names: Vec::default(),
            error_types,
        }
    }

    pub fn add_circuit(&mut self, mut circuit_artifact: SsaCircuitArtifact, is_main: bool) {
        self.program.functions.push(circuit_artifact.circuit);
        self.debug.push(circuit_artifact.debug_info);
        self.warnings.append(&mut circuit_artifact.warnings);
        if is_main {
            self.main_input_witnesses = circuit_artifact.input_witnesses;
            self.main_return_witnesses = circuit_artifact.return_witnesses;
        }
        self.names.push(circuit_artifact.name);
        // Acir and brillig both generate new error types, so we need to merge them
        // With the ones found during ssa generation.
        self.error_types.extend(circuit_artifact.error_types);
    }

    pub fn add_warnings(&mut self, mut warnings: Vec<SsaReport>) {
        self.warnings.append(&mut warnings);
    }
}

pub struct SsaCircuitArtifact {
    pub name: String,
    pub circuit: Circuit<FieldElement>,
    pub debug_info: DebugInfo,
    pub warnings: Vec<SsaReport>,
    pub input_witnesses: Vec<Witness>,
    pub return_witnesses: Vec<Witness>,
    pub error_types: BTreeMap<ErrorSelector, ErrorType>,
}
