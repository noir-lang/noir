// Test contract showing basic public access control that can be used in private. It uses a SharedMutable state variable to
// publicly store the address of an authorized account that can call private functions.
contract Auth {
    use dep::aztec::protocol_types::address::AztecAddress;
    use dep::aztec::{
        log::{emit_unencrypted_log, emit_unencrypted_log_from_private},
        state_vars::{PublicImmutable, SharedMutable}
    };

    // Authorizing a new address has a certain block delay before it goes into effect.
    global CHANGE_AUTHORIZED_DELAY_BLOCKS = 5;

    #[aztec(storage)]
    struct Storage {
        // Admin can change the value of the authorized address via set_authorized()
        admin: PublicImmutable<AztecAddress>,
        authorized: SharedMutable<AztecAddress, CHANGE_AUTHORIZED_DELAY_BLOCKS>, 
    }

    #[aztec(public)]
    #[aztec(initializer)]
    fn constructor(admin: AztecAddress) {
        assert(!admin.is_zero(), "invalid admin");
        storage.admin.initialize(admin);
        // Note that we don't initialize authorized with any value: because storage defaults to 0 it'll have a 'post'
        // value of 0 and block of change 0, meaning it is effectively autoinitialized at the zero adddress.
    }

    #[aztec(public)]
    fn set_authorized(authorized: AztecAddress) {
        assert_eq(storage.admin.read(), context.msg_sender(), "caller is not admin");
        storage.authorized.schedule_value_change(authorized);
    }

    #[aztec(public)]
    fn get_authorized() {
        // We emit logs because we cannot otherwise return these values
        emit_unencrypted_log(
            &mut context,
            storage.authorized.get_current_value_in_public()
        );
    }

    #[aztec(public)]
    fn get_scheduled_authorized() {
        // We emit logs because we cannot otherwise return these values
        emit_unencrypted_log(
            &mut context,
            storage.authorized.get_scheduled_value_in_public().0
        );
    }

    #[aztec(private)]
    fn do_private_authorized_thing(value: Field) {
        // Reading a value from authorized in private automatically adds an extra validity condition: the base rollup
        // circuit will reject this tx if included in a block past the block horizon, which is as far as the circuit can
        // guarantee the value will not change from some historical value (due to CHANGE_AUTHORIZED_DELAY_BLOCKS).
        let authorized = storage.authorized.get_current_value_in_private();
        assert_eq(authorized, context.msg_sender(), "caller is not authorized");

        // We emit logs because we cannot otherwise return these values
        emit_unencrypted_log_from_private(&mut context, value);
    }
}
