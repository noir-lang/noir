#include "init.hpp"
#include "native_public_kernel_circuit_no_previous_kernel.hpp"
#include "native_public_kernel_circuit_public_previous_kernel.hpp"
#include "native_public_kernel_circuit_private_previous_kernel.hpp"
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <gtest/gtest.h>

namespace {
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputsNoPreviousKernel;
using NT = aztec3::utils::types::NativeTypes;
} // namespace

namespace aztec3::circuits::kernel::public_kernel {

// TEST(public_kernel_tests, no_previous_kernel)
// {
//     DummyComposer dc;
//     PublicKernelInputsNoPreviousKernel<NT> inputs;
//     auto public_inputs = native_public_kernel_circuit_no_previous_kernel(dc, inputs);
//     ASSERT_TRUE(dc.failed());
// }

// TEST(public_kernel_tests, public_previous_kernel)
// {
//     DummyComposer dc;
//     PublicKernelInputs<NT> inputs;
//     auto public_inputs = native_public_kernel_circuit_public_previous_kernel(dc, inputs);
//     ASSERT_TRUE(dc.failed());
// }

// TEST(public_kernel_tests, private_previous_kernel)
// {
//     DummyComposer dc;
//     PublicKernelInputs<NT> inputs;
//     auto public_inputs = native_public_kernel_circuit_private_previous_kernel(dc, inputs);
//     ASSERT_TRUE(dc.failed());
// }
} // namespace aztec3::circuits::kernel::public_kernel