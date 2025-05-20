use dep::types::{
    abis::private_kernel::private_call_data::PrivateCallData,
    address::AztecAddress,
    constants::{
        DEFAULT_UPDATE_DELAY, DEPLOYER_CONTRACT_ADDRESS, MAX_PROTOCOL_CONTRACTS,
        UPDATED_CLASS_IDS_SLOT,
    },
    contract_class_id::ContractClassId,
    hash::private_functions_root_from_siblings,
    merkle_tree::conditionally_assert_check_membership,
    shared_mutable::{
        shared_mutable_values::SharedMutableValues, with_hash::validate_with_hash_hints,
    },
    storage::map::derive_storage_slot_in_map,
    traits::{is_empty, Packable, ToField},
};

pub fn validate_contract_address(
    private_call_data: PrivateCallData,
    protocol_contract_tree_root: Field,
) {
    let contract_address = private_call_data.public_inputs.call_context.contract_address;
    assert(!contract_address.is_zero(), "contract address cannot be zero");

    private_call_data.vk.check_hash();

    let hints = private_call_data.verification_key_hints;

    let private_functions_root = private_functions_root_from_siblings(
        private_call_data.public_inputs.call_context.function_selector,
        private_call_data.vk.hash,
        hints.function_leaf_membership_witness.leaf_index,
        hints.function_leaf_membership_witness.sibling_path,
    );

    let contract_class_id = ContractClassId::compute(
        hints.contract_class_artifact_hash,
        private_functions_root,
        hints.contract_class_public_bytecode_commitment,
    );

    let computed_address = AztecAddress::compute_from_class_id(
        contract_class_id,
        hints.salted_initialization_hash,
        hints.public_keys,
    );

    let shared_mutable_values: SharedMutableValues<ContractClassId, DEFAULT_UPDATE_DELAY> =
        Packable::unpack(hints.updated_class_id_shared_mutable_values);

    // A block horizon for this shared mutable should be set separately when generating/validating kernel output
    validate_with_hash_hints(
        private_call_data.public_inputs.historical_header,
        derive_storage_slot_in_map(UPDATED_CLASS_IDS_SLOT as Field, contract_address),
        DEPLOYER_CONTRACT_ADDRESS,
        shared_mutable_values,
        hints.updated_class_id_witness,
        hints.updated_class_id_leaf,
    );

    let updated_contract_class_id = shared_mutable_values.svc.get_current_at(
        private_call_data.public_inputs.historical_header.global_variables.block_number as u32,
    );

    let contract_address_field = contract_address.to_field();
    let is_protocol_contract = contract_address_field.lt(MAX_PROTOCOL_CONTRACTS as Field);
    let is_updated_contract = !is_empty(updated_contract_class_id);

    let address_derivation_check =
        !is_protocol_contract & computed_address.eq(contract_address) & !is_updated_contract;
    let updated_class_check = !is_protocol_contract
        & is_updated_contract
        & contract_class_id.eq(updated_contract_class_id);
    let protocol_contract_check = is_protocol_contract
        & hints.protocol_contract_membership_witness.leaf_index.eq(contract_address_field)
        & !is_updated_contract;

    // We can have a normal contract address, which must match the calculated address, an updated contract so the class id used must be the updated one, or
    // A computed protocol contract address which exists at the index held in call_context.contract_address.
    assert(
        address_derivation_check | updated_class_check | protocol_contract_check,
        "computed contract address does not match expected one",
    );

    // A non-protocol computed contract address is checked for non-membership below using protocol_contract_leaf as a low leaf.
    // A protocol contract address is checked for membership below where protocol_contract_leaf contains the
    // computed_address at the index given by contract_address.
    conditionally_assert_check_membership(
        computed_address.to_field(),
        is_protocol_contract,
        hints.protocol_contract_leaf,
        hints.protocol_contract_membership_witness,
        protocol_contract_tree_root,
    );
}
