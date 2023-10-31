use crate::abis::function_data::FunctionData;
use crate::address::EthAddress;

struct OptionallyRevealedData {
    call_stack_item_hash : Field,
    function_data : FunctionData,
    vk_hash : Field,
    portal_contract_address : EthAddress,
    pay_fee_from_l1 : bool,
    pay_fee_from_public_l2 : bool,
    called_from_l1 : bool,
    called_from_public_l2 : bool,
}
