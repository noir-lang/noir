use crate::{abis::{call_request::CallRequest, public_call_stack_item::PublicCallStackItem}, address::EthAddress};
use crate::constants::MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL;
// Mocked here as the only remaining non-recursive proof
struct Proof {}

struct PublicCallData {
    call_stack_item: PublicCallStackItem,
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    proof: Proof,
    bytecode_hash: Field,
}
