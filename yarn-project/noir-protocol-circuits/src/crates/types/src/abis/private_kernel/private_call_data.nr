use crate::abis::{
    call_request::CallRequest,
    call_stack_item::PrivateCallStackItem,
    membership_witness::{
        ContractLeafMembershipWitness,
        FunctionLeafMembershipWitness,
        ReadRequestMembershipWitness,
    },
};
use crate::address::EthAddress;
use crate::mocked::{Proof, VerificationKey};
use dep::aztec::constants_gen::{
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_READ_REQUESTS_PER_CALL,
};

struct PrivateCallData {
    call_stack_item: PrivateCallStackItem,

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],

    proof: Proof,
    vk: VerificationKey,

    function_leaf_membership_witness: FunctionLeafMembershipWitness,
    contract_leaf_membership_witness: ContractLeafMembershipWitness,

    read_request_membership_witnesses: [ReadRequestMembershipWitness; MAX_READ_REQUESTS_PER_CALL],

    portal_contract_address: EthAddress,
    acir_hash: Field,
}
