use crate::{hash::pedersen_hash, traits::ToField};

pub fn derive_storage_slot_in_map<K>(storage_slot: Field, key: K) -> Field where K: ToField {
    pedersen_hash([storage_slot, key.to_field()], 0)
}
