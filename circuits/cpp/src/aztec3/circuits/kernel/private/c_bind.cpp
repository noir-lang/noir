#include "c_bind.h"

#include "index.hpp"
#include "utils.hpp"

#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_inner.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_ordering.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>

namespace {
using Builder = UltraCircuitBuilder;
using NT = aztec3::utils::types::NativeTypes;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsOrdering;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_initial;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_inner;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_ordering;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;

}  // namespace

// WASM Cbinds

CBIND(private_kernel__dummy_previous_kernel, []() { return dummy_previous_kernel(); });

CBIND(private_kernel__sim_init, [](PrivateKernelInputsInit<NT> private_inputs) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_init");
    auto const& public_inputs = native_private_kernel_circuit_initial(builder, private_inputs);
    return builder.result_or_error(public_inputs);
});

CBIND(private_kernel__sim_inner, [](PrivateKernelInputsInner<NT> private_inputs) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_inner");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);
    return builder.result_or_error(public_inputs);
});

CBIND(private_kernel__sim_ordering, [](PrivateKernelInputsOrdering<NT> private_inputs) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_ordering");
    auto const& public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);
    return builder.result_or_error(public_inputs);
});
