use crate::public_kernel_phase::PublicKernelPhase;
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::PublicKernelCircuitPublicInputs, public_call_data::PublicCallData,
    public_call_request::PublicCallRequest
},
    constants::MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, utils::arrays::array_length
};

struct PublicCallDataValidator {
    data: PublicCallData,
    phase: u8,
}

impl PublicCallDataValidator {
    pub fn new(data: PublicCallData, phase: u8) -> Self {
        PublicCallDataValidator { data, phase }
    }

    pub fn validate(self) {
        self.validate_common_inputs();
        self.validate_revert_code();
        self.validate_call();
        self.validate_call_requests();
    }

    pub fn validate_against_previous_kernel(self, previous_kernel: PublicKernelCircuitPublicInputs) {
        self.validate_global_variables(previous_kernel);
        self.validate_start_gas(previous_kernel);
        self.validate_transaction_fee(previous_kernel);
        self.validate_against_call_request(previous_kernel);
    }

    fn validate_common_inputs(self) {
        let call_stack_item = self.data.call_stack_item;
        assert(!call_stack_item.contract_address.is_zero(), "Contract address cannot be zero");
        assert(call_stack_item.function_data.selector.to_field() != 0, "Function signature cannot be zero");
        assert_eq(
            call_stack_item.function_data.is_private, false, "Cannot execute a private function with the public kernel circuit"
        );
        assert_eq(
            call_stack_item.function_data.selector, call_stack_item.public_inputs.call_context.function_selector, "function selector in call context does not match call stack item"
        );
        assert(self.data.bytecode_hash != 0, "Bytecode hash cannot be zero");
    }

    fn validate_revert_code(self) {
        if self.phase == PublicKernelPhase.SETUP {
            assert_eq(self.data.call_stack_item.public_inputs.revert_code, 0, "Public call cannot be reverted");
        }
    }

    fn validate_call(self) {
        let call_stack_item = self.data.call_stack_item;
        let public_inputs = call_stack_item.public_inputs;

        let call_context = public_inputs.call_context;
        if call_context.is_delegate_call {
            assert(
                !call_stack_item.contract_address.eq(call_context.storage_contract_address), "curent contract address must not match storage contract address for delegate calls"
            );
        } else {
            assert(
                call_context.storage_contract_address.eq(call_stack_item.contract_address), "call stack storage address does not match expected contract address"
            );
        }

        if call_context.is_static_call {
            // No state changes are allowed for static calls:
            let note_hashes_length = array_length(public_inputs.note_hashes);
            assert(note_hashes_length == 0, "note_hashes must be empty for static calls");

            let nullifiers_length = array_length(public_inputs.nullifiers);
            assert(nullifiers_length == 0, "nullifiers must be empty for static calls");

            let update_requests_length = array_length(public_inputs.contract_storage_update_requests);
            assert(
                update_requests_length == 0, "No contract storage update requests are allowed for static calls"
            );

            let l2_to_l1_msgs_length = array_length(public_inputs.l2_to_l1_msgs);
            assert(l2_to_l1_msgs_length == 0, "l2_to_l1_msgs must be empty for static calls");

            let new_unencrypted_logs_length = array_length(public_inputs.unencrypted_logs_hashes);
            assert(new_unencrypted_logs_length == 0, "No unencrypted logs are allowed for static calls");
        }
    }

    fn validate_call_requests(self) {
        let call_requests = self.data.call_stack_item.public_inputs.public_call_requests;
        let this_context = self.data.call_stack_item.public_inputs.call_context;
        for i in 0..call_requests.len() {
            let request = call_requests[i];
            if !request.item.contract_address.is_zero() {
                let target_context = request.item.call_context;
                let target_contract = request.item.contract_address;

                if target_context.is_delegate_call {
                    assert_eq(
                        target_context.msg_sender, this_context.msg_sender, "incorrect msg_sender for delegate call request"
                    );
                    assert_eq(
                        target_context.storage_contract_address, this_context.storage_contract_address, "incorrect storage_contract_address for delegate call request"
                    );
                } else {
                    assert_eq(
                        target_context.msg_sender, this_context.storage_contract_address, "incorrect msg_sender for call request"
                    );
                    assert_eq(
                        target_context.storage_contract_address, target_contract, "incorrect storage_contract_address for call request"
                    );
                }
                if !target_context.is_static_call {
                    assert(this_context.is_static_call == false, "static call cannot make non-static calls");
                }
            }
        }
    }

