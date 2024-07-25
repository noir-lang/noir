use crate::{
    abis::{
    private_call_stack_item::PrivateCallStackItem,
    private_circuit_public_inputs::PrivateCircuitPublicInputs
},
    address::{SaltedInitializationHash, PublicKeysHash, EthAddress}, constants::FUNCTION_TREE_HEIGHT,
    contract_class_id::ContractClassId, merkle_tree::membership::MembershipWitness,
    recursion::{verification_key::VerificationKey, proof::RecursiveProof, traits::Verifiable}
};

struct PrivateCallData {
    call_stack_item: PrivateCallStackItem,

    vk: VerificationKey,

    salted_initialization_hash: SaltedInitializationHash,
    public_keys_hash: PublicKeysHash,
    contract_class_artifact_hash: Field,
    contract_class_public_bytecode_commitment: Field,
    function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,

    acir_hash: Field,
}
