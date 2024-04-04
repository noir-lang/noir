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

    global big_field_128_bits: Field = 0x001234567890abcdef1234567890abcdef;
    global big_field_136_bits: Field = 0x991234567890abcdef1234567890abcdef;

    // Libs
    use dep::aztec::prelude::Map;
    use dep::aztec::state_vars::{PublicImmutable, PublicMutable};
    use dep::aztec::protocol_types::{
        address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH,
        contract_instance::ContractInstance
    };
    use dep::aztec::oracle::get_contract_instance::{get_contract_instance_avm, get_contract_instance_internal_avm};
    use dep::aztec::protocol_types::abis::function_selector::FunctionSelector;
    use dep::aztec::protocol_types::traits::ToField;
    use dep::aztec::protocol_types::constants::RETURN_VALUES_LENGTH;
    use dep::compressed_string::CompressedString;

    // avm lib
    use dep::aztec::avm::hash::{keccak256, poseidon, sha256};

    struct Storage {
        single: PublicMutable<Field>,
        list: PublicMutable<Note>,
        map: Map<AztecAddress, PublicMutable<u32>>,
        immutable: PublicImmutable<Field>,
    }

    /************************************************************************
     * Storage
     ************************************************************************/
    // FIX: calls unsupported getNullifierMembershipWitness.
    // #[aztec(public-vm)]
    // #[aztec(initializer)]
    // fn constructor() {
    //     storage.immutable.initialize(42);
    // }

    unconstrained fn view_storage_immutable() -> pub Field {
        storage.immutable.read()
    }

    unconstrained fn view_storage_single() -> pub Field {
        storage.single.read()
    }

    unconstrained fn view_storage_list() -> pub [Field; 2] {
        storage.list.read().serialize()
    }

    unconstrained fn view_storage_map(address: AztecAddress) -> pub u32 {
        storage.map.at(address).read()
    }

    #[aztec(public-vm)]
    fn set_storage_single(a: Field) {
        storage.single.write(a);
    }

    #[aztec(public-vm)]
    fn read_storage_single() -> pub Field {
        storage.single.read()
    }

    #[aztec(public-vm)]
    fn set_read_storage_single(a: Field) -> pub Field {
        storage.single.write(a);
        storage.single.read()
    }

    #[aztec(public-vm)]
    fn set_storage_list(a: Field, b: Field) {
        storage.list.write(Note { a, b });
    }

    #[aztec(public-vm)]
    fn read_storage_list() -> pub [Field; 2] {
        let note: Note = storage.list.read();
        note.serialize()
    }

    #[aztec(public-vm)]
    fn set_storage_map(to: AztecAddress, amount: u32) -> pub Field {
        storage.map.at(to).write(amount);
        // returns storage slot for key
        dep::std::hash::pedersen_hash([storage.map.storage_slot, to.to_field()])
    }

    #[aztec(public-vm)]
    fn add_storage_map(to: AztecAddress, amount: u32) -> pub Field {
        let new_balance = storage.map.at(to).read().add(amount);
        storage.map.at(to).write(new_balance);
        // returns storage slot for key
        dep::std::hash::pedersen_hash([storage.map.storage_slot, to.to_field()])
    }

    #[aztec(public-vm)]
    fn read_storage_map(address: AztecAddress) -> pub u32 {
        storage.map.at(address).read()
    }

    #[aztec(public-vm)]
    fn add_args_return(arg_a: Field, arg_b: Field) -> pub Field {
        arg_a + arg_b
    }

    /************************************************************************
     * General Opcodes
     ************************************************************************/
    #[aztec(public-vm)]
    fn set_opcode_u8() -> pub u8 {
        8 as u8
    }

    #[aztec(public-vm)]
    fn set_opcode_u32() -> pub u32 {
        1 << 30 as u32
    }

    #[aztec(public-vm)]
    fn set_opcode_u64() -> pub u64 {
        1 << 60 as u64
    }

    #[aztec(public-vm)]
    fn set_opcode_small_field() -> pub Field {
        big_field_128_bits
    }

    #[aztec(public-vm)]
    fn set_opcode_big_field() -> pub Field {
        big_field_136_bits
    }

    #[aztec(public-vm)]
    fn add_u128(a: U128, b: U128) -> pub U128 {
        a + b
    }

    /************************************************************************
     * Hashing functions
     ************************************************************************/
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

    #[aztec(public-vm)]
    fn pedersen_hash_with_index(data: [Field; 3]) -> pub Field {
        dep::std::hash::pedersen_hash_with_separator(data, 20)
    }

    /************************************************************************
     * Contract instance
     ************************************************************************/
    #[aztec(public-vm)]
    fn test_get_contract_instance_raw() {
        let fields = get_contract_instance_internal_avm(context.this_address());
        assert(fields.len() == 7);
        assert(fields[0] == 0x1);
        assert(fields[1] == 0x123);
        assert(fields[2] == 0x456);
        assert(fields[3] == 0x789);
        assert(fields[4] == 0x101112);
        assert(fields[5] == 0x131415);
        assert(fields[6] == 0x161718);
    }

    #[aztec(public-vm)]
    fn test_get_contract_instance() {
        let ci = get_contract_instance_avm(context.this_address());
        assert(ci.is_some());
    }

    /************************************************************************
     * AvmContext functions
     ************************************************************************/
    #[aztec(public-vm)]
    fn get_address() -> pub AztecAddress {
        context.this_address()
    }

    #[aztec(public-vm)]
    fn get_storage_address() -> pub AztecAddress {
        context.storage_address()
    }

    #[aztec(public-vm)]
    fn get_sender() -> pub AztecAddress {
        context.msg_sender()
    }

    #[aztec(public-vm)]
    fn get_origin() -> pub AztecAddress {
        context.origin()
    }

    #[aztec(public-vm)]
    fn get_portal() -> pub EthAddress {
        context.this_portal_address()
    }

    #[aztec(public-vm)]
    fn get_fee_per_l1_gas() -> pub Field {
        context.fee_per_l1_gas()
    }

    #[aztec(public-vm)]
    fn get_fee_per_l2_gas() -> pub Field {
        context.fee_per_l2_gas()
    }

    #[aztec(public-vm)]
    fn get_fee_per_da_gas() -> pub Field {
        context.fee_per_da_gas()
    }

    #[aztec(public-vm)]
    fn get_chain_id() -> pub Field {
        context.chain_id()
    }

    #[aztec(public-vm)]
    fn get_version() -> pub Field {
        context.version()
    }

    #[aztec(public-vm)]
    fn get_block_number() -> pub Field {
        context.block_number()
    }

    #[aztec(public-vm)]
    fn get_timestamp() -> pub u64 {
        context.timestamp()
    }

    // #[aztec(public-vm)]
    // fn get_contract_call_depth() -> pub Field {
    //     context.contract_call_depth()
    // }

    #[aztec(public-vm)]
    fn check_selector() {
        assert(context.selector() == FunctionSelector::from_signature("check_selector()"));
    }

    #[aztec(public-vm)]
    fn get_args_hash(_a: u8, _fields: [Field; 3]) -> pub Field {
        context.get_args_hash()
    }

    #[aztec(public-vm)]
    fn emit_unencrypted_log() {
        context.accumulate_unencrypted_logs(/*event_selector=*/ 5, /*message=*/ [10, 20, 30]);
        context.accumulate_unencrypted_logs(/*event_selector=*/ 8, /*message=*/ "Hello, world!");
        let s: CompressedString<2,44> = CompressedString::from_string("A long time ago, in a galaxy far far away...");
        context.accumulate_unencrypted_logs(/*event_selector=*/ 10, /*message=*/ s);
    }

    #[aztec(public-vm)]
    fn note_hash_exists(note_hash: Field, leaf_index: Field) -> pub bool {
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
    fn nullifier_exists(nullifier: Field) -> pub bool {
        context.nullifier_exists(nullifier)
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public-vm)]
    fn emit_nullifier_and_check(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        let exists = context.nullifier_exists(nullifier);
        assert(exists, "Nullifier was just created, but its existence wasn't detected!");
    }

    // Create the same nullifier twice (shouldn't work!)
    #[aztec(public-vm)]
    fn nullifier_collision(nullifier: Field) {
        context.push_new_nullifier(nullifier, 0);
        // Can't do this twice!
        context.push_new_nullifier(nullifier, 0);
    }

    #[aztec(public-vm)]
    fn l1_to_l2_msg_exists(msg_hash: Field, msg_leaf_index: Field) -> pub bool {
        context.l1_to_l2_msg_exists(msg_hash, msg_leaf_index)
    }

    #[aztec(public-vm)]
    fn send_l2_to_l1_msg(recipient: EthAddress, content: Field) {
        context.message_portal(recipient, content)
    }

    // Directly call the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn raw_nested_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)").to_field();
        let gas = [/*l1_gas*/42, /*l2_gas*/24, /*da_gas*/420];

        // Nested call
        let results = context.call_public_function_raw(gas, context.this_address(), selector, [arg_a, arg_b]);
        let data_to_return: [Field; 1] = results.0;
        // this explicit size   ^ is necessary to ensure that ret_size is compile-time
        // (ensure the data_to_return is in a HeapArray not a HeapVector)
        let success: u8 = results.1;

        assert(success == 1, "Call failed");

        let add_result = data_to_return[0];
        add_result
    }

    // Use the `call_public_function` wrapper to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn nested_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)");

        // Nested call using standard context interface function
        let data_to_return: [Field; RETURN_VALUES_LENGTH] = context.call_public_function(context.this_address(), selector, [arg_a, arg_b]);
        // this explicit size   ^ is necessary to ensure that ret_size is compile-time
        // (ensure the data_to_return is in a HeapArray not a HeapVector)

        let add_result = data_to_return[0];
        add_result
    }

    // Directly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn raw_nested_static_call_to_add(arg_a: Field, arg_b: Field) -> pub (Field, u8) {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)").to_field();
        let gas = [/*l1_gas*/42, /*l2_gas*/24, /*da_gas*/420];

        let (result_data, success): ([Field; 1], u8) = context.static_call_public_function_raw(gas, context.this_address(), selector, [arg_a, arg_b]);

        (result_data[0], success)
    }

    // Directly call_static `set_storage_single`. Should fail since it's accessing storage.
    #[aztec(public-vm)]
    fn raw_nested_static_call_to_set_storage() -> pub u8 {
        let selector = FunctionSelector::from_signature("set_storage_single(Field)").to_field();
        let gas = [/*l1_gas*/42, /*l2_gas*/24, /*da_gas*/420];
        let calldata: [Field; 1] = [20];

        let (_data_to_return, success): ([Field; 0], u8) = context.static_call_public_function_raw(gas, context.this_address(), selector, calldata);

        success
    }

    // Indirectly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public-vm)]
    fn nested_static_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        let selector = FunctionSelector::from_signature("add_args_return(Field,Field)");

        let result_data: [Field; RETURN_VALUES_LENGTH] = context.static_call_public_function(context.this_address(), selector, [arg_a, arg_b]);

        result_data[0]
    }

    // Indirectly call_static `set_storage_single`. Should revert since it's accessing storage.
    #[aztec(public-vm)]
    fn nested_static_call_to_set_storage() {
        let selector = FunctionSelector::from_signature("set_storage_single(Field)");
        let calldata: [Field; 1] = [20];

        let _resultData: [Field; RETURN_VALUES_LENGTH] = context.static_call_public_function(context.this_address(), selector, calldata);
    }
}
