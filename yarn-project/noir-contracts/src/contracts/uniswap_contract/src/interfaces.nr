use dep::aztec::{
  context::{ PrivateContext, PublicContext, Context },
  oracle::compute_selector::compute_selector,
  types::address::AztecAddress,
};

struct Token {
  address: Field,
}

impl Token {
  fn at(address: Field) -> Self {
    Self { address }
  }

  fn transfer_public(self: Self, context: PublicContext, from: Field, to: Field, amount: Field, nonce: Field) {
    let _transfer_return_values = context.call_public_function(
      self.address,
      compute_selector("transfer_public((Field),(Field),Field,Field)"),
      [from, to, amount, nonce]
    );
  }

  fn unshield(self: Self, context: &mut PrivateContext, from: Field, to: Field,  amount: Field, nonce: Field) {
    let _return_values = context.call_private_function(
      self.address,
      compute_selector("unshield((Field),(Field),Field,Field)"),
      [from, to, amount, nonce]
    );
  }
}

struct TokenBridge {
  address: Field,
}

impl TokenBridge {
  fn at(address: Field) -> Self {
    Self { address }
  }

  fn token(self: Self, context: PublicContext) -> Field {
    let return_values = context.call_public_function(self.address, compute_selector("get_token()"), []);
    return_values[0]
  }

  fn claim_public(self: Self, context: PublicContext, to: Field, amount: Field, canceller: Field, msg_key: Field, secret: Field) {
    let _return_values = context.call_public_function(
      self.address,
      compute_selector("claim_public((Field),Field,Field,Field,Field)"),
      [to, amount, canceller, msg_key, secret]
    );
  }

  fn exit_to_l1_public(self: Self, context: PublicContext, recipient: Field, amount: Field, callerOnL1: Field, nonce: Field) {
    let _return_values = context.call_public_function(
      self.address,
      compute_selector("exit_to_l1_public((Field),Field,(Field),Field)"),
      [recipient, amount, callerOnL1, nonce]
    );
  }
}
