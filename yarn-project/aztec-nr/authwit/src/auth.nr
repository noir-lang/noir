use dep::std::hash::pedersen_with_separator;

use dep::aztec::{
  context::{PrivateContext, PublicContext, Context},
  constants_gen::{EMPTY_NULLIFIED_COMMITMENT, GENERATOR_INDEX__SIGNATURE_PAYLOAD},
  types::address::AztecAddress,
  abi::hash_args,
};

global IS_VALID_SELECTOR = 0xe86ab4ff;
global IS_VALID_PUBLIC_SELECTOR = 0xf3661153;

// @todo #2676 Should use different generator than the payload to limit probability of collisions.

// docs:start:assert_valid_authwit
// Assert that `on_behalf_of` have authorized `message_hash` with a valid authentication witness
pub fn assert_valid_authwit(context: &mut PrivateContext, on_behalf_of: AztecAddress, message_hash: Field) {
  let result = context.call_private_function(on_behalf_of.address, IS_VALID_SELECTOR, [message_hash])[0];
  context.push_new_nullifier(message_hash, EMPTY_NULLIFIED_COMMITMENT);
  assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}
// docs:end:assert_valid_authwit

// docs:start:assert_current_call_valid_authwit
// Assert that `on_behalf_of` have authorized the current call with a valid authentication witness
pub fn assert_current_call_valid_authwit(context: &mut PrivateContext, on_behalf_of: AztecAddress) {
  // message_hash = H(caller, contract_this, selector, args_hash)
  let message_hash = pedersen_with_separator(
    [context.msg_sender(), context.this_address(), context.selector(), context.args_hash],
    GENERATOR_INDEX__SIGNATURE_PAYLOAD
  )[0];
  assert_valid_authwit(context, on_behalf_of, message_hash);
}
// docs:end:assert_current_call_valid_authwit

// docs:start:assert_valid_authwit_public
// Assert that `on_behalf_of` have authorized `message_hash` in a public context
pub fn assert_valid_authwit_public(context: &mut PublicContext, on_behalf_of: AztecAddress, message_hash: Field) {
  let result = context.call_public_function(on_behalf_of.address, IS_VALID_PUBLIC_SELECTOR, [message_hash])[0];
  context.push_new_nullifier(message_hash, EMPTY_NULLIFIED_COMMITMENT);
  assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}
// docs:end:assert_valid_authwit_public

// docs:start:assert_current_call_valid_authwit_public
// Assert that `on_behalf_of` have authorized the current call in a public context
pub fn assert_current_call_valid_authwit_public(context: &mut PublicContext, on_behalf_of: AztecAddress) {
  // message_hash = H(caller, contract_this, selector, args_hash)
  let message_hash = pedersen_with_separator(
    [context.msg_sender(), context.this_address(), context.selector(), context.args_hash],
    GENERATOR_INDEX__SIGNATURE_PAYLOAD
  )[0];
  assert_valid_authwit_public(context, on_behalf_of, message_hash);
}
// docs:end:assert_current_call_valid_authwit_public

// docs:start:compute_authwit_message_hash
// Compute the message hash to be used by an authentication witness 
pub fn compute_authwit_message_hash<N>(
  caller: AztecAddress, 
  target: AztecAddress, 
  selector: Field, 
  args: [Field; N]
) -> Field {
  let args_hash = hash_args(args);
  pedersen_with_separator([caller.address, target.address, selector, args_hash], GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0]
}
// docs:end:compute_authwit_message_hash