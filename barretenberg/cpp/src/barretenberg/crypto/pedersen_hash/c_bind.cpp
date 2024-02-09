#include "c_bind.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "pedersen.hpp"

extern "C" {

WASM_EXPORT void pedersen_hash(bb::fr::vec_in_buf inputs_buffer, uint32_t const* hash_index, bb::fr::out_buf output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    bb::crypto::GeneratorContext<bb::curve::Grumpkin> ctx;
    ctx.offset = static_cast<size_t>(ntohl(*hash_index));
    auto r = bb::crypto::pedersen_hash::hash(to_hash, ctx);
    bb::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen_hashes(bb::fr::vec_in_buf inputs_buffer, uint32_t const* hash_index, bb::fr::out_buf output)
{
    std::vector<grumpkin::fq> to_hash;
    read(inputs_buffer, to_hash);
    bb::crypto::GeneratorContext<bb::curve::Grumpkin> ctx;
    ctx.offset = static_cast<size_t>(ntohl(*hash_index));
    const size_t numHashes = to_hash.size() / 2;
    std::vector<grumpkin::fq> results;
    size_t count = 0;
    while (count < numHashes) {
        auto r = bb::crypto::pedersen_hash::hash({ to_hash[count * 2], to_hash[count * 2 + 1] }, ctx);
        results.push_back(r);
        ++count;
    }
    write(output, results);
}

WASM_EXPORT void pedersen_hash_buffer(uint8_t const* input_buffer, uint32_t const* hash_index, bb::fr::out_buf output)
{
    std::vector<uint8_t> to_hash;
    read(input_buffer, to_hash);
    bb::crypto::GeneratorContext<bb::curve::Grumpkin> ctx;
    ctx.offset = static_cast<size_t>(ntohl(*hash_index));
    auto r = bb::crypto::pedersen_hash::hash_buffer(to_hash, ctx);
    bb::fr::serialize_to_buffer(r, output);
}
}