use dep::protocol_types::{address::AztecAddress, constants::CANONICAL_KEY_REGISTRY_ADDRESS, grumpkin_point::GrumpkinPoint};
use crate::{
    context::PrivateContext, oracle::keys::get_public_keys_and_partial_address,
    state_vars::{
    map::derive_storage_slot_in_map,
    shared_mutable::shared_mutable_private_getter::SharedMutablePrivateGetter
}
};

// Note: In fetch_key_from_registry we expect that the shared mutable slot is index * 2 + 1 for the x coordinate and
// index * 2 + 2 for the y coordinate. For example, the npk_m x coordinates will be stored in a map at storage slot
// 0 * 2 + 1 = 1, and the npk_m y coordinates at 2 * 2 + 2 = 6. If this changes the function will need to be
// refactored.
global NULLIFIER_INDEX = 0;
global INCOMING_INDEX = 1;
global OUTGOING_INDEX = 2;
global TAGGING_INDEX = 3;

global DELAY = 5;

pub fn get_npk_m(context: &mut PrivateContext, address: AztecAddress) -> GrumpkinPoint {
    get_master_key(context, address, NULLIFIER_INDEX)
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

fn get_master_key(
    context: &mut PrivateContext,
    address: AztecAddress,
    key_index: Field
) -> GrumpkinPoint {
    let key = fetch_key_from_registry(context, key_index, address);
    if key.is_zero() {
        // Keys were not registered in registry yet --> fetch key from PXE
        fetch_and_constrain_keys(address)[key_index]
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
fn fetch_and_constrain_keys(address: AztecAddress) -> [GrumpkinPoint; 4] {
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
        partial_address
    );

    assert(computed_address.eq(address));

    [nullifier_pub_key, incoming_pub_key, outgoing_pub_key, tagging_pub_key]
}
