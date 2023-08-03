#include "c_bind.h"

#include "index.hpp"
#include "init.hpp"

#include "aztec3/constants.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using Builder = UltraCircuitBuilder;
using NT = aztec3::utils::types::NativeTypes;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit;
using aztec3::circuits::rollup::native_root_rollup::RootRollupInputs;
using aztec3::circuits::rollup::native_root_rollup::RootRollupPublicInputs;

}  // namespace

// WASM Cbinds
extern "C" {

WASM_EXPORT size_t root_rollup__init_proving_key(uint8_t const** pk_buf)
{
    std::vector<uint8_t> pk_vec(42, 0);

    auto* raw_buf = (uint8_t*)malloc(pk_vec.size());
    memcpy(raw_buf, (void*)pk_vec.data(), pk_vec.size());
    *pk_buf = raw_buf;

    return pk_vec.size();
}

WASM_EXPORT size_t root_rollup__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf)
{
    std::vector<uint8_t> vk_vec(42, 0);
    // TODO remove when proving key is used
    (void)pk_buf;  // unused

    auto* raw_buf = (uint8_t*)malloc(vk_vec.size());
    memcpy(raw_buf, (void*)vk_vec.data(), vk_vec.size());
    *vk_buf = raw_buf;

    return vk_vec.size();
}

WASM_EXPORT uint8_t* root_rollup__sim(uint8_t const* root_rollup_inputs_buf,
                                      size_t* root_rollup_public_inputs_size_out,
                                      uint8_t const** root_rollup_public_inputs_buf)
{
    RootRollupInputs root_rollup_inputs;
    read(root_rollup_inputs_buf, root_rollup_inputs);

    DummyCircuitBuilder builder = DummyCircuitBuilder("root_rollup__sim");
    RootRollupPublicInputs const public_inputs = root_rollup_circuit(builder, root_rollup_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto* raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *root_rollup_public_inputs_buf = raw_public_inputs_buf;
    *root_rollup_public_inputs_size_out = public_inputs_vec.size();
    return builder.alloc_and_serialize_first_failure();
}

WASM_EXPORT size_t root_rollup__verify_proof(uint8_t const* vk_buf, uint8_t const* proof, uint32_t length)
{
    (void)vk_buf;  // unused
    (void)proof;   // unused
    (void)length;  // unused
    return 1U;
}

}  // extern "C"