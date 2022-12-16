#pragma once

#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {
namespace lookup {

constexpr size_t BITS_PER_HASH = 512;
constexpr size_t BITS_PER_TABLE = 9;
constexpr size_t BITS_OF_BETA = 192;
constexpr size_t BITS_ON_CURVE = 254;
constexpr size_t BITS_PER_LAST_TABLE = 2;
constexpr size_t PEDERSEN_TABLE_SIZE = (1UL) << BITS_PER_TABLE;
constexpr size_t PEDERSEN_SMALL_TABLE_SIZE = (1UL) << BITS_PER_LAST_TABLE;
constexpr size_t TABLE_MULTIPLICITY = 2; // using group automorphism, we can read from the same table twice
constexpr size_t NUM_PEDERSEN_TABLES_RAW = (BITS_PER_HASH / (BITS_PER_TABLE * TABLE_MULTIPLICITY)) + 1;
constexpr size_t NUM_PEDERSEN_TABLES = NUM_PEDERSEN_TABLES_RAW + (NUM_PEDERSEN_TABLES_RAW & 1);
constexpr size_t PEDERSEN_IV_TABLE_SIZE = (1UL) << 10;
constexpr size_t NUM_PEDERSEN_IV_TABLES = 4;

grumpkin::g1::affine_element get_table_generator(const size_t table_index);

const std::array<grumpkin::fq, 2>& get_endomorphism_scalars();

const std::vector<grumpkin::g1::affine_element>& get_table(const size_t table_index);
const std::vector<grumpkin::g1::affine_element>& get_iv_table();

grumpkin::g1::element hash_single(const grumpkin::fq& input, const bool parity);

grumpkin::fq hash_pair(const grumpkin::fq& left, const grumpkin::fq& right);
grumpkin::g1::element merkle_damgard_compress(const std::vector<grumpkin::fq>& inputs, const size_t iv);

grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index = 0);
std::vector<uint8_t> compress_native(const std::vector<uint8_t>& input);

grumpkin::fq compress_native_buffer_to_field(const std::vector<uint8_t>& input);

template <size_t T> grumpkin::fq compress_native(const std::array<grumpkin::fq, T>& inputs)
{
    std::vector<grumpkin::fq> in(inputs.begin(), inputs.end());
    return compress_native(in);
}

grumpkin::g1::affine_element commit_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index = 0);

} // namespace lookup
} // namespace pedersen
} // namespace crypto