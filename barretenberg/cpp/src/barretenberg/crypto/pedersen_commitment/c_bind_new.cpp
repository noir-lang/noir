#include "barretenberg/common/serialize.hpp"
#include "c_bind.hpp"
#include "pedersen.hpp"
#include "pedersen_lookup.hpp"

extern "C" {

using namespace barretenberg;

WASM_EXPORT void pedersen___init()
{
    crypto::generators::init_generator_data();
}

WASM_EXPORT void pedersen___compress_fields(fr::in_buf left, fr::in_buf right, fr::out_buf result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen_commitment::compress_native({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen___plookup_compress_fields(fr::in_buf left, fr::in_buf right, fr::out_buf result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen_commitment::lookup::compress_native({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen___compress(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_commitment::compress_native(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen___plookup_compress(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_commitment::lookup::compress_native(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen___compress_with_hash_index(fr::vec_in_buf inputs_buffer,
                                                     uint32_t const* hash_index,
                                                     fr::out_buf output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen_commitment::compress_native(to_compress, ntohl(*hash_index));
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen___commit(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    grumpkin::g1::affine_element pedersen_hash = crypto::pedersen_commitment::commit_native(to_compress);

    serialize::write(output, pedersen_hash);
}

WASM_EXPORT void pedersen___plookup_commit(fr::vec_in_buf inputs_buffer, fr::out_buf output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    grumpkin::g1::affine_element pedersen_hash = crypto::pedersen_commitment::lookup::commit_native(to_compress);

    serialize::write(output, pedersen_hash);
}

WASM_EXPORT void pedersen___plookup_commit_with_hash_index(fr::vec_in_buf inputs_buffer,
                                                           uint32_t const* hash_index,
                                                           fr::out_buf output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    grumpkin::g1::affine_element pedersen_hash =
        crypto::pedersen_commitment::lookup::commit_native(to_compress, ntohl(*hash_index));

    serialize::write(output, pedersen_hash);
}

WASM_EXPORT void pedersen___buffer_to_field(uint8_t const* data, fr::out_buf r)
{
    std::vector<uint8_t> to_compress;
    read(data, to_compress);
    auto output = crypto::pedersen_commitment::compress_native(to_compress);
    write(r, output);
}
}