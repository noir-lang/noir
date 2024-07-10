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
    use std::embedded_curve_ops::multi_scalar_mul;
    use dep::aztec::protocol_types::constants::CONTRACT_INSTANCE_LENGTH;
    use dep::aztec::prelude::{Map, Deserialize};
    use dep::aztec::state_vars::PublicMutable;
    use dep::aztec::protocol_types::{address::{AztecAddress, EthAddress}, constants::L1_TO_L2_MESSAGE_LENGTH, point::Point, scalar::Scalar};
    use dep::aztec::oracle::get_contract_instance::{get_contract_instance_avm, get_contract_instance_internal_avm};
    use dep::aztec::protocol_types::abis::function_selector::FunctionSelector;
    use dep::aztec::context::gas::GasOpts;
    use dep::compressed_string::CompressedString;

    #[aztec(storage)]
    struct Storage {
        single: PublicMutable<Field>,
        list: PublicMutable<Note>,
        map: Map<AztecAddress, PublicMutable<u32>>,
    }

    /************************************************************************
     * Storage
     ************************************************************************/
    #[aztec(public)]
    fn set_storage_single(a: Field) {
        storage.single.write(a);
    }

    #[aztec(public)]
    fn read_storage_single() -> Field {
        storage.single.read()
    }

    // should still be able to use ` -> pub *` for return type even though macro forces `pub`
    #[aztec(public)]
    fn set_read_storage_single(a: Field) -> pub Field {
        storage.single.write(a);
        storage.single.read()
    }

    #[aztec(public)]
    fn set_storage_list(a: Field, b: Field) {
        storage.list.write(Note { a, b });
    }

    #[aztec(public)]
    fn read_storage_list() -> [Field; 2] {
        let note: Note = storage.list.read();
        note.serialize()
    }

    #[aztec(public)]
    fn set_storage_map(to: AztecAddress, amount: u32) -> Field {
        storage.map.at(to).write(amount);
        // returns storage slot for key
        std::hash::pedersen_hash([storage.map.storage_slot, to.to_field()])
    }

    #[aztec(public)]
    fn add_storage_map(to: AztecAddress, amount: u32) -> Field {
        let new_balance = storage.map.at(to).read().add(amount);
        storage.map.at(to).write(new_balance);
        // returns storage slot for key
        std::hash::pedersen_hash([storage.map.storage_slot, to.to_field()])
    }

    #[aztec(public)]
    fn read_storage_map(address: AztecAddress) -> u32 {
        storage.map.at(address).read()
    }

    #[aztec(public)]
    fn add_args_return(arg_a: Field, arg_b: Field) -> Field {
        arg_a + arg_b
    }

    /************************************************************************
     * General Opcodes
     ************************************************************************/
    #[aztec(public)]
    fn set_opcode_u8() -> u8 {
        8 as u8
    }

    #[aztec(public)]
    fn set_opcode_u32() -> u32 {
        1 << 30 as u8
    }

    #[aztec(public)]
    fn set_opcode_u64() -> u64 {
        1 << 60 as u8
    }

    #[aztec(public)]
    fn set_opcode_small_field() -> Field {
        big_field_128_bits
    }

    #[aztec(public)]
    fn set_opcode_big_field() -> Field {
        big_field_136_bits
    }

    #[aztec(public)]
    fn add_u128(a: U128, b: U128) -> U128 {
        a + b
    }

    #[aztec(public)]
    fn modulo2(a: u64) -> u64 {
        a % 2
    }

    #[aztec(public)]
    fn elliptic_curve_add_and_double() -> Point {
        let g = Point { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };

        let doubled = g + g;
        let added = g + doubled;
        added
    }

    #[aztec(public)]
    fn variable_base_msm() -> Point {
        let g = Point { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
        let scalar = Scalar { lo: 3, hi: 0 };
        let scalar2 = Scalar { lo: 20, hi: 0 };
        let triple_g = multi_scalar_mul([g, g], [scalar, scalar2]);
        triple_g
    }

    /************************************************************************
     * Misc 
     ************************************************************************/

    #[aztec(public)]
    fn u128_addition_overflow() -> U128 {
        let max_u128: U128 = U128::from_hex("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        let one: U128 = U128::from_integer(1);
        max_u128 + one
    }

    #[aztec(public)]
    fn u128_from_integer_overflow() -> U128 {
        let should_overflow: Field = 2.pow_32(128); // U128::max() + 1;
        U128::from_integer(should_overflow)
    }

    #[aztec(public)]
    fn to_radix_le(input: Field) -> [u8; 10] {
        let result: [u8] = input.to_le_radix(/*base=*/ 2, /*limbs=*/ 10);
        result.as_array()
    }

    // Helper functions to demonstrate an internal call stack in error messages
    fn inner_helper_with_failed_assertion() {
        let not_true = false;
        assert(not_true == true, "This assertion should fail!");
    }
    fn helper_with_failed_assertion() {
        inner_helper_with_failed_assertion();
    }

    #[aztec(public)]
    fn assertion_failure() {
        helper_with_failed_assertion()
    }

    #[aztec(public)]
    fn debug_logging() {
        dep::aztec::oracle::debug_log::debug_log("just text");
        dep::aztec::oracle::debug_log::debug_log_format("second: {1}", [1, 2, 3, 4]);
        dep::aztec::oracle::debug_log::debug_log_format("whole array: {}", [1, 2, 3, 4]);
        dep::aztec::oracle::debug_log::debug_log("tabs and newlines\n\t- first\n\t- second");
    }

    #[aztec(public)]
    fn assert_same(arg_a: Field, arg_b: Field) -> pub Field {
        assert(arg_a == arg_b, "Values are not equal");
        1
    }

    /************************************************************************
     * Hashing functions
     ************************************************************************/
    #[aztec(public)]
    fn keccak_hash(data: [u8; 10]) -> [u8; 32] {
        std::hash::keccak256(data, data.len() as u32)
    }

    #[aztec(public)]
    fn poseidon2_hash(data: [Field; 10]) -> Field {
        std::hash::poseidon2::Poseidon2::hash(data, data.len())
    }

    #[aztec(public)]
    fn sha256_hash(data: [u8; 10]) -> [u8; 32] {
        std::hash::sha256(data)
    }

    #[aztec(public)]
    fn pedersen_hash(data: [Field; 10]) -> Field {
        std::hash::pedersen_hash(data)
    }

    #[aztec(public)]
    fn pedersen_hash_with_index(data: [Field; 10]) -> Field {
        std::hash::pedersen_hash_with_separator(data, /*index=*/ 20)
    }

    /************************************************************************
     * Contract instance
     ************************************************************************/
    #[aztec(public)]
    fn test_get_contract_instance_raw() {
        let fields = get_contract_instance_internal_avm(context.this_address());
        // The values here should match those in `avm_simulator.test.ts>Contract>GETCONTRACTINSTANCE deserializes correctly`
        assert(fields.len() == CONTRACT_INSTANCE_LENGTH + 1);
        assert(fields[0] == 0x1);
        assert(fields[1] == 0x123);
        assert(fields[2] == 0x456);
        assert(fields[3] == 0x789);
        assert(fields[4] == 0x101112);
        assert(fields[5] == 0x161718);
    }

    #[aztec(public)]
    fn test_get_contract_instance() {
        let ci = get_contract_instance_avm(context.this_address());
        assert(ci.is_some(), "Contract instance not found!");
    }

    /************************************************************************
     * AvmContext functions
     ************************************************************************/
    #[aztec(public)]
    fn get_address() -> AztecAddress {
        context.this_address()
    }

    #[aztec(public)]
    fn get_storage_address() -> AztecAddress {
        context.storage_address()
    }

    #[aztec(public)]
    fn get_sender() -> AztecAddress {
        context.msg_sender()
    }

    #[aztec(public)]
    fn get_function_selector() -> FunctionSelector {
        context.selector()
    }

    #[aztec(public)]
    fn get_transaction_fee() -> Field {
        context.transaction_fee()
    }

    #[aztec(public)]
    fn get_chain_id() -> Field {
        context.chain_id()
    }

    #[aztec(public)]
    fn get_version() -> Field {
        context.version()
    }

    #[aztec(public)]
    fn get_block_number() -> Field {
        context.block_number()
    }

    #[aztec(public)]
    fn get_timestamp() -> u64 {
        context.timestamp()
    }

    #[aztec(public)]
    fn get_fee_per_l2_gas() -> Field {
        context.fee_per_l2_gas()
    }

    #[aztec(public)]
    fn get_fee_per_da_gas() -> Field {
        context.fee_per_da_gas()
    }

    #[aztec(public)]
    fn get_l2_gas_left() -> Field {
        context.l2_gas_left()
    }

    #[aztec(public)]
    fn get_da_gas_left() -> Field {
        context.da_gas_left()
    }

    #[aztec(public)]
    fn assert_timestamp(expected_timestamp: u64) {
        let timestamp = context.timestamp();
        assert(timestamp == expected_timestamp, "timestamp does not match");
    }

    #[aztec(public)]
    fn check_selector() {
        assert(
            context.selector() == FunctionSelector::from_signature("check_selector()"), "Unexpected selector!"
        );
    }

    #[aztec(public)]
    fn get_args_hash(_a: u8, _fields: [Field; 3]) -> Field {
        context.get_args_hash()
    }

    #[aztec(public)]
    fn emit_unencrypted_log() {
        context.emit_unencrypted_log(/*message=*/ [10, 20, 30]);
        context.emit_unencrypted_log(/*message=*/ "Hello, world!");
        let s: CompressedString<2,44> = CompressedString::from_string("A long time ago, in a galaxy far far away...");
        context.emit_unencrypted_log(/*message=*/ s);
    }

    #[aztec(public)]
    fn note_hash_exists(note_hash: Field, leaf_index: Field) -> bool {
        context.note_hash_exists(note_hash, leaf_index)
    }

    // Use the standard context interface to emit a new note hash
    #[aztec(public)]
    fn new_note_hash(note_hash: Field) {
        context.push_note_hash(note_hash);
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public)]
    fn new_nullifier(nullifier: Field) {
        context.push_nullifier(nullifier, 0);
    }

    // Use the standard context interface to check for a nullifier
    #[aztec(public)]
    fn nullifier_exists(nullifier: Field) -> bool {
        context.nullifier_exists(nullifier, context.storage_address())
    }

    #[aztec(public)]
    fn assert_nullifier_exists(nullifier: Field) {
        assert(context.nullifier_exists(nullifier, context.storage_address()), "Nullifier doesn't exist!");
    }

    // Use the standard context interface to emit a new nullifier
    #[aztec(public)]
    fn emit_nullifier_and_check(nullifier: Field) {
        context.push_nullifier(nullifier, 0);
        let exists = context.nullifier_exists(nullifier, context.storage_address());
        assert(exists, "Nullifier was just created, but its existence wasn't detected!");
    }

    // Create the same nullifier twice (shouldn't work!)
    #[aztec(public)]
    fn nullifier_collision(nullifier: Field) {
        context.push_nullifier(nullifier, 0);
        // Can't do this twice!
        context.push_nullifier(nullifier, 0);
    }

    #[aztec(public)]
    fn l1_to_l2_msg_exists(msg_hash: Field, msg_leaf_index: Field) -> bool {
        context.l1_to_l2_msg_exists(msg_hash, msg_leaf_index)
    }

    #[aztec(public)]
    fn send_l2_to_l1_msg(recipient: EthAddress, content: Field) {
        context.message_portal(recipient, content)
    }

    /************************************************************************
     * Nested calls
     ************************************************************************/
    #[aztec(public)]
    fn nested_call_to_add_with_gas(
        arg_a: Field,
        arg_b: Field,
        l2_gas: Field,
        da_gas: Field
    ) -> pub Field {
        AvmTest::at(context.this_address()).add_args_return(arg_a, arg_b).with_gas(GasOpts::new(l2_gas, da_gas)).call(&mut context)
    }

    // Use the `call_public_function` wrapper to initiate a nested call to the add function
    #[aztec(public)]
    fn nested_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        AvmTest::at(context.this_address()).add_args_return(arg_a, arg_b).call(&mut context)
    }

    // Indirectly call_static the external call opcode to initiate a nested call to the add function
    #[aztec(public)]
    fn nested_static_call_to_add(arg_a: Field, arg_b: Field) -> pub Field {
        AvmTest::at(context.this_address()).add_args_return(arg_a, arg_b).view(&mut context)
    }

    // Indirectly call_static `set_storage_single`. Should revert since it's accessing storage.
    #[aztec(public)]
    fn nested_static_call_to_set_storage() {
        AvmTest::at(context.this_address()).set_storage_single(20).view(&mut context);
    }

    #[aztec(public)]
    fn create_same_nullifier_in_nested_call(nestedAddress: AztecAddress, nullifier: Field) {
        context.push_nullifier(nullifier, 0);
        AvmTest::at(nestedAddress).new_nullifier(nullifier).call(&mut context);
    }

    #[aztec(public)]
    fn create_different_nullifier_in_nested_call(nestedAddress: AztecAddress, nullifier: Field) {
        context.push_nullifier(nullifier, 0);
        AvmTest::at(nestedAddress).new_nullifier(nullifier + 1).call(&mut context);
    }

    /**
     * Enqueue a public call from private
     */
    #[aztec(private)]
    fn enqueue_public_from_private() {
        AvmTest::at(context.this_address()).set_opcode_u8().enqueue_view(&mut context);
        AvmTest::at(context.this_address()).set_read_storage_single(5).enqueue(&mut context);
    }
}
