
use crate::{
    traits::{Serialize, Deserialize, Empty},
    constants::{ VERIFICATION_KEY_LENGTH_IN_FIELDS },
};

global SERIALIZED_VERIFICATION_KEY_LENGTH = VERIFICATION_KEY_LENGTH_IN_FIELDS + 1;

struct VerificationKey {
    key: [Field; VERIFICATION_KEY_LENGTH_IN_FIELDS],
    hash: Field,
}

impl Serialize<SERIALIZED_VERIFICATION_KEY_LENGTH> for VerificationKey {
    fn serialize(self) -> [Field; SERIALIZED_VERIFICATION_KEY_LENGTH] {
      let mut fields = [0; SERIALIZED_VERIFICATION_KEY_LENGTH];
      for i in 0..VERIFICATION_KEY_LENGTH_IN_FIELDS {
        fields[i] = self.key[i];
      }
      fields[VERIFICATION_KEY_LENGTH_IN_FIELDS] = self.hash;
      fields
    }
}

impl Deserialize<SERIALIZED_VERIFICATION_KEY_LENGTH> for VerificationKey {
    fn deserialize(fields: [Field; SERIALIZED_VERIFICATION_KEY_LENGTH]) -> Self {
      let mut key = VerificationKey::empty();
      for i in 0..VERIFICATION_KEY_LENGTH_IN_FIELDS {
        key.key[i] = fields[i];
      }
      key.hash = fields[VERIFICATION_KEY_LENGTH_IN_FIELDS];
      key
    }
}

impl Empty for VerificationKey {
  fn empty() -> Self {
    VerificationKey {
      hash: 0,
      key: [0; VERIFICATION_KEY_LENGTH_IN_FIELDS],
    }
  }
}
