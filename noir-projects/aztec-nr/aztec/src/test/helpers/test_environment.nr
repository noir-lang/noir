use dep::protocol_types::{
    abis::{function_selector::FunctionSelector, private_circuit_public_inputs::PrivateCircuitPublicInputs},
    address::{AztecAddress, PartialAddress}, storage::map::derive_storage_slot_in_map,
    constants::CANONICAL_KEY_REGISTRY_ADDRESS, grumpkin_point::GrumpkinPoint, traits::Deserialize
};

use crate::context::inputs::{PublicContextInputs, PrivateContextInputs};
use crate::context::{packed_returns::PackedReturns, call_interfaces::CallInterface};

use crate::context::{PrivateContext, PublicContext, PrivateVoidCallInterface};
use crate::test::helpers::{cheatcodes, types::{Deployer, TestAccount}, keys};
use crate::keys::constants::{NULLIFIER_INDEX, INCOMING_INDEX, OUTGOING_INDEX, TAGGING_INDEX};
use crate::hash::hash_args;

use crate::note::{
    note_header::NoteHeader, note_interface::NoteInterface,
    utils::{compute_inner_note_hash, compute_note_hash_for_consumption}
};
use crate::oracle::notes::notify_created_note;

struct TestEnvironment {
    contract_address: Option<AztecAddress>,
    args_hash: Option<Field>,
    function_selector: Option<FunctionSelector>
}

impl TestEnvironment {
    fn new() -> Self {
        cheatcodes::reset();
        Self { contract_address: Option::none(), args_hash: Option::none(), function_selector: Option::none() }
    }

    fn block_number(self) -> u32 {
        cheatcodes::get_block_number()
    }

    fn advance_block_to(&mut self, block_number: u32) {
        let difference = block_number - cheatcodes::get_block_number();
        self.advance_block_by(difference);
    }

    fn advance_block_by(&mut self, blocks: u32) {
        cheatcodes::advance_blocks(blocks);
    }

    fn public(self) -> PublicContext {
        PublicContext::empty()
    }

    fn private(&mut self) -> PrivateContext {
        self.private_at(cheatcodes::get_block_number())
    }

    fn private_at(&mut self, historical_block_number: u32) -> PrivateContext {
        if historical_block_number >= cheatcodes::get_block_number() {
            self.advance_block_to(historical_block_number + 1);
        }

        let mut inputs = cheatcodes::get_private_context_inputs(historical_block_number);

        PrivateContext::new(inputs, 0)
    }

    fn create_account(self) -> AztecAddress {
        let test_account = cheatcodes::create_account();
        let address = test_account.address;
        let keys = test_account.keys;

        keys::store_master_key(NULLIFIER_INDEX, address, keys.npk_m);
        keys::store_master_key(INCOMING_INDEX, address, keys.ivpk_m);
        keys::store_master_key(OUTGOING_INDEX, address, keys.ovpk_m);
        keys::store_master_key(TAGGING_INDEX, address, keys.tpk_m);

        test_account.address
    }

    fn create_account_contract(self, secret: Field) -> AztecAddress {
        let public_keys = cheatcodes::derive_keys(secret);
        let args = &[public_keys.ivpk_m.x, public_keys.ivpk_m.y];
        let address = cheatcodes::deploy(
            "@aztec/noir-contracts.js/SchnorrAccount",
            "constructor",
            args,
            public_keys.hash().to_field()
        );
        cheatcodes::advance_blocks(1);
        let test_account = cheatcodes::add_account(secret, PartialAddress::from_field(address.to_field()));
        let address = test_account.address;
        let keys = test_account.keys;

        keys::store_master_key(NULLIFIER_INDEX, address, keys.npk_m);
        keys::store_master_key(INCOMING_INDEX, address, keys.ivpk_m);
        keys::store_master_key(OUTGOING_INDEX, address, keys.ovpk_m);
        keys::store_master_key(TAGGING_INDEX, address, keys.tpk_m);

        test_account.address
    }

    fn deploy<N>(self, path: str<N>) -> Deployer<N> {
        Deployer { path, public_keys_hash: 0 }
    }

