#include "aztec3/utils/dummy_composer.hpp"
#include "index.hpp"
#include "c_bind.h"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using DummyComposer = aztec3::utils::DummyComposer;
using aztec3::circuits::abis::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::abis::MergeRollupInputs;
using aztec3::circuits::rollup::merge::merge_rollup_circuit;

} // namespace
#define WASM_EXPORT __attribute__((visibility("default")))
// WASM Cbinds
extern "C" {

WASM_EXPORT uint8_t* merge_rollup__sim(uint8_t const* merge_rollup_inputs_buf,
                                       size_t* merge_rollup_public_inputs_size_out,
                                       uint8_t const** merge_rollup_public_inputs_buf)
{
    DummyComposer composer = DummyComposer();
    MergeRollupInputs<NT> merge_rollup_inputs;
    read(merge_rollup_inputs_buf, merge_rollup_inputs);

    BaseOrMergeRollupPublicInputs public_inputs = merge_rollup_circuit(composer, merge_rollup_inputs);

    // serialize public inputs to bytes vec
    std::vector<uint8_t> public_inputs_vec;
    write(public_inputs_vec, public_inputs);
    // copy public inputs to output buffer
    auto raw_public_inputs_buf = (uint8_t*)malloc(public_inputs_vec.size());
    memcpy(raw_public_inputs_buf, (void*)public_inputs_vec.data(), public_inputs_vec.size());
    *merge_rollup_public_inputs_buf = raw_public_inputs_buf;
    *merge_rollup_public_inputs_size_out = public_inputs_vec.size();

    info("merge_rollup__sim: composer.failed() = ", composer.failed());
    info("merge_rollup__sim: composer.get_first_failure() = ", composer.get_first_failure());

    return composer.alloc_and_serialize_first_failure();
}
} // extern "C"