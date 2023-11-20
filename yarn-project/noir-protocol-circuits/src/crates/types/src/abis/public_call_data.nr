use dep::aztec::constants_gen::{MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL};
use crate::{
    abis::call_stack_item::PublicCallStackItem,
    address::EthAddress,
    mocked::Proof,
};

struct PublicCallData {
    call_stack_item: PublicCallStackItem,
    public_call_stack_preimages: [PublicCallStackItem; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    proof: Proof,
    portal_contract_address: EthAddress,
    bytecode_hash: Field,
}
