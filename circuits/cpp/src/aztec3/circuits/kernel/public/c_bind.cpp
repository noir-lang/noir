#include "aztec3/utils/dummy_composer.hpp"
#include "index.hpp"
#include "init.hpp"
#include "c_bind.h"

#include <aztec3/constants.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs.hpp>
#include <aztec3/circuits/abis/public_kernel/public_kernel_inputs_no_previous_kernel.hpp>

#include "barretenberg/srs/reference_string/env_reference_string.hpp"

#include "barretenberg/common/serialize.hpp"
// #include "barretenberg/plonk/composer/turbo_composer.hpp"

namespace {
using Composer = plonk::UltraComposer;
using NT = aztec3::utils::types::NativeTypes;
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputs;
using aztec3::circuits::abis::public_kernel::PublicKernelInputsNoPreviousKernel;
using aztec3::circuits::kernel::public_kernel::native_public_kernel_circuit_no_previous_kernel;
using aztec3::circuits::kernel::public_kernel::native_public_kernel_circuit_private_previous_kernel;
using aztec3::circuits::kernel::public_kernel::native_public_kernel_circuit_public_previous_kernel;
} // namespace

#define WASM_EXPORT __attribute__((visibility("default")))
// WASM Cbinds
extern "C" {

WASM_EXPORT size_t public_kernel__init_proving_key(uint8_t const** pk_buf)
{
    std::vector<uint8_t> pk_vec(42, 0);

    auto raw_buf = (uint8_t*)malloc(pk_vec.size());
    memcpy(raw_buf, (void*)pk_vec.data(), pk_vec.size());
    *pk_buf = raw_buf;

    return pk_vec.size();
}

WASM_EXPORT size_t public_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf)
{
    std::vector<uint8_t> vk_vec(42, 0);
    // TODO remove when proving key is used
    (void)pk_buf; // unused

    auto raw_buf = (uint8_t*)malloc(vk_vec.size());
    memcpy(raw_buf, (void*)vk_vec.data(), vk_vec.size());
    *vk_buf = raw_buf;

    return vk_vec.size();
}

WASM_EXPORT size_t public_kernel__sim(uint8_t const* public_kernel_inputs_buf, uint8_t const** public_inputs_buf)
{
    DummyComposer composer = DummyComposer();

    PublicKernelInputs<NT> public_kernel_inputs;
    read(public_kernel_inputs_buf, public_kernel_inputs);

    KernelCircuitPublicInputs<NT> public_inputs =
        public_kernel_inputs.previous_kernel.public_inputs.is_private
            ? native_public_kernel_circuit_private_previous_kernel(composer, public_kernel_inputs)
            : native_public_kernel_circuit_public_previous_kernel(composer, public_kernel_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *public_inputs_buf = raw_public_inputs_buf;

    return public_inputs_vec.size();
}

WASM_EXPORT size_t public_kernel_no_previous_kernel__sim(uint8_t const* public_kernel_inputs_buf,
                                                         uint8_t const** public_inputs_buf)
{
    DummyComposer composer = DummyComposer();

    PublicKernelInputsNoPreviousKernel<NT> public_kernel_inputs;
    read(public_kernel_inputs_buf, public_kernel_inputs);

    KernelCircuitPublicInputs<NT> public_inputs =
        native_public_kernel_circuit_no_previous_kernel(composer, public_kernel_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *public_inputs_buf = raw_public_inputs_buf;

    return public_inputs_vec.size();
}

} // extern "C"