use crate::public_kernel_phase::PublicKernelPhase;
use dep::types::abis::public_kernel_data::PublicKernelData;

struct PreviousKernelValidator {
    previous_kernel: PublicKernelData,
}

impl PreviousKernelValidator {
    pub fn new(previous_kernel: PublicKernelData) -> Self {
        PreviousKernelValidator { previous_kernel }
    }

    pub fn validate_proof<let N: u32>(self, _allowed_indices: [u32; N]) {
        if !dep::std::runtime::is_unconstrained() {
            // Recursively verify the tube proof or a previous public kernel proof
            self.previous_kernel.verify();
            // TODO(#7410) currently stubbed out until tube vk handled
            // self.previous_kernel.validate_in_vk_tree(allowed_indices);
        }
    }

    pub fn validate_phase(self, phase: u8) {
        let public_inputs = self.previous_kernel.public_inputs;

        let needs_setup = !public_inputs.end_non_revertible.public_call_stack[0].item.contract_address.is_zero();
        if phase == PublicKernelPhase.SETUP {
            assert_eq(needs_setup, true, "Cannot run unnecessary setup circuit");
        }

        let needs_app_logic = !public_inputs.end.public_call_stack[0].item.contract_address.is_zero();
        if phase == PublicKernelPhase.APP_LOGIC {
            assert_eq(needs_setup, false, "Cannot run app logic circuit before setup circuit");
            assert_eq(needs_app_logic, true, "Cannot run unnecessary app logic circuit");
        }

        let needs_teardown = !public_inputs.public_teardown_call_stack[0].item.contract_address.is_zero();
        if phase == PublicKernelPhase.TEARDOWN {
            assert_eq(needs_setup, false, "Cannot run teardown circuit before setup circuit");
            assert_eq(needs_app_logic, false, "Cannot run teardown circuit before app logic circuit");
            assert_eq(needs_teardown, true, "Cannot run unnecessary teardown circuit");
        }

        if phase == PublicKernelPhase.TAIL {
            assert_eq(
                needs_setup, false, "Revertible call stack must be empty when executing the tail circuit"
            );
            assert_eq(
                needs_app_logic, false, "Non-revertible call stack must be empty when executing the tail circuit"
            );
            assert_eq(
                needs_teardown, false, "Teardown call stack must be empty when executing the tail circuit"
            );
        }
    }
}
