use dep::types::abis::public_call_data::PublicCallData;
use dep::types::abis::previous_kernel_data::PreviousKernelData;
use dep::types::KernelCircuitPublicInputs;
use dep::types::abis::kernel_circuit_public_inputs::KernelCircuitPublicInputsBuilder;
use crate::common;
use dep::std::unsafe;

struct PublicKernelPublicPreviousInputs {
    previous_kernel: PreviousKernelData,
    public_call: PublicCallData,
}

impl PublicKernelPublicPreviousInputs {

// this is the only difference between the two PublicKernels' logic:
    fn validate_inputs(self) {
        let previous_call_is_private = self.previous_kernel.public_inputs.is_private;
        assert(previous_call_is_private == false,
            "Previous kernel must be public when in this public kernel version");
    }


    fn public_kernel_public_previous(self) -> KernelCircuitPublicInputs {
        // construct the circuit outputs
        let mut public_inputs: KernelCircuitPublicInputsBuilder = unsafe::zeroed();

        // initialise the end state with our provided previous kernel state
        common::initialize_end_values(self.previous_kernel, &mut public_inputs);

        // validate the inputs common to all invocation circumstances
        common::validate_inputs(self.public_call);

        // validate the inputs unique to having a previous public kernel
        self.validate_inputs();

        // Pops the item from the call stack and validates it against the current execution.
        let call_request = public_inputs.end.public_call_stack.pop();
        common::validate_call_against_request(self.public_call, call_request);

        common::update_public_end_values(self.public_call,&mut public_inputs);

        common::accumulate_unencrypted_logs(self.public_call, self.previous_kernel,&mut public_inputs);

        public_inputs.finish()
    }
}

mod tests {
    use crate::{
        public_kernel_public_previous::PublicKernelPublicPreviousInputs,
        utils::{
            assert_eq_public_data_reads,
            assert_eq_public_data_update_requests,
            compute_public_data_reads,
            compute_public_data_update_requests,
        },
    };
    use dep::types::{
        abis::{
            kernel_circuit_public_inputs::KernelCircuitPublicInputs,
            public_data_read::PublicDataRead,
            public_data_update_request::PublicDataUpdateRequest,
            side_effect::{SideEffect, SideEffectLinkedToNoteHash},
        },
        hash::{
            compute_l2_to_l1_hash,
            compute_logs_hash,
            silo_commitment,
            silo_nullifier,
        },
        tests::{
            previous_kernel_data_builder::PreviousKernelDataBuilder,
            public_call_data_builder::PublicCallDataBuilder,
        },
        utils::{
            arrays::array_eq,
        },
    };
    use dep::types::constants::{
        MAX_PUBLIC_DATA_READS_PER_CALL,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
    };

    struct PublicKernelPublicPreviousInputsBuilder {
        previous_kernel: PreviousKernelDataBuilder,
        public_call: PublicCallDataBuilder,
    }

    impl PublicKernelPublicPreviousInputsBuilder {
        pub fn new() -> Self {
            let previous_kernel = PreviousKernelDataBuilder::new().is_public();
            let public_call = PublicCallDataBuilder::new();
            
            PublicKernelPublicPreviousInputsBuilder {
                previous_kernel,
                public_call,
            }
        }

        pub fn is_delegate_call(&mut self) -> Self {
            let _ = self.public_call.is_delegate_call();
            *self
        }

        pub fn get_current_public_data_reads(self) -> [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_CALL] {
            let read_requests = self.public_call.public_inputs.contract_storage_reads.storage;
            compute_public_data_reads(self.public_call.contract_address, read_requests)
        }

        pub fn get_current_public_data_update_requests(self) -> [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL] {
            compute_public_data_update_requests(
                self.public_call.contract_address,
                self.public_call.public_inputs.contract_storage_update_requests.storage,
            )
        }

        pub fn execute(&mut self) -> KernelCircuitPublicInputs {
            let public_call = self.public_call.finish();
            // Adjust the call stack item hash for the current call in the previous iteration.
            let hash = public_call.call_stack_item.hash();
            let is_delegate_call = public_call.call_stack_item.public_inputs.call_context.is_delegate_call;
            self.previous_kernel.push_public_call_request(hash, is_delegate_call);
            let previous_kernel = self.previous_kernel.finish();

            let kernel = PublicKernelPublicPreviousInputs {
                previous_kernel,
                public_call,
            };

            kernel.public_kernel_public_previous()
        }

