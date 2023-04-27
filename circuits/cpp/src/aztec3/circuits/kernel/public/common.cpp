#include "init.hpp"
#include "common.hpp"

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
    end.l1_msg_stack = start.l1_msg_stack;

    end.optionally_revealed_data = start.optionally_revealed_data;

    end.state_transitions = start.state_transitions;
}
} // namespace aztec3::circuits::kernel::public_kernel