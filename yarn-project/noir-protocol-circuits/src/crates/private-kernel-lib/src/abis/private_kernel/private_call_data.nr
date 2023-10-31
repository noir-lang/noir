use crate::mocked::{Proof, VerificationKey};

use crate::address::{Address, EthAddress};
use crate::abis::read_request_membership_witness::ReadRequestMembershipWitness;
use dep::aztec::constants_gen::{
    MAX_READ_REQUESTS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    CONTRACT_TREE_HEIGHT,
    FUNCTION_TREE_HEIGHT,
    NOTE_HASH_TREE_HEIGHT
};
use crate::abis::call_stack_item::PrivateCallStackItem;
use crate::abis::membership_witness::{ContractLeafMembershipWitness,FunctionLeafMembershipWitness};

use crate::utils::bounded_vec::BoundedVec;

type ReadRequestMembershipWitnessPrivateData = ReadRequestMembershipWitness<NOTE_HASH_TREE_HEIGHT>;

struct PrivateCallData {
    call_stack_item : PrivateCallStackItem,

    private_call_stack_preimages : [PrivateCallStackItem; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],

    proof : Proof,
    vk : VerificationKey,

    function_leaf_membership_witness : FunctionLeafMembershipWitness,
    contract_leaf_membership_witness : ContractLeafMembershipWitness,

    read_request_membership_witnesses : [ReadRequestMembershipWitnessPrivateData; MAX_READ_REQUESTS_PER_CALL],

    portal_contract_address : EthAddress,
    acir_hash : Field,
}
