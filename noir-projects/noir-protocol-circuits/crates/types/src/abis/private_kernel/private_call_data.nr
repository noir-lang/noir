use crate::address::{SaltedInitializationHash, PublicKeysHash, EthAddress};
use crate::contract_class_id::ContractClassId;
use crate::recursion::{verification_key::VerificationKey, proof::RecursiveProof};
use crate::abis::{
    call_request::CallRequest, private_call_stack_item::PrivateCallStackItem,
    private_circuit_public_inputs::PrivateCircuitPublicInputs
};
use crate::constants::{MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, FUNCTION_TREE_HEIGHT};
use crate::merkle_tree::membership::MembershipWitness;

struct PrivateCallData {
    call_stack_item: PrivateCallStackItem,

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    public_teardown_call_request: CallRequest,

    proof: RecursiveProof,
    vk: VerificationKey,

    salted_initialization_hash: SaltedInitializationHash,
    public_keys_hash: PublicKeysHash,
    contract_class_artifact_hash: Field,
    contract_class_public_bytecode_commitment: Field,
    function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
    
    acir_hash: Field,
}

fn verify_private_call(call: PrivateCallData) {
    let inputs = PrivateCircuitPublicInputs::serialize(call.call_stack_item.public_inputs);
    dep::std::verify_proof(
        call.vk.key.as_slice(),
        call.proof.fields.as_slice(),
        inputs.as_slice(),
        call.vk.hash
    );
}
