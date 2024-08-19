use dep::protocol_types::{
    address::AztecAddress, storage::map::derive_storage_slot_in_map,
    constants::CANONICAL_KEY_REGISTRY_ADDRESS, point::Point
};

use crate::test::helpers::cheatcodes;

pub fn store_master_key(key_index: Field, address: AztecAddress, key: Point) {
    let x_coordinate_map_slot = key_index * 2 + 1;
    let y_coordinate_map_slot = x_coordinate_map_slot + 1;
    let x_coordinate_derived_slot = derive_storage_slot_in_map(x_coordinate_map_slot, address);
    let y_coordinate_derived_slot = derive_storage_slot_in_map(y_coordinate_map_slot, address);

    cheatcodes::direct_storage_write(
        CANONICAL_KEY_REGISTRY_ADDRESS,
        x_coordinate_derived_slot,
        [key.x]
    );
    cheatcodes::direct_storage_write(
        CANONICAL_KEY_REGISTRY_ADDRESS,
        y_coordinate_derived_slot,
        [key.y]
    );
}
