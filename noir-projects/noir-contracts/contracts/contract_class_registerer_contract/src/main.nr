mod events;
mod capsule;

contract ContractClassRegisterer {
    use dep::aztec::prelude::{AztecAddress, EthAddress, FunctionSelector};
    use dep::aztec::protocol_types::{
        contract_class_id::ContractClassId,
        constants::{
        ARTIFACT_FUNCTION_TREE_MAX_HEIGHT, FUNCTION_TREE_HEIGHT,
        MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS, REGISTERER_CONTRACT_CLASS_REGISTERED_MAGIC_VALUE
    },
        traits::Serialize
    };

    use crate::events::{
        class_registered::ContractClassRegistered,
        private_function_broadcasted::{ClassPrivateFunctionBroadcasted, PrivateFunction},
        unconstrained_function_broadcasted::{ClassUnconstrainedFunctionBroadcasted, UnconstrainedFunction}
    };

    use crate::capsule::pop_capsule;

    #[aztec(private)]
    fn register(artifact_hash: Field, private_functions_root: Field, public_bytecode_commitment: Field) {
        // TODO: Validate public_bytecode_commitment is the correct commitment of packed_public_bytecode
        // TODO: Validate packed_public_bytecode is legit public bytecode

        let packed_public_bytecode: [Field; MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS] = pop_capsule();

        // Compute contract class id from preimage
        let contract_class_id = ContractClassId::compute(
            artifact_hash,
            private_functions_root,
            public_bytecode_commitment
        );

        // Emit the contract class id as a nullifier to be able to prove that this class has been (not) registered
        let event = ContractClassRegistered { contract_class_id, version: 1, artifact_hash, private_functions_root, packed_public_bytecode };
        context.push_new_nullifier(contract_class_id.to_field(), 0);

        // Broadcast class info including public bytecode
        dep::aztec::oracle::debug_log::debug_log_array_with_prefix(
            "ContractClassRegistered",
            [
            contract_class_id.to_field(),
            artifact_hash,
            private_functions_root,
            public_bytecode_commitment
        ]
        );
        context.emit_unencrypted_log(event.serialize());
    }

    #[aztec(private)]
    fn broadcast_private_function(
        contract_class_id: ContractClassId,
        artifact_metadata_hash: Field,
        unconstrained_functions_artifact_tree_root: Field,
        private_function_tree_sibling_path: [Field; FUNCTION_TREE_HEIGHT],
        private_function_tree_leaf_index: Field,
        artifact_function_tree_sibling_path: [Field; ARTIFACT_FUNCTION_TREE_MAX_HEIGHT],
        artifact_function_tree_leaf_index: Field,
        function_data: PrivateFunction
    ) {
        let event = ClassPrivateFunctionBroadcasted {
            contract_class_id,
            artifact_metadata_hash,
            unconstrained_functions_artifact_tree_root,
            private_function_tree_sibling_path,
            private_function_tree_leaf_index,
            artifact_function_tree_sibling_path,
            artifact_function_tree_leaf_index,
            function: function_data
        };
        dep::aztec::oracle::debug_log::debug_log_array_with_prefix(
            "ClassPrivateFunctionBroadcasted",
            [
            contract_class_id.to_field(),
            artifact_metadata_hash,
            unconstrained_functions_artifact_tree_root,
            function_data.selector.to_field(),
            function_data.vk_hash,
            function_data.metadata_hash
        ]
        );
        context.emit_unencrypted_log(event.serialize());
    }

    #[aztec(private)]
    fn broadcast_unconstrained_function(
        contract_class_id: ContractClassId,
        artifact_metadata_hash: Field,
        private_functions_artifact_tree_root: Field,
        artifact_function_tree_sibling_path: [Field; ARTIFACT_FUNCTION_TREE_MAX_HEIGHT],
        artifact_function_tree_leaf_index: Field,
        function_data: UnconstrainedFunction
    ) {
        let event = ClassUnconstrainedFunctionBroadcasted {
            contract_class_id,
            artifact_metadata_hash,
            private_functions_artifact_tree_root,
            artifact_function_tree_sibling_path,
            artifact_function_tree_leaf_index,
            function: function_data
        };
        dep::aztec::oracle::debug_log::debug_log_array_with_prefix(
            "ClassUnconstrainedFunctionBroadcasted",
            [
            contract_class_id.to_field(),
            artifact_metadata_hash,
            private_functions_artifact_tree_root,
            function_data.selector.to_field(),
            function_data.metadata_hash
        ]
        );
        context.emit_unencrypted_log(event.serialize());
    }
}
