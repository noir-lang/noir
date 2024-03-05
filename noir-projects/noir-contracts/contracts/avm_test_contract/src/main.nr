contract AvmTest {
    // Libs
    use dep::aztec::protocol_types::{address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH};
    use dep::compressed_string::CompressedString;

    // avm lib
    use dep::aztec::avm::hash::{keccak256, poseidon, sha256};

    #[aztec(private)]
    fn constructor() {}

    // Public-vm macro will prefix avm to the function name for transpilation
    #[aztec(public-vm)]
    fn addArgsReturn(argA: Field, argB: Field) -> pub Field {
        argA + argB
    }

    #[aztec(public-vm)]
    fn setOpcodeUint8() -> pub u8 {
        8 as u8
    }

    // Bit size 16 in Noir is deprecated.
    // #[aztec(public-vm)]
    // fn setOpcodeUint16() -> pub u16 {
    //     60000 as u16
    // }

    #[aztec(public-vm)]
    fn setOpcodeUint32() -> pub u32 {
        1 << 30 as u32
    }

    #[aztec(public-vm)]
    fn setOpcodeUint64() -> pub u64 {
        1 << 60 as u64
    }

    // Can't return this since it doesn't fit in a Noir field.
    // #[aztec(public-vm)]
    // fn setOpcodeUint128() -> pub u128 {
    //     1 << 120 as u128
    // }

    // Field should fit in 128 bits
    // ACIR only supports fields of up to 126 bits!
    // Same with internal fields for unconstrained functions, apprently.
    #[aztec(public-vm)]
    fn setOpcodeSmallField() -> pub Field {
        200 as Field
    }

    // /************************************************************************
    //  * Hashing functions
    //  ************************************************************************/
    #[aztec(public-vm)]
    fn keccak_hash(data: [Field; 3]) -> pub [Field; 2] {
        keccak256(data)
    }

    #[aztec(public-vm)]
    fn poseidon_hash(data: [Field; 3]) -> pub Field {
        poseidon(data)
    }

    #[aztec(public-vm)]
    fn sha256_hash(data: [Field; 3]) -> pub [Field; 2] {
        sha256(data)
    }

    #[aztec(public-vm)]
    fn pedersen_hash(data: [Field; 3]) -> pub Field {
        dep::std::hash::pedersen_hash(data)
    }

    // /************************************************************************
    //  * AvmContext functions
    //  ************************************************************************/
    #[aztec(public-vm)]
    fn getAddress() -> pub AztecAddress {
        context.address()
    }

    #[aztec(public-vm)]
    fn getStorageAddress() -> pub AztecAddress {
        context.storage_address()
    }

    #[aztec(public-vm)]
    fn getSender() -> pub AztecAddress {
        context.sender()
    }

    #[aztec(public-vm)]
    fn getOrigin() -> pub AztecAddress {
        context.origin()
    }

    #[aztec(public-vm)]
    fn getPortal() -> pub EthAddress {
        context.portal()
    }

    #[aztec(public-vm)]
    fn getFeePerL1Gas() -> pub Field {
        context.fee_per_l1_gas()
    }

    #[aztec(public-vm)]
    fn getFeePerL2Gas() -> pub Field {
        context.fee_per_l2_gas()
    }

    #[aztec(public-vm)]
    fn getFeePerDaGas() -> pub Field {
        context.fee_per_da_gas()
    }

    #[aztec(public-vm)]
    fn getChainId() -> pub Field {
        context.chain_id()
    }

    #[aztec(public-vm)]
    fn getVersion() -> pub Field {
        context.version()
    }

    #[aztec(public-vm)]
    fn getBlockNumber() -> pub Field {
        context.block_number()
    }

    #[aztec(public-vm)]
    fn getTimestamp() -> pub Field {
        context.timestamp()
    }

    // #[aztec(public-vm)]
    // fn getContractCallDepth() -> pub Field {
    //     context.contract_call_depth()
    // }

    #[aztec(public-vm)]
    fn emit_unencrypted_log() {
        context.emit_unencrypted_log(/*event_selector=*/ 5, /*message=*/ [10, 20, 30]);
        context.emit_unencrypted_log(/*event_selector=*/ 8, /*message=*/ "Hello, world!");
        // FIXME: Try this once Brillig codegen produces uniform bit sizes for LT
        // FIXME: TagCheckError: Tag mismatch at offset 22, got UINT64, expected UINT32
        // let s: CompressedString<1,13> = CompressedString::from_string("Hello, world!");
        // context.emit_unencrypted_log(/*event_selector=*/ 10, /*message=*/ s);
    }

    #[aztec(public-vm)]
    fn note_hash_exists(note_hash: Field, leaf_index: Field) -> pub u8 {
        context.note_hash_exists(note_hash, leaf_index)
    }

    // Use the standard context interface to emit a new note hash
    #[aztec(public-vm)]
    fn new_note_hash(note_hash: Field) {
        context.push_new_note_hash(note_hash);
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public-vm)]
    fn new_nullifier(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
    }

    // Use the standard context interface to check for a nullifier
    #[aztec(public-vm)]
    fn nullifier_exists(nullifier: Field) -> pub u8 {
        context.nullifier_exists(nullifier)
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public-vm)]
    fn emit_nullifier_and_check(nullifier: Field) {
        context.emit_nullifier(nullifier);
        let exists = context.nullifier_exists(nullifier);
        assert(exists == 1, "Nullifier was just created, but its existence wasn't detected!");
    }

    // Create the same nullifier twice (shouldn't work!)
    #[aztec(public-vm)]
    fn nullifier_collision(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        // Can't do this twice!
        context.push_new_nullifier(nullifier, 0);
    }

    #[aztec(public-vm)]
    fn l1_to_l2_msg_exists(msg_hash: Field, msg_leaf_index: Field) -> pub u8 {
        context.l1_to_l2_msg_exists(msg_hash, msg_leaf_index)
    }

    #[aztec(public-vm)]
    fn send_l2_to_l1_msg(recipient: EthAddress, content: Field) {
        context.message_portal(recipient, content)
    }
}
