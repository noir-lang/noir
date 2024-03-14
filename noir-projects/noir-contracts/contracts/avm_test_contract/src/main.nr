use dep::aztec::protocol_types::traits::{Serialize, Deserialize};

struct Note {
    a: Field,
    b: Field,
}

impl Serialize<2> for Note {
    fn serialize(self) -> [Field; 2] {
        [self.a, self.b]
    }
}

impl Deserialize<2> for Note {
    fn deserialize(wire: [Field; 2]) -> Note {
        Note {a: wire[0], b: wire[1]}
    }
}

contract AvmTest {
    use crate::Note;

    // Libs
    use dep::aztec::prelude::Map;
    use dep::aztec::state_vars::PublicMutable;
    use dep::aztec::protocol_types::{address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH};
    use dep::aztec::protocol_types::abis::function_selector::FunctionSelector;
    use dep::aztec::protocol_types::{hash::pedersen_hash, traits::{ToField}};
    use dep::compressed_string::CompressedString;

    // avm lib
    use dep::aztec::avm::hash::{keccak256, poseidon, sha256};

    struct Storage {
        single: PublicMutable<Field>,
        list: PublicMutable<Note>,
        map: Map<AztecAddress, PublicMutable<u32>>,
    }

    #[aztec(public-vm)]
    fn setStorageSingle(a: Field) {
        storage.single.write(a);
    }

    #[aztec(public-vm)]
    fn readStorageSingle() -> pub Field {
        storage.single.read()
    }

    #[aztec(public-vm)]
    fn setReadStorageSingle(a: Field) -> pub Field {
        storage.single.write(a);
        storage.single.read()
    }

    #[aztec(public-vm)]
    fn setStorageList(a: Field, b: Field) {
        storage.list.write(Note { a, b });
    }

    #[aztec(public-vm)]
    fn readStorageList() -> pub [Field; 2] {
        let note: Note = storage.list.read();
        note.serialize()
    }

    #[aztec(public-vm)]
    fn setStorageMap(to: AztecAddress, amount: u32) -> pub Field {
        storage.map.at(to).write(amount);
        // returns storage slot for key
        pedersen_hash([storage.map.storage_slot, to.to_field()], 0)
    }

    #[aztec(public-vm)]
    fn addStorageMap(to: AztecAddress, amount: u32) -> pub Field {
        let new_balance = storage.map.at(to).read().add(amount);
        storage.map.at(to).write(new_balance);
        // returns storage slot for key
        pedersen_hash([storage.map.storage_slot, to.to_field()], 0)
    }

    #[aztec(public-vm)]
    fn readStorageMap(address: AztecAddress) -> pub u32 {
        storage.map.at(address).read()
    }

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

    #[aztec(public-vm)]
    fn addU128(a: U128, b: U128) -> pub U128 {
        a + b
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
        let s: CompressedString<2,44> = CompressedString::from_string("A long time ago, in a galaxy far far away...");
        context.emit_unencrypted_log(/*event_selector=*/ 10, /*message=*/ s);
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

    // Directly call the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn raw_nested_call_to_add(argA: Field, argB: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("avm_addArgsReturn(Field,Field)").to_field();
        let gas = [/*l1Gas*/42, /*l2Gas*/24, /*daGas*/420];

        // Nested call
        let results = context.call(gas, context.address(), [argA, argB], selector);
        let returnData: [Field; 1] = results.0;
        // this explicit size   ^ is necessary to ensure that retSize is compile-time
        // (ensure the returnData is in a HeapArray not a HeapVector)
        let success: u8 = results.1;

        assert(success == 1, "Call failed");

        let addResult = returnData[0];
        addResult
    }

    // Use the `call_public_function` wrapper to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn nested_call_to_add(argA: Field, argB: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("avm_addArgsReturn(Field,Field)");

        // Nested call using standard context interface function
        let returnData: [Field; 1] = context.call_public_function(context.address(), selector, [argA, argB]);
        // this explicit size   ^ is necessary to ensure that retSize is compile-time
        // (ensure the returnData is in a HeapArray not a HeapVector)

        let addResult = returnData[0];
        addResult
    }

    // Directly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn raw_nested_static_call_to_add(argA: Field, argB: Field) -> pub (Field, u8) {
        let selector = FunctionSelector::from_signature("avm_addArgsReturn(Field,Field)").to_field();
        let gas = [/*l1Gas*/42, /*l2Gas*/24, /*daGas*/420];

        let (resultData, success): ([Field; 1], u8) = context.call_static(gas, context.address(), [argA, argB], selector);

        (resultData[0], success)
    }

    // Directly call_static setAdmin. Should fail since it's accessing storage.
    #[aztec(public-vm)]
    fn raw_nested_static_call_to_set_admin() -> pub u8 {
        let selector = FunctionSelector::from_signature("avm_setAdmin()").to_field();
        let gas = [/*l1Gas*/42, /*l2Gas*/24, /*daGas*/420];

        let (_returnData, success): ([Field; 0], u8) = context.call_static(gas, context.address(), [], selector);

        success
    }

    // Indirectly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn nested_static_call_to_add(argA: Field, argB: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("avm_addArgsReturn(Field,Field)");

        let resultData: [Field; 1] = context.static_call_public_function(context.address(), selector, [argA, argB]);

        resultData[0]
    }

    // Indirectly call_static setAdmin. Should revert since it's accessing storage.
    #[aztec(public-vm)]
    fn nested_static_call_to_set_admin() {
        let selector = FunctionSelector::from_signature("avm_setAdmin()");

        let _resultData: [Field; 0] = context.static_call_public_function(context.address(), selector, []);
    }
}
