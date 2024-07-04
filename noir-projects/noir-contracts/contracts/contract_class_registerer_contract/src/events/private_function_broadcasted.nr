use dep::aztec::prelude::FunctionSelector;
use dep::aztec::protocol_types;
use dep::aztec::protocol_types::{
    contract_class_id::ContractClassId,
    constants::{
    FUNCTION_TREE_HEIGHT, ARTIFACT_FUNCTION_TREE_MAX_HEIGHT,
    MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS,
    REGISTERER_PRIVATE_FUNCTION_BROADCASTED_MAGIC_VALUE,
    REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS
},
    traits::Serialize
};

struct InnerPrivateFunction {
    selector: FunctionSelector,
    metadata_hash: Field, 
    vk_hash: Field, 
}

impl Serialize<3> for InnerPrivateFunction {
    fn serialize(self: Self) -> [Field; 3] {
        [self.selector.to_field(), self.metadata_hash, self.vk_hash]
    }
}

struct PrivateFunction {
    selector: FunctionSelector,
    metadata_hash: Field, 
    vk_hash: Field, 
    bytecode: [Field; MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS],
}

impl Serialize<MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + 3> for PrivateFunction {
    fn serialize(self: Self) -> [Field; MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + 3] {
        let mut packed = [0; MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + 3];
        packed[0] = self.selector.to_field();
        packed[1] = self.metadata_hash;
        packed[2] = self.vk_hash;
        for i in 0..MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS {
            packed[i + 3] = self.bytecode[i];
        }
        packed
    }
}

// #[aztec(event)]
struct ClassPrivateFunctionBroadcasted {
    contract_class_id: ContractClassId,
    artifact_metadata_hash: Field,
    unconstrained_functions_artifact_tree_root: Field,
    private_function_tree_sibling_path: [Field; FUNCTION_TREE_HEIGHT],
    private_function_tree_leaf_index: Field,
    artifact_function_tree_sibling_path: [Field; ARTIFACT_FUNCTION_TREE_MAX_HEIGHT],
    artifact_function_tree_leaf_index: Field,
    function: PrivateFunction
}

impl Serialize<MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS> for ClassPrivateFunctionBroadcasted {
    fn serialize(self: Self) -> [Field; MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS] {
        let mut packed = [0; MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + REGISTERER_PRIVATE_FUNCTION_BROADCASTED_ADDITIONAL_FIELDS];
        packed[0] = REGISTERER_PRIVATE_FUNCTION_BROADCASTED_MAGIC_VALUE;
        packed[1] = self.contract_class_id.to_field();
        packed[2] = self.artifact_metadata_hash;
        packed[3] = self.unconstrained_functions_artifact_tree_root;
        for i in 0..FUNCTION_TREE_HEIGHT {
          packed[i + 4] = self.private_function_tree_sibling_path[i];
        }
        packed[4 + FUNCTION_TREE_HEIGHT] = self.private_function_tree_leaf_index;
        for i in 0..ARTIFACT_FUNCTION_TREE_MAX_HEIGHT {
          packed[i + 5 + FUNCTION_TREE_HEIGHT] = self.artifact_function_tree_sibling_path[i];
        }
        packed[5 + ARTIFACT_FUNCTION_TREE_MAX_HEIGHT + FUNCTION_TREE_HEIGHT] = self.artifact_function_tree_leaf_index;
        let packed_function = self.function.serialize();
        for i in 0..MAX_PACKED_BYTECODE_SIZE_PER_PRIVATE_FUNCTION_IN_FIELDS + 3 {
            packed[i + 6 + ARTIFACT_FUNCTION_TREE_MAX_HEIGHT + FUNCTION_TREE_HEIGHT] = packed_function[i];
        }
        packed
    }
}
