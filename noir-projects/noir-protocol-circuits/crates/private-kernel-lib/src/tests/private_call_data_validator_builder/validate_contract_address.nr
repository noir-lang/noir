use crate::tests::private_call_data_validator_builder::PrivateCallDataValidatorBuilder;
use dep::types::address::AztecAddress;

#[test(should_fail_with="contract address cannot be zero")]
fn validate_contract_address_zero_storage_contract_address_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Set (storage) contract_address to 0
    builder.private_call.contract_address = AztecAddress::zero();
    builder.private_call.storage_contract_address = AztecAddress::zero();

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_function_leaf_index_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Set the leaf index of the function leaf to a wrong value (the correct value + 1).
    let leaf_index = builder.private_call.function_leaf_membership_witness.leaf_index;
    builder.private_call.function_leaf_membership_witness.leaf_index = leaf_index + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_function_leaf_sibling_path_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    // Set the first value of the sibling path to a wrong value (the correct value + 1).
    let sibling_path_0 = builder.private_call.function_leaf_membership_witness.sibling_path[0];
    builder.private_call.function_leaf_membership_witness.sibling_path[0] = sibling_path_0 + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_contract_class_preimage_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.contract_class_artifact_hash = builder.private_call.contract_class_artifact_hash + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_partial_address_preimage_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.salted_initialization_hash.inner = builder.private_call.salted_initialization_hash.inner + 1;

    builder.validate();
}

#[test(should_fail_with="computed contract address does not match expected one")]
fn validate_contract_address_incorrect_address_preimage_fails() {
    let mut builder = PrivateCallDataValidatorBuilder::new();

    builder.private_call.public_keys_hash.inner = builder.private_call.public_keys_hash.inner + 1;

    builder.validate();
}
