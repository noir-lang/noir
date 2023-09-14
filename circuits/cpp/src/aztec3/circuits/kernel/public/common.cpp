#include "common.hpp"

#include "init.hpp"

#include "aztec3/circuits/abis/call_stack_item.hpp"
#include "aztec3/circuits/abis/types.hpp"

namespace aztec3::circuits::kernel::public_kernel {

using aztec3::utils::array_pop;

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

    end.public_data_update_requests = start.public_data_update_requests;
    end.public_data_reads = start.public_data_reads;

    // Public kernel does not modify encrypted logs values --> we just copy them to output
    end.encrypted_logs_hash = start.encrypted_logs_hash;
    end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
}

/**
 * @brief Validates that the call stack item for this circuit iteration is at the top of the call stack
 * @param builder The circuit builder
 * @param public_kernel_inputs The inputs to this iteration of the kernel circuit
 */
void validate_this_public_call_hash(DummyBuilder& builder,
                                    PublicKernelInputs<NT> const& public_kernel_inputs,
                                    KernelCircuitPublicInputs<NT>& public_inputs)
{
    // If public call stack is empty, we bail so array_pop doesn't throw_or_abort
    if (array_length(public_inputs.end.public_call_stack) == 0) {
        builder.do_assert(
            false, "Public call stack can't be empty", CircuitErrorCode::PUBLIC_KERNEL__EMPTY_PUBLIC_CALL_STACK);
        return;
    }

    // Pops the current function execution from the stack and validates it against the call stack item

    // TODO: this logic might need to change to accommodate the weird edge 3 initial txs (the 'main' tx, the 'fee' tx,
    // and the 'gas rebate' tx).
    const auto popped_public_call_hash = array_pop(public_inputs.end.public_call_stack);
    const auto calculated_this_public_call_hash =
        get_call_stack_item_hash(public_kernel_inputs.public_call.call_stack_item);

    builder.do_assert(
        popped_public_call_hash == calculated_this_public_call_hash,
        format("calculated public_call_hash (",
               calculated_this_public_call_hash,
               ") does not match provided public_call_hash (",
               popped_public_call_hash,
               ") at the top of the call stack"),
        CircuitErrorCode::PUBLIC_KERNEL__CALCULATED_PUBLIC_CALL_HASH_AND_PROVIDED_PUBLIC_CALL_HASH_MISMATCH);
};
}  // namespace aztec3::circuits::kernel::public_kernel