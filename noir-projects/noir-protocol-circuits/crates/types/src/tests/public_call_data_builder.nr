use crate::{
    abis::{
    gas_settings::GasSettings, gas::Gas, call_context::CallContext, function_data::FunctionData,
    public_call_data::{PublicCallData, Proof}, public_call_request::PublicCallRequest,
    public_call_stack_item::PublicCallStackItem,
    public_call_stack_item_compressed::PublicCallStackItemCompressed, log_hash::LogHash
},
    address::AztecAddress,
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    tests::{fixtures, public_circuit_public_inputs_builder::PublicCircuitPublicInputsBuilder}
};
use crate::constants::{
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH
};

struct PublicCallDataBuilder {
    contract_address: AztecAddress,
    public_inputs: PublicCircuitPublicInputsBuilder,
    is_execution_request: bool,
    function_data: FunctionData,
    proof: Proof,
    bytecode_hash: Field,
}

impl PublicCallDataBuilder {
    pub fn new() -> Self {
        let contract_data = fixtures::contracts::default_contract;
        let contract_address = contract_data.address;

        let contract_function = fixtures::contract_functions::default_public_function;
        let function_data = contract_function.data;

        let mut public_inputs = PublicCircuitPublicInputsBuilder::new();

        public_inputs.call_context = CallContext {
            msg_sender: fixtures::contracts::parent_contract.address,
            storage_contract_address: contract_address,
            function_selector: function_data.selector,
            is_delegate_call: false,
            is_static_call: false,
        };

        PublicCallDataBuilder {
            contract_address,
            public_inputs,
            is_execution_request: false,
            function_data,
            proof: Proof {},
            bytecode_hash: contract_function.acir_hash
        }
    }

    pub fn is_delegate_call(&mut self) -> Self {
        self.public_inputs.call_context.is_delegate_call = true;
        self.public_inputs.call_context.storage_contract_address = fixtures::contracts::parent_contract.address;
        self.public_inputs.call_context.msg_sender = fixtures::MSG_SENDER;
        *self
    }

    pub fn append_public_call_requests_for_regular_calls(&mut self, num_requests: u32) {
        self.append_public_call_requests(num_requests, false);
    }

    pub fn append_public_call_requests_for_delegate_calls(&mut self, num_requests: u32) {
        self.append_public_call_requests(num_requests, true);
    }

    fn append_public_call_requests(&mut self, num_requests: u32, is_delegate_call: bool) {
        let index_offset = self.public_inputs.public_call_requests.len();
        for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL {
            if i < num_requests {
                let call_request = PublicCallRequest { item: self.mock_public_call_request_item(index_offset + i, is_delegate_call), counter: i };
                self.public_inputs.public_call_requests.push(call_request);
            }
        }
    }

    fn mock_public_call_request_item(self, index: u32, is_delegate_call: bool) -> PublicCallStackItemCompressed {
        let value_offset = 7788 + index as Field;
        let mut fields = [0; PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH];
        for i in 0..fields.len() {
            fields[i] = value_offset + i as Field;
        }
        let mut item = PublicCallStackItemCompressed::deserialize(fields);
        item.call_context.msg_sender = if is_delegate_call {
            self.public_inputs.call_context.msg_sender
        } else {
            self.public_inputs.call_context.storage_contract_address
        };
        item.call_context.storage_contract_address = if is_delegate_call {
            self.public_inputs.call_context.storage_contract_address
        } else {
            item.contract_address
        };
        item.call_context.is_delegate_call = is_delegate_call;
        item
    }

    pub fn append_public_data_read_requests(&mut self, num_reads: u32) {
        let value_offset = self.public_inputs.contract_storage_reads.len();
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            if i < num_reads {
                let read_request = StorageRead {
                    // The default storage slot is its index + 1.
                    storage_slot: (value_offset + i + 1) as Field,
                    // The default value is its index + 999.
                    current_value: (value_offset + i + 999) as Field,
                    counter: i as u32
                };
                self.public_inputs.contract_storage_reads.push(read_request);
            }
        }
    }

    pub fn append_empty_public_data_read_requests(&mut self, num_reads: u32) {
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            if i < num_reads {
                self.public_inputs.contract_storage_reads.push(StorageRead::empty());
            }
        }
    }

    pub fn append_update_requests(&mut self, num_updates: u32) {
        let value_offset = self.public_inputs.contract_storage_update_requests.len();
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            if i < num_updates {
                let update_request = StorageUpdateRequest {
                    // The default storage slot is its index + 1.
                    storage_slot: (value_offset + i + 1) as Field,
                    // The default value is its index + 890.
                    new_value: (value_offset + i + 890) as Field,
                    counter: i as u32
                };
                self.public_inputs.contract_storage_update_requests.push(update_request);
            }
        }
    }

    pub fn append_empty_update_requests(&mut self, num_updates: u32) {
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            if i < num_updates {
                self.public_inputs.contract_storage_update_requests.push(StorageUpdateRequest::empty());
            }
        }
    }

    pub fn set_unencrypted_logs(&mut self, hash: Field, preimages_length: Field) {
        // Counter set as 0 for testing, like note read requests
        let side_effect = LogHash { value: hash, counter: 0, length: preimages_length };
        self.public_inputs.unencrypted_logs_hashes.push(side_effect);
    }

    pub fn finish(self) -> PublicCallData {
        PublicCallData {
            call_stack_item: PublicCallStackItem {
                contract_address: self.contract_address,
                function_data: self.function_data,
                is_execution_request: self.is_execution_request,
                public_inputs: self.public_inputs.finish()
            },
            proof: self.proof,
            bytecode_hash: self.bytecode_hash
        }
    }
}
