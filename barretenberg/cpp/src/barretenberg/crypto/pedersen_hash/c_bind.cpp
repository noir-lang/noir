#include "c_bind.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "pedersen.hpp"

extern "C" {

WASM_EXPORT void pedersen__hash_with_hash_index(uint8_t const* inputs_buffer, uint32_t hash_index, uint8_t* output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    crypto::GeneratorContext<curve::Grumpkin> ctx; // todo fix
    ctx.offset = static_cast<size_t>(hash_index);
    auto r = crypto::pedersen_hash::hash(to_hash, ctx);
    barretenberg::fr::serialize_to_buffer(r, output);
}
}