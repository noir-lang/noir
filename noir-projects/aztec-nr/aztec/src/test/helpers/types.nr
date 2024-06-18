use dep::protocol_types::{
    traits::{Deserialize, Serialize}, address::AztecAddress,
    abis::{function_selector::FunctionSelector, private_circuit_public_inputs::PrivateCircuitPublicInputs}
};

use crate::context::inputs::{PublicContextInputs, PrivateContextInputs};
use crate::context::call_interfaces::CallInterface;
use crate::test::helpers::cheatcodes;
use crate::keys::public_keys::{PUBLIC_KEYS_LENGTH, PublicKeys};
use crate::hash::hash_args;

struct Deployer<N> {
        path: str<N>,
        public_keys_hash: Field
    }

impl<N> Deployer<N> {
    pub fn with_private_initializer<C, M, Env>(
        self,
        call_interface: C
    ) -> AztecAddress where C: CallInterface<M, PrivateContextInputs, PrivateCircuitPublicInputs, Env> {
        let address = cheatcodes::deploy(
            self.path,
            call_interface.get_name(),
            call_interface.get_args(),
            self.public_keys_hash
        );
        cheatcodes::advance_blocks(1);
        let block_number = cheatcodes::get_block_number();
        let original_fn = call_interface.get_original();
        let original_msg_sender = cheatcodes::get_msg_sender();
        let original_contract_address = cheatcodes::get_contract_address();

        cheatcodes::set_contract_address(address);
        cheatcodes::set_msg_sender(original_contract_address);
        let mut inputs = cheatcodes::get_private_context_inputs(block_number - 1);
        inputs.call_context.function_selector = call_interface.get_selector();
        let _result = original_fn(inputs);

        cheatcodes::set_contract_address(original_contract_address);
        cheatcodes::set_msg_sender(original_msg_sender);
        address
    }

    pub fn with_public_initializer<C, M, T, Env>(
        self,
        call_interface: C
    ) -> AztecAddress where C: CallInterface<M, PublicContextInputs, T, Env> {
        let address = cheatcodes::deploy(
            self.path,
            call_interface.get_name(),
            call_interface.get_args(),
            self.public_keys_hash
        );
        cheatcodes::advance_blocks(1);
        let original_fn = call_interface.get_original();
        let original_msg_sender = cheatcodes::get_msg_sender();
        let original_contract_address = cheatcodes::get_contract_address();

        cheatcodes::set_contract_address(address);
        cheatcodes::set_msg_sender(original_contract_address);
        let mut inputs = cheatcodes::get_public_context_inputs();
        inputs.selector = call_interface.get_selector().to_field();
        inputs.args_hash = hash_args(call_interface.get_args());
        let _result: T = original_fn(inputs);

        cheatcodes::set_contract_address(original_contract_address);
        cheatcodes::set_msg_sender(original_msg_sender);
        address
    }

    pub fn without_initializer(self) -> AztecAddress {
        let address = cheatcodes::deploy(self.path, "", &[], self.public_keys_hash);
        address
    }
}

// Keys length + address
global TEST_ACCOUNT_LENGTH = PUBLIC_KEYS_LENGTH + 1;

struct TestAccount {
    address: AztecAddress,
    keys: PublicKeys
}

impl Serialize<TEST_ACCOUNT_LENGTH> for TestAccount {
    fn serialize(self) -> [Field; TEST_ACCOUNT_LENGTH] {
        let mut output = [0; TEST_ACCOUNT_LENGTH];

        output[0] = self.address.to_field();

        for i in 0..PUBLIC_KEYS_LENGTH {
            output[i+1] = self.keys.serialize()[i];
        }
        output
    }
}

impl Deserialize<TEST_ACCOUNT_LENGTH> for TestAccount {
    fn deserialize(input: [Field; TEST_ACCOUNT_LENGTH]) -> Self {
        let address = AztecAddress::from_field(input[0]);
        let mut key_buffer = [0; PUBLIC_KEYS_LENGTH];
        for i in 0..PUBLIC_KEYS_LENGTH {
            key_buffer[i] = input[i+1];
        }
        let keys = PublicKeys::deserialize(key_buffer);

        Self { address, keys }
    }
}
