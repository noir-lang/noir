contract KeyRegistry {
    use dep::authwit::auth::assert_current_call_valid_authwit_public;

    use dep::aztec::{
        keys::PublicKeys, state_vars::{SharedMutable, Map},
        protocol_types::{point::Point, address::{AztecAddress, PartialAddress}}
    };

    global KEY_ROTATION_DELAY = 5;

    #[aztec(storage)]
  struct Storage {
    // The following stores a hash of individual master public keys
    // If you change slots of vars below, you must update the slots in `SharedMutablePrivateGetter` in aztec-nr/keys.
    // We store x and y coordinates in individual shared mutables as shared mutable currently supports only 1 field
    npk_m_x_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
    npk_m_y_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,

    ivpk_m_x_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
    ivpk_m_y_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
    
    ovpk_m_x_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
    ovpk_m_y_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
    
    tpk_m_x_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
    tpk_m_y_registry: Map<AztecAddress, SharedMutable<Field, KEY_ROTATION_DELAY>>,
  }

    #[aztec(public)]
  fn rotate_npk_m(address: AztecAddress, new_npk_m: Point, nonce: Field) {
        // TODO: (#6137)
        if (!address.eq(context.msg_sender())) {
            assert_current_call_valid_authwit_public(&mut context, address);
        } else {
            assert(nonce == 0, "invalid nonce");
        }

        let npk_m_x_registry = storage.npk_m_x_registry.at(address);
        let npk_m_y_registry = storage.npk_m_y_registry.at(address);
        npk_m_x_registry.schedule_value_change(new_npk_m.x);
        npk_m_y_registry.schedule_value_change(new_npk_m.y);
    }

    // We need to have two separate register functions because a single one would produce too many storage writes, since
    // each SharedMutable.schedule_value_change call results in 5 writes (pre, post, block_of_change, delay and hash),
    // totaling 40 writes, while the kernels only accept up to 32 writes.
    // Once SharedMutable accepts multi-field values, we can have a single state variable hold all keys, and that way
    // also have a single block of change, hash, and delay.
    // TODO (#5491): make this be a single function with a single schedule call.

    #[aztec(public)]
  fn register_npk_and_ivpk(address: AztecAddress, partial_address: PartialAddress, keys: PublicKeys) {
        let computed_address = AztecAddress::compute(keys.hash(), partial_address);

        assert(computed_address.eq(address), "Computed address does not match supplied address");

        let npk_m_x_registry = storage.npk_m_x_registry.at(address);
        let npk_m_y_registry = storage.npk_m_y_registry.at(address);
        let ivpk_m_x_registry = storage.ivpk_m_x_registry.at(address);
        let ivpk_m_y_registry = storage.ivpk_m_y_registry.at(address);

        npk_m_x_registry.schedule_value_change(keys.npk_m.x);
        npk_m_y_registry.schedule_value_change(keys.npk_m.y);
        ivpk_m_x_registry.schedule_value_change(keys.ivpk_m.x);
        ivpk_m_y_registry.schedule_value_change(keys.ivpk_m.y);
    }

    #[aztec(public)]
  fn register_ovpk_and_tpk(address: AztecAddress, partial_address: PartialAddress, keys: PublicKeys) {
        let computed_address = AztecAddress::compute(keys.hash(), partial_address);

        assert(computed_address.eq(address), "Computed address does not match supplied address");

        let ovpk_m_x_registry = storage.ovpk_m_x_registry.at(address);
        let ovpk_m_y_registry = storage.ovpk_m_y_registry.at(address);
        let tpk_m_x_registry = storage.tpk_m_x_registry.at(address);
        let tpk_m_y_registry = storage.tpk_m_y_registry.at(address);

        ovpk_m_x_registry.schedule_value_change(keys.ovpk_m.x);
        ovpk_m_y_registry.schedule_value_change(keys.ovpk_m.y);
        tpk_m_x_registry.schedule_value_change(keys.tpk_m.x);
        tpk_m_y_registry.schedule_value_change(keys.tpk_m.y);
    }
}
