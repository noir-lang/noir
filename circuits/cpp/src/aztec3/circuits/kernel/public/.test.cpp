#include "init.hpp"
#include "native_public_kernel_circuit.hpp"
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <gtest/gtest.h>

using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using NT = aztec3::utils::types::NativeTypes;

namespace aztec3::circuits::kernel::public_kernel {
/**
 * @brief Some private circuit proof (`constructor`, in this case)
 */
TEST(public_kernel_tests, end_values_are_initialised)
{
    DummyComposer dc;
    PublicKernelInputs<NT> inputs;
    auto public_inputs = native_public_kernel_circuit(dc, inputs);

    ASSERT_EQ(public_inputs.end, inputs.previous_kernel.public_inputs.end);
}
} // namespace aztec3::circuits::kernel::public_kernel