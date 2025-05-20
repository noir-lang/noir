use crate::traits::{Deserialize, Serialize, ToField};
use super::{address::aztec_address::AztecAddress, hash::poseidon2_hash};
use std::meta::derive;

pub global INDEXED_TAGGING_SECRET_LENGTH: u32 = 2;

#[derive(Serialize, Deserialize)]
pub struct IndexedTaggingSecret {
    app_tagging_secret: Field,
    index: u32,
}

impl IndexedTaggingSecret {
    pub fn compute_tag(self, recipient: AztecAddress) -> Field {
        poseidon2_hash(
            [self.app_tagging_secret, recipient.to_field(), self.index as Field],
        )
    }
}
