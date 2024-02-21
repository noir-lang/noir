use dep::aztec::protocol_types;
use dep::aztec::protocol_types::{
    contract_class_id::ContractClassId, abis::function_selector::FunctionSelector,
    constants::{
    ARTIFACT_FUNCTION_TREE_MAX_HEIGHT, MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS,
    REGISTERER_UNCONSTRAINED_FUNCTION_BROADCASTED_MAGIC_VALUE
},
    traits::{Serialize}
};

struct UnconstrainedFunction {
    selector: FunctionSelector,
    metadata_hash: Field, 
    bytecode: [Field; MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS],
}

impl Serialize<MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 2> for UnconstrainedFunction {
    fn serialize(self: Self) -> [Field; MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 2] {
        let mut packed = [0; MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 2];
        packed[0] = self.selector.to_field();
        packed[1] = self.metadata_hash;
        for i in 0..MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS {
            packed[i + 2] = self.bytecode[i];
        }
        packed
    }
}

// #[event]
struct ClassUnconstrainedFunctionBroadcasted {
    contract_class_id: ContractClassId,
    artifact_metadata_hash: Field,
    private_functions_artifact_tree_root: Field,
    artifact_function_tree_sibling_path: [Field; ARTIFACT_FUNCTION_TREE_MAX_HEIGHT],
    function: UnconstrainedFunction
}

impl Serialize<MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 11> for ClassUnconstrainedFunctionBroadcasted {
    fn serialize(self: Self) -> [Field; MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 11] {
        let mut packed = [0; MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 11];
        packed[0] = REGISTERER_UNCONSTRAINED_FUNCTION_BROADCASTED_MAGIC_VALUE;
        packed[1] = self.contract_class_id.to_field();
        packed[2] = self.artifact_metadata_hash;
        packed[3] = self.private_functions_artifact_tree_root;
        for i in 0..ARTIFACT_FUNCTION_TREE_MAX_HEIGHT {
          packed[i + 4] = self.artifact_function_tree_sibling_path[i];
        }
        let packed_function = self.function.serialize();
        for i in 0..MAX_PACKED_BYTECODE_SIZE_PER_UNCONSTRAINED_FUNCTION_IN_FIELDS + 2 {
            packed[i + 4 + ARTIFACT_FUNCTION_TREE_MAX_HEIGHT] = packed_function[i];
        }
        packed
    }
}
