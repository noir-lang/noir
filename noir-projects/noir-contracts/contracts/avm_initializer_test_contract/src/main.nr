contract AvmInitializerTest {
    // Libs
    use dep::aztec::state_vars::PublicImmutable;
    use dep::aztec::protocol_types::address::AztecAddress;

    #[aztec(storage)]
    struct Storage {
        immutable: PublicImmutable<Field>,
    }

    /************************************************************************
     * Storage
     ************************************************************************/
    #[aztec(public-vm)]
    #[aztec(initializer)]
    fn constructor() {
        storage.immutable.initialize(42);
    }

    #[aztec(public-vm)]
    fn read_storage_immutable() -> pub Field {
        storage.immutable.read()
    }
}
