use std::collections::HashMap;

use acvm::acir::circuit::Opcode;
use acvm::brillig_vm::brillig::Opcode as BrilligOpcode;
use noirc_artifacts::program::ProgramArtifact;
pub type Branch = (usize, usize);
pub type BranchToFeatureMap = HashMap<Branch, usize>;
pub fn analyze_brillig_program_before_fuzzing(program: &ProgramArtifact) -> BranchToFeatureMap {
    let program_bytecode = &program.bytecode;
    let main_function = &program_bytecode.functions[0];
    let starting_opcode = &main_function.opcodes[0];
    let fuzzed_brillig_function_id = match starting_opcode {
        Opcode::BrilligCall { id, .. } => id,
        _ => panic!(
            "If a method is compiled to brillig, the first opcode in ACIR has to be brillig call"
        ),
    };
    let fuzzed_brillig_function =
        &program_bytecode.unconstrained_functions[fuzzed_brillig_function_id.as_usize()];
    let mut location_to_feature_map = HashMap::new();
    let mut total_features = 0usize;
    for (opcode_index, opcode) in fuzzed_brillig_function.bytecode.iter().enumerate() {
        match opcode {
            &BrilligOpcode::JumpIf { location, .. }
            | &BrilligOpcode::JumpIfNot { location, .. } => {
                location_to_feature_map.insert((opcode_index, location), total_features);
                location_to_feature_map
                    .insert((opcode_index, opcode_index + 1), total_features + 1);
                total_features += 2;
            }
            _ => (),
        }
    }
    location_to_feature_map
}
