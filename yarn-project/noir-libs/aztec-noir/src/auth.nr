use crate::context::PrivateContext;

global IS_VALID_SELECTOR = 0xe86ab4ff;

fn assert_valid_message_for(context: &mut PrivateContext, whom: Field, message_hash: Field) {
  let result = context.call_private_function(whom, IS_VALID_SELECTOR, [message_hash])[0];
  assert(result == IS_VALID_SELECTOR, "Message not authorised by account");
}