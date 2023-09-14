#include "c_bind.h"

#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using Builder = UltraCircuitBuilder;
using NT = aztec3::utils::types::NativeTypes;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::kernel::public_kernel::native_public_kernel_circuit_private_previous_kernel;
using aztec3::circuits::kernel::public_kernel::native_public_kernel_circuit_public_previous_kernel;

// WASM Cbinds
CBIND(public_kernel__sim, [](PublicKernelInputs<NT> const& public_kernel_inputs) {
    DummyBuilder builder = DummyBuilder("public_kernel__sim");
    KernelCircuitPublicInputs<NT> const result =
        public_kernel_inputs.previous_kernel.public_inputs.is_private
            ? native_public_kernel_circuit_private_previous_kernel(builder, public_kernel_inputs)
            : native_public_kernel_circuit_public_previous_kernel(builder, public_kernel_inputs);
    return builder.result_or_error(result);
});

}  // namespace
