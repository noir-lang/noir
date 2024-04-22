use crate::abis::{
    call_request::CallRequest, private_call_stack_item::PrivateCallStackItem,
    membership_witness::{FunctionLeafMembershipWitness, NoteHashReadRequestMembershipWitness}
};
use crate::address::{SaltedInitializationHash, PublicKeysHash, EthAddress};
use crate::contract_class_id::ContractClassId;
use crate::mocked::{Proof, VerificationKey};
use crate::constants::{
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL
};

struct PrivateCallData {
    call_stack_item: PrivateCallStackItem,

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],

    proof: Proof,
    vk: VerificationKey,

    salted_initialization_hash: SaltedInitializationHash,
    public_keys_hash: PublicKeysHash,
    contract_class_artifact_hash: Field,
    contract_class_public_bytecode_commitment: Field,
    function_leaf_membership_witness: FunctionLeafMembershipWitness,
    
    note_hash_read_request_membership_witnesses: [NoteHashReadRequestMembershipWitness; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],

    acir_hash: Field,
}
