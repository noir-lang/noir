use std::collections::BTreeMap;

use acvm::{
    FieldElement,
    acir::circuit::{Circuit, ErrorSelector, Program, brillig::BrilligBytecode},
};
use noirc_errors::debug_info::DebugInfo;

use crate::{ErrorType, errors::SsaReport};

#[derive(Default)]
pub struct SsaProgramArtifact {
    pub program: Program<FieldElement>,
    pub debug: Vec<DebugInfo>,
    pub warnings: Vec<SsaReport>,
    pub names: Vec<String>,
    pub brillig_names: Vec<String>,
    pub error_types: BTreeMap<ErrorSelector, ErrorType>,
}

impl SsaProgramArtifact {
    pub fn new(
        functions: Vec<SsaCircuitArtifact>,
        brillig_names: Vec<String>,
        unconstrained_functions: Vec<BrilligBytecode<FieldElement>>,
        error_types: BTreeMap<ErrorSelector, ErrorType>,
        warnings: Vec<SsaReport>,
    ) -> Self {
        assert_eq!(brillig_names.len(), unconstrained_functions.len());
        let program = Program { functions: Vec::default(), unconstrained_functions };
        let mut this = Self {
            program,
            debug: Vec::default(),
            warnings,
            names: Vec::default(),
            brillig_names,
            error_types,
        };

        for function in functions {
            this.add_circuit(function);
        }

        this
    }

    fn add_circuit(&mut self, mut circuit_artifact: SsaCircuitArtifact) {
        self.program.functions.push(circuit_artifact.circuit);
        self.debug.push(circuit_artifact.debug_info);
        self.warnings.append(&mut circuit_artifact.warnings);
        self.names.push(circuit_artifact.name);
        // Acir and brillig both generate new error types, so we need to merge them
        // With the ones found during ssa generation.
        self.error_types.extend(circuit_artifact.error_types);
    }
}

pub struct SsaCircuitArtifact {
    pub name: String,
    pub circuit: Circuit<FieldElement>,
    pub debug_info: DebugInfo,
    pub warnings: Vec<SsaReport>,
    pub error_types: BTreeMap<ErrorSelector, ErrorType>,
}
