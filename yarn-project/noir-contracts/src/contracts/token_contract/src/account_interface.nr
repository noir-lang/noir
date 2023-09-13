
use dep::std::option::Option;

use dep::aztec::context::{
    PrivateContext,
    PublicContext,
    Context,
};

struct AccountContract {
  address: Field,
}

impl AccountContract {
  fn at(address: Field) -> Self {
    Self {
      address,
    }
  }

  // Will call is_valid(_public) on the account contract
  // Will insert the nullifier into the context
  // Will revert if the authorization fails
  fn is_valid(
    self: Self, 
    context: Context,
    message_hash: Field
  ) {
    let return_value = if context.private.is_some() {
      let context = context.private.unwrap();
      context.push_new_nullifier(message_hash, 0);
      // @todo @lherskind Call should be static but unsupported atm
      context.call_private_function(
        self.address, 
        0xe86ab4ff,
        [message_hash]
      )[0]
    } else if context.public.is_some() {
      let context = context.public.unwrap();
      context.push_new_nullifier(message_hash, 0);
      // @todo @lherskind Call should be static but unsupported atm
      (*context).call_public_function(
        self.address,
        0xf3661153,
        [message_hash]
      )[0]
    } else {
      0
    };
    assert(return_value == 0xe86ab4ff, "invalid call");
  }
}