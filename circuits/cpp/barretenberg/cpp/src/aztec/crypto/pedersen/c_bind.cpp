#include "pedersen.hpp"
#include <common/serialize.hpp>
#include <common/timer.hpp>
#include <common/mem.hpp>
#include <common/streams.hpp>
#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pedersen__init()
{
    crypto::pedersen::init_generator_data();
}

WASM_EXPORT void pedersen__compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen::compress_native({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen__compress(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen::compress_native(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen__compress_with_hash_index(uint8_t const* inputs_buffer, uint8_t* output, uint32_t hash_index)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen::compress_native(to_compress, hash_index);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen__buffer_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> to_compress(data, data + length);
    auto output = crypto::pedersen::compress_native(to_compress);
    write(r, output);
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
        fields.push_back(crypto::pedersen::compress_native({ fields[i], fields[i + 1] }));
    }

    auto buf_size = 4 + num_outputs * sizeof(grumpkin::fq);
    auto buf = (uint8_t*)aligned_alloc(64, buf_size);
    auto dst = &buf[0];
    write(dst, fields);

    return buf;
}
}