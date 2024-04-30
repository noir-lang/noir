// Test contract showing basic public access control that can be used in private. It uses a SharedMutable state variable to
// publicly store the address of an authorized account that can call private functions.
contract Auth {
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::aztec::state_vars::{PublicImmutable, SharedMutable};

    // Authorizing a new address has a certain block delay before it goes into effect.
    global CHANGE_AUTHORIZED_DELAY_BLOCKS = 5;

    #[aztec(storage)]
    struct Storage {
        // Admin can change the value of the authorized address via set_authorized()
        admin: PublicImmutable<AztecAddress>,
        // docs:start:shared_mutable_storage
        authorized: SharedMutable<AztecAddress, CHANGE_AUTHORIZED_DELAY_BLOCKS>, 
        // docs:end:shared_mutable_storage
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(admin: AztecAddress) {
        assert(!admin.is_zero(), "invalid admin");
        storage.admin.initialize(admin);
        // Note that we don't initialize authorized with any value: because storage defaults to 0 it'll have a 'post'
        // value of 0 and block of change 0, meaning it is effectively autoinitialized at the zero address.
    }

    // docs:start:shared_mutable_schedule
    #[aztec(public)]
    fn set_authorized(authorized: AztecAddress) {
        assert_eq(storage.admin.read(), context.msg_sender(), "caller is not admin");
        storage.authorized.schedule_value_change(authorized);
        // docs:end:shared_mutable_schedule
    }

    #[aztec(public)]
    fn get_authorized() -> AztecAddress {
        // docs:start:shared_mutable_get_current_public
        storage.authorized.get_current_value_in_public()
        // docs:end:shared_mutable_get_current_public
    }

    #[aztec(public)]
    fn get_scheduled_authorized() -> AztecAddress {
        // docs:start:shared_mutable_get_scheduled_public
        let (scheduled_value, _block_of_change): (AztecAddress, u32) = storage.authorized.get_scheduled_value_in_public();
        // docs:end:shared_mutable_get_scheduled_public
        scheduled_value
    }

    #[aztec(private)]
    fn do_private_authorized_thing() {
        // Reading a value from authorized in private automatically adds an extra validity condition: the base rollup
        // circuit will reject this tx if included in a block past the block horizon, which is as far as the circuit can
        // guarantee the value will not change from some historical value (due to CHANGE_AUTHORIZED_DELAY_BLOCKS).
        // docs:start:shared_mutable_get_current_private
        let authorized = storage.authorized.get_current_value_in_private();
        // docs:end:shared_mutable_get_current_private
        assert_eq(authorized, context.msg_sender(), "caller is not authorized");
    }
}
