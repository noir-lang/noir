struct AztecAddress {
  address: Field
}

impl AztecAddress {
  fn new(address: Field) -> Self {
    Self {
      address
    }
  }

  fn serialize(self: Self) -> [Field; 1] {
    [self.address]
  }

  fn deserialize(fields: [Field; 1]) -> Self {
    Self {
      address: fields[0]
    }
  }
}

struct EthereumAddress {
  address: Field
}

impl EthereumAddress {
  fn new(address: Field) -> Self {
    // Check that it actually will fit. Spending a lot of constraints here :grimacing:
    let bytes = address.to_be_bytes(32);
    for i in 0..12 {
        assert(bytes[i] == 0, "Value too large for an ethereum address");
    }
    Self {
      address
    }
  }


  fn serialize(self: Self) -> [Field; 1] {
    [self.address]
  }

  fn deserialize(fields: [Field; 1]) -> Self {
    Self {
      address: fields[0]
    }
  }
}