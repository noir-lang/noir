use dep::protocol_types::{
  address::{
    AztecAddress,
    PartialAddress
  }, 
  constants::{
    GENERATOR_INDEX__PUBLIC_KEYS_HASH,
    GENERATOR_INDEX__CONTRACT_ADDRESS_V1,
    CANONICAL_KEY_REGISTRY_ADDRESS
  },
  grumpkin_point::GrumpkinPoint,
};

use crate::context::PrivateContext;
use crate::hash::{
  pedersen_hash, 
  poseidon2_hash,
};
use crate::oracle::keys::get_public_keys_and_partial_address;
use crate::state_vars::{
  map::derive_storage_slot_in_map,
  shared_mutable::shared_mutable_private_getter::SharedMutablePrivateGetter,
};

struct PublicKeyTypeEnum {
  NULLIFIER: u8,
}

global PublicKeyType = PublicKeyTypeEnum {
  NULLIFIER: 0,
};

pub fn get_fresh_nullifier_public_key_hash(
  context: &mut PrivateContext,
  address: AztecAddress, 
) -> Field {
  // This is the storage slot of the nullifier_public_key inside the key registry contract
  // TODO: (#6133) We should have this be directly imported from the other contract if possible, or at least this should not be this brittle
  let storage_slot_of_nullifier_public_key = 1;

  let derived_slot = derive_storage_slot_in_map(storage_slot_of_nullifier_public_key, address);

  // We read from the canonical Key Registry
  // TODO: (#6134) It's a bit wonky because we need to know the delay for get_current_value_in_private to work correctly.
  // We should allow for this usecase without needing to hard code it here.
  let registry_private_getter: SharedMutablePrivateGetter<Field, 5> = SharedMutablePrivateGetter::new(*context, AztecAddress::from_field(CANONICAL_KEY_REGISTRY_ADDRESS), derived_slot);
  let nullifier_public_key_hash_in_registry = registry_private_getter.get_current_value_in_private();

  let nullifier_public_key_hash = if nullifier_public_key_hash_in_registry == 0 {
    let keys = get_original_public_keys_internal(address);
    poseidon2_hash(keys[PublicKeyType.NULLIFIER].serialize())
  } else {
    nullifier_public_key_hash_in_registry
  };

  nullifier_public_key_hash
}

// This constraint only works on keys that have not been rotated, otherwise this call will fail as the public keys are not constrained
fn get_original_public_keys_internal(address: AztecAddress) -> [GrumpkinPoint; 4] {
  let (public_keys, partial_address) = get_public_keys_and_partial_address(address);

  let nullifier_pub_key = public_keys[0];
  let incoming_pub_key = public_keys[1];
  let outgoing_pub_key = public_keys[2];
  let tagging_pub_key = public_keys[3];

  let computed_address = AztecAddress::compute_from_public_keys_and_partial_address(
    nullifier_pub_key,
    incoming_pub_key,
    outgoing_pub_key,
    tagging_pub_key,
    partial_address,
  );

  assert(computed_address.eq(address));

  [nullifier_pub_key, incoming_pub_key, outgoing_pub_key, tagging_pub_key]
}
