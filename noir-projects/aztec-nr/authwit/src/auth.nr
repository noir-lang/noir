use dep::aztec::protocol_types::{
    abis::function_selector::FunctionSelector, address::AztecAddress,
    constants::{GENERATOR_INDEX__AUTHWIT_INNER, GENERATOR_INDEX__AUTHWIT_OUTER, CANONICAL_AUTH_REGISTRY_ADDRESS},
    hash::pedersen_hash
};
use dep::aztec::{prelude::Deserialize, context::{PrivateContext, PublicContext, gas::GasOpts}, hash::hash_args_array};

global IS_VALID_SELECTOR = 0xabf64ad4; // 4 first bytes of keccak256("IS_VALID()")

// docs:start:assert_current_call_valid_authwit
// Assert that `on_behalf_of` have authorized the current call with a valid authentication witness
pub fn assert_current_call_valid_authwit(context: &mut PrivateContext, on_behalf_of: AztecAddress) {
    let function_selector = FunctionSelector::from_signature("spend_private_authwit(Field)");
    let inner_hash = compute_inner_authwit_hash([context.msg_sender().to_field(), context.selector().to_field(), context.args_hash]);
    let result: Field = context.call_private_function(on_behalf_of, function_selector, [inner_hash]).unpack_into();
    assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}
// docs:end:assert_current_call_valid_authwit

// docs:start:assert_current_call_valid_authwit_public
// Assert that `on_behalf_of` have authorized the current call in a public context
pub fn assert_current_call_valid_authwit_public(context: &mut PublicContext, on_behalf_of: AztecAddress) {
    let inner_hash = compute_inner_authwit_hash(
        [(*context).msg_sender().to_field(), (*context).selector().to_field(), (*context).get_args_hash()]
    );

    let result: Field = context.call_public_function(
        AztecAddress::from_field(CANONICAL_AUTH_REGISTRY_ADDRESS),
        FunctionSelector::from_signature("consume((Field),Field)"),
        [on_behalf_of.to_field(), inner_hash].as_slice(),
        GasOpts::default()
    ).deserialize_into();
    assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}
// docs:end:assert_current_call_valid_authwit_public

// docs:start:compute_call_authwit_hash
// Compute the message hash to be used by an authentication witness 
pub fn compute_call_authwit_hash<N>(
    caller: AztecAddress,
    consumer: AztecAddress,
    chain_id: Field,
    version: Field,
    selector: FunctionSelector,
    args: [Field; N]
) -> Field {
    let args_hash = hash_args_array(args);
    let inner_hash = compute_inner_authwit_hash([caller.to_field(), selector.to_field(), args_hash]);
    compute_outer_authwit_hash(consumer, chain_id, version, inner_hash)
}
// docs:end:compute_call_authwit_hash

pub fn compute_inner_authwit_hash<N>(args: [Field; N]) -> Field {
    pedersen_hash(args, GENERATOR_INDEX__AUTHWIT_INNER)
}

pub fn compute_outer_authwit_hash(
    consumer: AztecAddress,
    chain_id: Field,
    version: Field,
    inner_hash: Field
) -> Field {
    pedersen_hash(
        [
        consumer.to_field(),
        chain_id,
        version,
        inner_hash
    ],
        GENERATOR_INDEX__AUTHWIT_OUTER
    )
}

/**
 * Helper function to set the authorization status of a message hash
 * 
 * @param message_hash The hash of the message to authorize
 * @param authorize True if the message should be authorized, false if it should be revoked
 */
pub fn set_authorized(context: &mut PublicContext, message_hash: Field, authorize: bool) {
    context.call_public_function(
        AztecAddress::from_field(CANONICAL_AUTH_REGISTRY_ADDRESS),
        FunctionSelector::from_signature("set_authorized(Field,bool)"),
        [message_hash, authorize as Field].as_slice(),
        GasOpts::default()
    ).assert_empty();
}

/**
 * Helper function to reject all authwits
 *
 * @param reject True if all authwits should be rejected, false otherwise 
 */
pub fn set_reject_all(context: &mut PublicContext, reject: bool) {
    context.call_public_function(
        AztecAddress::from_field(CANONICAL_AUTH_REGISTRY_ADDRESS),
        FunctionSelector::from_signature("set_reject_all(bool)"),
        [context.this_address().to_field(), reject as Field].as_slice(),
        GasOpts::default()
    ).assert_empty();
}
