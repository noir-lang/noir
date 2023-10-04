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

// Assert that `whom` have authorized `message_hash` with a valid authentication witness
fn assert_valid_authwit(context: &mut PrivateContext, whom: AztecAddress, message_hash: Field) {
  let result = context.call_private_function(whom.address, IS_VALID_SELECTOR, [message_hash])[0];
  context.push_new_nullifier(message_hash, EMPTY_NULLIFIED_COMMITMENT);
  assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}

// Assert that `whom` have authorized the current call with a valid authentication witness
fn assert_current_call_valid_authwit(context: &mut PrivateContext, whom: AztecAddress) {
  let args = [context.msg_sender(), context.this_address(), context.selector(), context.args_hash];
  let message_hash = pedersen_with_separator(args, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0];
  assert_valid_authwit(context, whom, message_hash);
}

// Assert that `whom` have authorized `message_hash` in a public context
fn assert_valid_authwit_public(context: &mut PublicContext, whom: AztecAddress, message_hash: Field) {
  let result = context.call_public_function(whom.address, IS_VALID_PUBLIC_SELECTOR, [message_hash])[0];
  context.push_new_nullifier(message_hash, EMPTY_NULLIFIED_COMMITMENT);
  assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}

// Assert that `whom` have authorized the current call in a public context
fn assert_current_call_valid_authwit_public(context: &mut PublicContext, whom: AztecAddress) {
  let args = [context.msg_sender(), context.this_address(), context.selector(), context.args_hash];
  let message_hash = pedersen_with_separator(args, GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0];
  assert_valid_authwit_public(context, whom, message_hash);
}

// Compute the message hash to be used by an authentication witness 
fn compute_authwit_message_hash<N>(caller: AztecAddress, target: AztecAddress, selector: Field, args: [Field; N]) -> Field {
  let args_hash = hash_args(args);
  pedersen_with_separator([caller.address, target.address, selector, args_hash], GENERATOR_INDEX__SIGNATURE_PAYLOAD)[0]
}