    fn call_private<C, M, T, Env, N>(
        self,
        call_interface: C
    ) -> T where C: CallInterface<M, PrivateContextInputs, PrivateCircuitPublicInputs, Env>, T: Deserialize<N> {
        let original_fn = call_interface.get_original();
        let original_msg_sender = cheatcodes::get_msg_sender();
        let original_contract_address = cheatcodes::get_contract_address();
        let target_address = call_interface.get_contract_address();

        cheatcodes::set_contract_address(target_address);
        cheatcodes::set_msg_sender(original_contract_address);
        let mut inputs = cheatcodes::get_private_context_inputs(cheatcodes::get_block_number() - 1);
        inputs.call_context.function_selector = call_interface.get_selector();
        let public_inputs = original_fn(inputs);

        cheatcodes::set_contract_address(original_contract_address);
        cheatcodes::set_msg_sender(original_msg_sender);
        PackedReturns::new(public_inputs.returns_hash).unpack_into()
    }

    fn call_private_void<C, M, Env, N>(
        self,
        call_interface: C
    ) where C: CallInterface<M, PrivateContextInputs, PrivateCircuitPublicInputs, Env> {
        let original_fn = call_interface.get_original();
        let original_msg_sender = cheatcodes::get_msg_sender();
        let original_contract_address = cheatcodes::get_contract_address();
        let target_address = call_interface.get_contract_address();

        cheatcodes::set_contract_address(target_address);
        cheatcodes::set_msg_sender(original_contract_address);
        let mut inputs = cheatcodes::get_private_context_inputs(cheatcodes::get_block_number() - 1);
        inputs.call_context.function_selector = call_interface.get_selector();
        let public_inputs = original_fn(inputs);

        cheatcodes::set_contract_address(original_contract_address);
        cheatcodes::set_msg_sender(original_msg_sender);
        PackedReturns::new(public_inputs.returns_hash).assert_empty();
    }

    fn call_public<C, M, T, Env, N>(self, call_interface: C) -> T where C: CallInterface<M, PublicContextInputs, T, Env> {
        let original_fn = call_interface.get_original();
        let original_msg_sender = cheatcodes::get_msg_sender();
        let original_contract_address = cheatcodes::get_contract_address();
        let target_address = call_interface.get_contract_address();

        cheatcodes::set_contract_address(target_address);
        cheatcodes::set_msg_sender(original_contract_address);
        let mut inputs = cheatcodes::get_public_context_inputs();
        inputs.selector = call_interface.get_selector().to_field();
        inputs.args_hash = hash_args(call_interface.get_args());
        let result = original_fn(inputs);

        cheatcodes::set_contract_address(original_contract_address);
        cheatcodes::set_msg_sender(original_msg_sender);
        result
    }

    fn call_public_void<C, M, Env, N>(self, call_interface: C) where C: CallInterface<M, PublicContextInputs, (), Env> {
        let original_fn = call_interface.get_original();
        let original_msg_sender = cheatcodes::get_msg_sender();
        let original_contract_address = cheatcodes::get_contract_address();
        let target_address = call_interface.get_contract_address();

        cheatcodes::set_contract_address(target_address);
        cheatcodes::set_msg_sender(original_contract_address);
        let mut inputs = cheatcodes::get_public_context_inputs();
        inputs.selector = call_interface.get_selector().to_field();
        inputs.args_hash = hash_args(call_interface.get_args());
        original_fn(inputs);

        cheatcodes::set_contract_address(original_contract_address);
        cheatcodes::set_msg_sender(original_msg_sender);
    }

    pub fn store_note_in_cache<Note, N, M>(
        self,
        note: &mut Note,
        storage_slot: Field,
        contract_address: AztecAddress
    ) where Note: NoteInterface<N, M> {
        let original_contract_address = cheatcodes::get_contract_address();
        cheatcodes::set_contract_address(contract_address);
        let note_hash_counter = cheatcodes::get_side_effects_counter();

        let header = NoteHeader { contract_address, storage_slot, nonce: 0, note_hash_counter };
        // TODO: change this to note.set_header(header) once https://github.com/noir-lang/noir/issues/4095 is fixed
        Note::set_header(note, header);
        let inner_note_hash = compute_inner_note_hash(*note);

        // TODO: Strong typing required because of https://github.com/noir-lang/noir/issues/4088
        let serialized_note: [Field; N] = Note::serialize_content(*note);
        assert(
            notify_created_note(
            storage_slot,
            Note::get_note_type_id(),
            serialized_note,
            inner_note_hash,
            note_hash_counter
        )
            == 0
        );
        cheatcodes::set_contract_address(original_contract_address);
    }
}
