use dep::protocol_types::{address::AztecAddress, constants::CANONICAL_KEY_REGISTRY_ADDRESS, grumpkin_point::GrumpkinPoint};
use crate::{
    context::PrivateContext,
    oracle::{
    keys::get_public_keys_and_partial_address,
    nullifier_keys::{NullifierKeys, get_nullifier_keys as get_nullifier_keys_oracle}
},
    keys::public_keys::{PublicKeys, NULLIFIER_INDEX, INCOMING_INDEX},
    state_vars::{
    map::derive_storage_slot_in_map,
    shared_mutable::shared_mutable_private_getter::SharedMutablePrivateGetter
}
};

global DELAY = 5;

// docs:start:key-getters
pub fn get_npk_m(context: &mut PrivateContext, address: AztecAddress) -> GrumpkinPoint {
    get_master_key(context, address, NULLIFIER_INDEX)
}

pub fn get_npk_m_hash(context: &mut PrivateContext, address: AztecAddress) -> Field {
    get_master_key(context, address, NULLIFIER_INDEX).hash()
}

pub fn get_ivpk_m(context: &mut PrivateContext, address: AztecAddress) -> GrumpkinPoint {
    get_master_key(context, address, INCOMING_INDEX)
}

// Commented out as it's currently not enabled in key registry
// pub fn get_ovpk_m(context: &mut PrivateContext, address: AztecAddress) -> GrumpkinPoint {
//     get_master_key(context, address, OUTGOING_INDEX)
// }
// 
// pub fn get_tpk_m(context: &mut PrivateContext, address: AztecAddress) -> GrumpkinPoint {
//     get_master_key(context, address, TAGGING_INDEX)
// }
// docs:end:key-getters

fn get_master_key(
    context: &mut PrivateContext,
    address: AztecAddress,
    key_index: Field
) -> GrumpkinPoint {
    let key = fetch_key_from_registry(context, key_index, address);
    if key.is_zero() {
        // Keys were not registered in registry yet --> fetch key from PXE
        let keys = fetch_and_constrain_keys(address);
        // Return the corresponding to index
        keys.get_key_by_index(key_index)
    } else {
        // Keys were registered --> return the key
        key
    }
}

fn fetch_key_from_registry(
    context: &mut PrivateContext,
    key_index: Field,
    address: AztecAddress
) -> GrumpkinPoint {
    let x_coordinate_map_slot = key_index * 2 + 1;
    let y_coordinate_map_slot = x_coordinate_map_slot + 1;
    let x_coordinate_derived_slot = derive_storage_slot_in_map(x_coordinate_map_slot, address);
    let y_coordinate_derived_slot = derive_storage_slot_in_map(y_coordinate_map_slot, address);

    let x_coordinate_registry: SharedMutablePrivateGetter<Field, DELAY> = SharedMutablePrivateGetter::new(
        *context,
        AztecAddress::from_field(CANONICAL_KEY_REGISTRY_ADDRESS),
        x_coordinate_derived_slot
    );
    let y_coordinate_registry: SharedMutablePrivateGetter<Field, DELAY> = SharedMutablePrivateGetter::new(
        *context,
        AztecAddress::from_field(CANONICAL_KEY_REGISTRY_ADDRESS),
        y_coordinate_derived_slot
    );
    let x_coordinate = x_coordinate_registry.get_current_value_in_private();
    let y_coordinate = y_coordinate_registry.get_current_value_in_private();

    GrumpkinPoint::new(x_coordinate, y_coordinate)
}

// Passes only when keys were not rotated - is expected to be called only when keys were not registered yet
fn fetch_and_constrain_keys(address: AztecAddress) -> PublicKeys {
    let (public_keys, partial_address) = get_public_keys_and_partial_address(address);

    let computed_address = AztecAddress::compute(public_keys.hash(), partial_address);

    assert(computed_address.eq(address));

    public_keys
}

// We get the full struct Nullifier Keys here
pub fn get_nullifier_keys(npk_m_hash: Field) -> NullifierKeys {
    let nullifier_keys = get_nullifier_keys_oracle(npk_m_hash);
    assert_eq(nullifier_keys.master_nullifier_public_key.hash(), npk_m_hash);

    nullifier_keys
}

// We are only getting the app_nullifier_secret_key here
pub fn get_nsk_app(npk_m_hash: Field) -> Field {
    get_nullifier_keys(npk_m_hash).app_nullifier_secret_key
}