        pub fn succeeded(&mut self) {
            let _ = self.execute();
        }

        pub fn failed(&mut self) {
            let _ = self.execute();
        }
    }

    #[test]
    fn public_kernel_circuit_with_public_previous_kernel_should_succeed() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        builder.succeeded();
    }

    // TODO: Find another way to test this. Currently it will crash because we are popping from an empty array:
    // The application panicked (crashed). Message:  Expected array index to fit in u64
    // #[test(should_fail_with="Public call stack can not be empty")]
    // fn public_previous_kernel_empty_public_call_stack_should_fail() {
    //     let builder = PublicKernelPublicPreviousInputsBuilder::new();
    //     let previous_kernel = builder.previous_kernel.finish();
    //     let public_call = builder.public_call.finish();
    //     let kernel = PublicKernelPublicPreviousInputs {
    //         previous_kernel,
    //         public_call,
    //     };
    //
    //     let _ = kernel.public_kernel_public_previous();
    // }

    #[test(should_fail_with="Previous kernel must be public when in this public kernel version")]
    fn public_previous_kernel_private_previous_kernel_should_fail() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        builder.previous_kernel = PreviousKernelDataBuilder::new();

        builder.failed();
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_commitments() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        let contract_address = builder.public_call.contract_address;
        // Setup 2 new commitments on the previous kernel.
        builder.previous_kernel.append_new_commitments(2);
        let previous = builder.previous_kernel.end.new_commitments.storage;
        // Setup 2 new commitments on the current public inputs.
        let current = [
            SideEffect { value: previous[1].value + 1, counter: 3 },
            SideEffect { value: previous[1].value + 2, counter: 4 }
        ];
        builder.public_call.public_inputs.new_commitments.push_array(current);
        let siloed = current.map(|c: SideEffect| silo_commitment(contract_address, c.value));
        let new_commitments = [
            previous[0], previous[1], SideEffect { value: siloed[0], counter: 3 }, SideEffect { value: siloed[1], counter: 4 }
        ];

        let public_inputs = builder.execute();

        assert(array_eq(public_inputs.end.new_commitments, new_commitments));
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_update_requests() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        // Setup 2 data writes on the previous kernel.
        builder.previous_kernel.append_public_data_update_requests(2);
        let previous = builder.previous_kernel.end.public_data_update_requests.storage;
        // Setup 2 data writes on the current public inputs.
        builder.public_call.append_update_requests(2);
        let current = builder.get_current_public_data_update_requests();
        let public_data_update_requests = [previous[0], previous[1], current[0], current[1]];

        let public_inputs = builder.execute();
        assert_eq_public_data_update_requests(
            public_inputs.end.public_data_update_requests,
            public_data_update_requests
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_read_requests() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();

        // Setup 2 data reads on the previous kernel.
        builder.previous_kernel.append_public_data_read_requests(2);
        let previous = builder.previous_kernel.end.public_data_reads.storage;
        // Setup 2 data reads on the current public inputs.
        builder.public_call.append_read_requests(2);
        let current = builder.get_current_public_data_reads();
        let public_data_read_requests = [previous[0], previous[1], current[0], current[1]];
        let public_inputs = builder.execute();

        assert_eq_public_data_reads(
            public_inputs.end.public_data_reads,
            public_data_read_requests
        );
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_nullifiers() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        let contract_address = builder.public_call.contract_address;

        // Setup 2 new nullifiers on the previous kernel.
        builder.previous_kernel.append_new_nullifiers(2);
        let previous = builder.previous_kernel.end.new_nullifiers.storage;
        // Setup 2 new commitments on the current public inputs.

        let current = [
            SideEffectLinkedToNoteHash { value: previous[2].value + 1, note_hash: 0, counter: 3 }, SideEffectLinkedToNoteHash { value: previous[2].value + 2, note_hash: 0, counter: 4 }
        ];

        let siloed = current.map(
            |current: SideEffectLinkedToNoteHash| 
        SideEffectLinkedToNoteHash { value: silo_nullifier(contract_address, current.value), note_hash: current.note_hash, counter: current.counter }
        );

        builder.public_call.public_inputs.new_nullifiers.push_array(current);
        // There are 3 nullifiers in the previous kernel. The first one is the tx nullifier.
        let new_nullifiers = [previous[0], previous[1], previous[2], siloed[0], siloed[1]];

        let public_inputs = builder.execute();

        assert(array_eq(public_inputs.end.new_nullifiers, new_nullifiers));
    }

    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_l2_to_l1_msg() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();

        let contract_address = builder.public_call.contract_address;
        let portal_contract_address = builder.public_call.portal_contract_address;
        // Setup 1 new l2 to l1 message on the previous kernel.
        let previous = [12345];

        builder.previous_kernel.end.new_l2_to_l1_msgs.push_array(previous);
        // Setup 1 new l2 to l1 message on the current public inputs.
        let current = [67890];
        builder.public_call.public_inputs.new_l2_to_l1_msgs.push_array(current);
        let tx_context = builder.previous_kernel.tx_context;
        let version = tx_context.version;
        let chain_id = tx_context.chain_id;
        let siloed = current.map(|c| compute_l2_to_l1_hash(contract_address, version, portal_contract_address, chain_id, c));
        let new_l2_to_l1_msgs = [previous[0], siloed[0]];

        let public_inputs = builder.execute();
        assert(array_eq(public_inputs.end.new_l2_to_l1_msgs, new_l2_to_l1_msgs));
    }
    // TODO: Break up this test into smaller tests.
    #[test]
    fn circuit_outputs_should_be_correctly_populated_with_previous_public_kernel_logs() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        // Logs for the current call stack.
        let unencrypted_logs_hash = [26, 47];
        let unencrypted_log_preimages_length = 50;
        builder.public_call.set_unencrypted_logs(unencrypted_logs_hash, unencrypted_log_preimages_length);

        // Logs for the previous call stack.
        let prev_encrypted_logs_hash = [80, 429];
        let prev_encrypted_log_preimages_length = 13;
        let prev_unencrypted_logs_hash = [956, 112];
        let prev_unencrypted_log_preimages_length = 24;
        builder.previous_kernel.set_encrypted_logs(prev_encrypted_logs_hash, prev_encrypted_log_preimages_length);
        builder.previous_kernel.set_unencrypted_logs(
            prev_unencrypted_logs_hash,
            prev_unencrypted_log_preimages_length
        );

        let public_inputs = builder.execute();

        assert_eq(public_inputs.end.encrypted_log_preimages_length, prev_encrypted_log_preimages_length);
        assert_eq(
            public_inputs.end.unencrypted_log_preimages_length, unencrypted_log_preimages_length + prev_unencrypted_log_preimages_length
        );

        assert_eq(public_inputs.end.encrypted_logs_hash, prev_encrypted_logs_hash);

        let expected_unencrypted_logs_hash = compute_logs_hash(prev_unencrypted_logs_hash, unencrypted_logs_hash);
        assert_eq(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
    }

    #[test(should_fail_with="No contract storage update requests are allowed for static calls")]
    fn previous_public_kernel_fails_if_contract_storage_update_requests_on_static_call() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.append_update_requests(1);

        builder.failed();
    }

    #[test(should_fail_with="curent contract address must not match storage contract address for delegate calls")]
    fn previous_public_kernel_fails_if_incorrect_storage_contract_on_delegate_call() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new().is_delegate_call();

        builder.public_call.contract_address = builder.public_call.public_inputs.call_context.storage_contract_address;

        builder.failed();
    }

    #[test(should_fail_with="new_commitments must be empty for static calls")]
    fn public_kernel_fails_creating_new_commitments_on_static_call() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.public_inputs.new_commitments.push(SideEffect { value: 1, counter: 0 });

        builder.failed();
    }

    #[test(should_fail_with="new_nullifiers must be empty for static calls")]
    fn public_kernel_fails_creating_new_nullifiers_on_static_call() {
        let mut builder = PublicKernelPublicPreviousInputsBuilder::new();
        builder.public_call.public_inputs.call_context.is_static_call = true;
        builder.public_call.public_inputs.new_nullifiers.push(SideEffectLinkedToNoteHash { value: 1, note_hash: 0, counter: 0 });

        builder.failed();
    }
}
