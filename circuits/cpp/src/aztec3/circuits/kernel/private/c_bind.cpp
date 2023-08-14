#include "c_bind.h"

#include "index.hpp"
#include "utils.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>

namespace {
using Builder = UltraCircuitBuilder;
using NT = aztec3::utils::types::NativeTypes;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::TxRequest;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit;
using aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner;
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

// TODO(dbanks12): comment about how public_inputs is a confusing name
// returns size of public inputs
WASM_EXPORT uint8_t* private_kernel__sim_init(uint8_t const* tx_request_buf,
                                              uint8_t const* private_call_buf,
                                              size_t* private_kernel_public_inputs_size_out,
                                              uint8_t const** private_kernel_public_inputs_buf)
{
    DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_init");

    PrivateCallData<NT> private_call_data;
    serialize::read(private_call_buf, private_call_data);

    TxRequest<NT> tx_request;
    serialize::read(tx_request_buf, tx_request);

    PrivateKernelInputsInit<NT> const private_inputs = PrivateKernelInputsInit<NT>{
        .tx_request = tx_request,
        .private_call = private_call_data,
    };

    auto public_inputs = native_private_kernel_circuit_initial(builder, private_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    serialize::write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto* raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *private_kernel_public_inputs_buf = raw_public_inputs_buf;
    *private_kernel_public_inputs_size_out = public_inputs_vec.size();
    return builder.alloc_and_serialize_first_failure();
}

WASM_EXPORT uint8_t* private_kernel__sim_inner(uint8_t const* previous_kernel_buf,
                                               uint8_t const* private_call_buf,
                                               size_t* private_kernel_public_inputs_size_out,
                                               uint8_t const** private_kernel_public_inputs_buf)
{
    DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_inner");
    PrivateCallData<NT> private_call_data;
    serialize::read(private_call_buf, private_call_data);

    PreviousKernelData<NT> previous_kernel;
    serialize::read(previous_kernel_buf, previous_kernel);

    PrivateKernelInputsInner<NT> const private_inputs = PrivateKernelInputsInner<NT>{
        .previous_kernel = previous_kernel,
        .private_call = private_call_data,
    };

    auto public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    serialize::write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto* raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *private_kernel_public_inputs_buf = raw_public_inputs_buf;
    *private_kernel_public_inputs_size_out = public_inputs_vec.size();
    return builder.alloc_and_serialize_first_failure();
}

CBIND(private_kernel__sim_ordering, [](PreviousKernelData<NT> previous_kernel) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_ordering");
    auto const& public_inputs = native_private_kernel_circuit_ordering(builder, previous_kernel);
    return builder.result_or_error(public_inputs);
});
