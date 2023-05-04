#include "common.hpp"

#include "init.hpp"

namespace aztec3::circuits::kernel::public_kernel {

void common_initialise_end_values(PublicKernelInputs<NT> const& public_kernel_inputs,
                                  KernelCircuitPublicInputs<NT>& circuit_outputs)
{
    // Initialises the circuit outputs with the end state of the previous iteration
    circuit_outputs.constants = public_kernel_inputs.previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other functions
    // within this circuit:
    auto& end = circuit_outputs.end;
    const auto& start = public_kernel_inputs.previous_kernel.public_inputs.end;

    end.new_commitments = start.new_commitments;
    end.new_nullifiers = start.new_nullifiers;

    end.private_call_stack = start.private_call_stack;
    end.public_call_stack = start.public_call_stack;
    end.new_l2_to_l1_msgs = start.new_l2_to_l1_msgs;

    end.optionally_revealed_data = start.optionally_revealed_data;

    end.state_transitions = start.state_transitions;
    end.state_reads = start.state_reads;
}

/**
 * @brief Validates that the call stack item for this circuit iteration is at the top of the call stack
 * @param composer The circuit composer
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
void validate_this_public_call_hash(DummyComposer& composer,
                                    PublicKernelInputs<NT> const& public_kernel_inputs,
                                    KernelCircuitPublicInputs<NT>& public_inputs)
{
    // Pops the current function execution from the stack and validates it against the call stack item

    // TODO: this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee' tx,
    // and the 'gas rebate' tx).
    const auto popped_public_call_hash = array_pop(public_inputs.end.public_call_stack);
    const auto calculated_this_public_call_hash = public_kernel_inputs.public_call.call_stack_item.hash();

    composer.do_assert(
        popped_public_call_hash == calculated_this_public_call_hash,
        format("calculated public_call_hash (",
               calculated_this_public_call_hash,
               ") does not match provided public_call_hash (",
               popped_public_call_hash,
               ") at the top of the call stack"),
        CircuitErrorCode::PUBLIC_KERNEL__CALCULATED_PRIVATE_CALL_HASH_AND_PROVIDED_PRIVATE_CALL_HASH_MISMATCH);
};
}  // namespace aztec3::circuits::kernel::public_kernel