    fn validate_against_call_request(self, previous_kernel: PublicKernelCircuitPublicInputs) {
        let call_stack = if self.phase == PublicKernelPhase.SETUP {
            previous_kernel.end_non_revertible.public_call_stack
        } else if self.phase == PublicKernelPhase.APP_LOGIC {
            previous_kernel.end.public_call_stack
        } else if self.phase == PublicKernelPhase.TEARDOWN {
            previous_kernel.public_teardown_call_stack
        } else {
            assert(false, "Unknown phase");
            [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX]
        };

        let call_request = call_stack[array_length(call_stack) - 1];
        assert(
            self.data.call_stack_item.get_compressed() == call_request.item, "call stack item does not match item at the top of the call stack"
        );
    }

    fn validate_global_variables(self, previous_kernel: PublicKernelCircuitPublicInputs) {
        let prev_global_variables = previous_kernel.constants.global_variables;
        if !prev_global_variables.is_empty() { // It's empty when the previous kernel is from private_kernel_tail_to_pubic.
            let public_call_globals = self.data.call_stack_item.public_inputs.global_variables;
            assert_eq(
                public_call_globals, prev_global_variables, "Global variables injected into the public call do not match constants"
            );
        }
    }

    // Validates that the start gas injected into the app circuit matches the remaining gas.
    fn validate_start_gas(self, previous_kernel: PublicKernelCircuitPublicInputs) {
        // If this is a nested call (not an execution request), the start gas is correct as long as the
        // call being processed by this kernel iteration matches the call at the top of the callstack
        // as per the previous kernel's outputs.
        // An execution request's start gas is the remaining gas left in the transaction after the previous kernel.
        // A nested call's start gas is the gas allocated to it by its caller and placed in the callstack.
        if self.data.call_stack_item.is_execution_request {
            let public_call_start_gas = self.data.call_stack_item.public_inputs.start_gas_left;
            if self.phase != PublicKernelPhase.TEARDOWN {
                let tx_gas_limits = previous_kernel.constants.tx_context.gas_settings.gas_limits;
                let computed_start_gas = tx_gas_limits.sub(previous_kernel.end.gas_used).sub(previous_kernel.end_non_revertible.gas_used);
                assert_eq(
                    public_call_start_gas, computed_start_gas, "Start gas for public phase does not match transaction gas left"
                );
            } else {
                let teardown_gas_limit = previous_kernel.constants.tx_context.gas_settings.teardown_gas_limits;
                assert_eq(
                    public_call_start_gas, teardown_gas_limit, "Start gas for teardown phase does not match teardown gas allocation"
                );
            }
        }
    }

    fn validate_transaction_fee(self, previous_kernel: PublicKernelCircuitPublicInputs) {
        let transaction_fee = self.data.call_stack_item.public_inputs.transaction_fee;
        if self.phase != PublicKernelPhase.TEARDOWN {
            assert_eq(transaction_fee, 0, "Transaction fee must be zero on setup and app phases");
        } else {
            // Note that teardown_gas is already included in end.gas_used as it was injected by the private kernel
            let total_gas_used = previous_kernel.end.gas_used + previous_kernel.end_non_revertible.gas_used;
            let block_gas_fees = self.data.call_stack_item.public_inputs.global_variables.gas_fees;
            let inclusion_fee = previous_kernel.constants.tx_context.gas_settings.inclusion_fee;
            let computed_transaction_fee = total_gas_used.compute_fee(block_gas_fees) + inclusion_fee;
            assert(
                transaction_fee == computed_transaction_fee, "Transaction fee on teardown phase does not match expected value"
            );
        }
    }
}
