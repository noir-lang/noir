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
using aztec3::circuits::abis::TxRequest;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsOrdering;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_initial;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_inner;
using aztec3::circuits::kernel::private_kernel::native_private_kernel_circuit_ordering;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;

}  // namespace

// WASM Cbinds

// TODO(dbanks12): might be able to get rid of proving key buffer
WASM_EXPORT size_t private_kernel__init_proving_key(uint8_t const** pk_buf)
{
    std::vector<uint8_t> pk_vec(42, 0);

    auto* raw_buf = (uint8_t*)malloc(pk_vec.size());
    memcpy(raw_buf, (void*)pk_vec.data(), pk_vec.size());
    *pk_buf = raw_buf;

    return pk_vec.size();
}

WASM_EXPORT size_t private_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf)
{
    (void)pk_buf;

    // TODO(dbanks12) actual verification key?
    // NT:VKData vk_data = { 0 };

    std::vector<uint8_t> vk_vec(42, 0);
    // write(vk_vec, vk_data);

    auto* raw_buf = (uint8_t*)malloc(vk_vec.size());
    memcpy(raw_buf, (void*)vk_vec.data(), vk_vec.size());
    *vk_buf = raw_buf;

    return vk_vec.size();
}

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
