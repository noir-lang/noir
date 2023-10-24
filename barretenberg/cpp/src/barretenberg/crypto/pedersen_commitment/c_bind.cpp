#include "c_bind.hpp"
#include "../pedersen_hash/pedersen.hpp"
#include "barretenberg/common/serialize.hpp"
#include "pedersen.hpp"

WASM_EXPORT void pedersen__init() {}
WASM_EXPORT void pedersen__compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen_hash::hash({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen_plookup_compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    pedersen__compress_fields(left, right, result);
}

WASM_EXPORT void pedersen__compress(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_hash::hash(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}
WASM_EXPORT void pedersen_plookup_compress(uint8_t const* inputs_buffer, uint8_t* output)
{
    pedersen__compress(inputs_buffer, output);
}
WASM_EXPORT void pedersen__compress_with_hash_index(uint8_t const* inputs_buffer, uint8_t* output, uint32_t hash_index)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    crypto::GeneratorContext<curve::Grumpkin> ctx; // todo fix
    ctx.offset = static_cast<size_t>(hash_index);
    auto r = crypto::pedersen_hash::hash(to_compress, ctx);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen_plookup_compress_with_hash_index(uint8_t const* inputs_buffer,
                                                           uint8_t* output,
                                                           uint32_t hash_index)
{
    pedersen__compress_with_hash_index(inputs_buffer, output, hash_index);
}
WASM_EXPORT void pedersen__commit(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    grumpkin::g1::affine_element pedersen_hash = crypto::pedersen_commitment::commit_native(to_compress);

    serialize::write(output, pedersen_hash);
}
WASM_EXPORT void pedersen_plookup_commit(uint8_t const* inputs_buffer, uint8_t* output)
{
    pedersen__commit(inputs_buffer, output);
}

WASM_EXPORT void pedersen_plookup_commit_with_hash_index(uint8_t const* inputs_buffer,
                                                         uint8_t* output,
                                                         uint32_t hash_index)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    crypto::GeneratorContext<curve::Grumpkin> ctx;
    ctx.offset = hash_index;
    auto commitment = crypto::pedersen_commitment::commit_native(to_compress, ctx);
    serialize::write(output, commitment);
}

WASM_EXPORT void pedersen__buffer_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> to_compress(data, data + length);
    auto output = crypto::pedersen_hash::hash_buffer(to_compress);
    write(r, output);
}
