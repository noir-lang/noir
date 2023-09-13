#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/timer.hpp"
#include "barretenberg/common/wasm_export.hpp"
#include "pedersen.hpp"
#include "pedersen_lookup.hpp"

extern "C" {

WASM_EXPORT void pedersen_hash__init()
{
    // TODO: do we need this if we are using lookup-pedersen in merkle trees?
    crypto::generators::init_generator_data();
}

WASM_EXPORT void pedersen__hash_pair(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen_hash::lookup::hash_multiple({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen__hash_multiple(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_hash::lookup::hash_multiple(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen__hash_multiple_with_hash_index(uint8_t const* inputs_buffer,
                                                         uint8_t* output,
                                                         uint32_t hash_index)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_hash::lookup::hash_multiple(to_compress, hash_index);
    barretenberg::fr::serialize_to_buffer(r, output);
}

/**
 * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of
 * nodes that define a merkle tree.
 * e.g.
 * input:  [1][2][3][4]
 * output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)]
 */
WASM_EXPORT uint8_t* pedersen__hash_to_tree(uint8_t const* data)
{
    auto fields = from_buffer<std::vector<grumpkin::fq>>(data);
    auto num_outputs = fields.size() * 2 - 1;
    fields.reserve(num_outputs);

    for (size_t i = 0; fields.size() < num_outputs; i += 2) {
        fields.push_back(crypto::pedersen_hash::lookup::hash_multiple({ fields[i], fields[i + 1] }));
    }

    auto buf_size = 4 + num_outputs * sizeof(grumpkin::fq);
    auto buf = (uint8_t*)aligned_alloc(64, buf_size);
    auto dst = &buf[0];
    write(dst, fields);

    return buf;
}
}