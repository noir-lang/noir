use dep::aztec::{
  context::{ PrivateContext, PublicContext, Context },
  selector::compute_selector,
};

struct Token {
  address: Field,
}

impl Token {
  fn at(address: Field) -> Self {
    Self { address }
  }

  fn mint_public(self: Self, context: PublicContext, to: Field, amount: Field) {
    let _return_values = context.call_public_function(
      self.address,
      compute_selector("mint_public((Field),Field)"),
      [to, amount]
    );
  }

  // docs:start:public_burn_interface
  fn burn_public(self: Self, context: PublicContext, from: Field, amount: Field, nonce: Field) {
    let _return_values = context.call_public_function(
      self.address,
      compute_selector("burn_public((Field),Field,Field)"),
      [from, amount, nonce]
    );
  }
  // docs:end:public_burn_interface

  fn mint_private(self: Self, context: PublicContext, amount: Field, secret_hash: Field) {
    let _return_values = context.call_public_function(
      self.address,
      compute_selector("mint_private(Field,Field)"),
      [amount, secret_hash]
    );
  }


  // docs:start:private_burn_interface
  fn burn(self: Self, context: &mut PrivateContext, from: Field, amount: Field, nonce: Field) {
    let _return_values = context.call_private_function(
      self.address,
      compute_selector("burn((Field),Field,Field)"),
      [from, amount, nonce]
    );
  }
  // docs:end:private_burn_interface
}
