#include "pedersen.hpp"
#include <common/serialize.hpp>
#include <common/mem.hpp>
#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pedersen_compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen::compress_native({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen_compress(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen::compress_native(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen_compress_with_hash_index(uint8_t const* inputs_buffer, uint8_t* output, uint32_t hash_index)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen::compress_native(to_compress, hash_index);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen_buffer_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> to_compress(data, data + length);

    std::vector<uint8_t> output = crypto::pedersen::compress_native(to_compress);
    auto result = barretenberg::fr::serialize_from_buffer(output.data());
    barretenberg::fr::serialize_to_buffer(result, r);
}

// Given a buffer containing 64 byte leaves, return a new buffer containing the leaf hashes and all pairs of nodes that
// define a merkle tree.
WASM_EXPORT size_t pedersen_hash_to_tree(uint8_t const* data, size_t length, uint8_t** output)
{
    auto num_leaves = length / 64;
    std::vector<std::vector<uint8_t>> results;
    results.reserve(num_leaves * 2 - 2);

    // First compute leaf hashes.
    for (size_t i = 0; i < length; i += 64) {
        std::vector<uint8_t> to_compress(data + i, data + i + 64);
        std::vector<uint8_t> output = crypto::pedersen::compress_native(to_compress);
        results.push_back(std::move(output));
    }

    // Then compute layers of tree node hashes.
    for (size_t i = 0; i < (num_leaves - 1) * 2; i += 2) {
        auto lhs = from_buffer<barretenberg::fr>(results[i]);
        auto rhs = from_buffer<barretenberg::fr>(results[i + 1]);
        auto r = crypto::pedersen::compress_native({ lhs, rhs });
        results.push_back(to_buffer(r));
    }

    size_t buf_size = results.size() * 32;
    uint8_t* buf = (uint8_t*)aligned_alloc(64, buf_size);
    for (size_t i = 0; i < results.size(); ++i) {
        memcpy(&buf[i * 32], results[i].data(), 32);
    }

    *output = buf;
    return buf_size;
}

/*
WASM_EXPORT size_t pedersen_hash_to_tree(uint8_t const* data, size_t length, uint8_t** output)
{
    auto num_leaves = length / 64;
    std::vector<barretenberg::fr> results;
    results.reserve(num_leaves * 2 - 2);

    // First compute leaf hashes.
    for (size_t i = 0; i < length; i += 64) {
        std::vector<uint8_t> to_compress(data + i, data + i + 64);
        std::vector<uint8_t> output = crypto::pedersen::compress_native(to_compress);
        results.push_back(std::move(output));
    }

    // Then compute layers of tree node hashes.
    for (size_t i = 0; i < (num_leaves - 1) * 2; i += 2) {
        auto lhs = from_buffer<barretenberg::fr>(results[i]);
        auto rhs = from_buffer<barretenberg::fr>(results[i + 1]);
        auto r = crypto::pedersen::compress_native({ lhs, rhs });
        results.push_back(to_buffer(r));
    }

    size_t buf_size = results.size() * 32;
    uint8_t* buf = (uint8_t*)aligned_alloc(64, buf_size);
    for (size_t i = 0; i < results.size(); ++i) {
        memcpy(&buf[i * 32], results[i].data(), 32);
    }

    *output = buf;
    return buf_size;
}
*/
}