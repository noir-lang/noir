#pragma once

#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {
namespace sidon {

constexpr size_t BITS_PER_HASH = 512;
constexpr size_t BITS_PER_TABLE = 10;
constexpr size_t PEDERSEN_TABLE_SIZE = (1UL) << BITS_PER_TABLE;
constexpr size_t TABLE_MULTIPLICITY = 3; // using our sidon sequences, we can read from the same table three times
constexpr size_t NUM_PEDERSEN_TABLES =
    (BITS_PER_HASH + (BITS_PER_TABLE * TABLE_MULTIPLICITY)) / (BITS_PER_TABLE * TABLE_MULTIPLICITY);

grumpkin::g1::affine_element get_table_generator(const size_t table_index);

const std::array<grumpkin::fq, 2>& get_endomorphism_scalars();

const std::vector<uint64_t>& get_sidon_set();

const std::vector<grumpkin::g1::affine_element>& get_table(const size_t table_index);

grumpkin::g1::element compress_single(const grumpkin::fq& input, const bool parity);

grumpkin::fq compress_native(const grumpkin::fq& left, const grumpkin::fq& right);
grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs);
template <size_t T> grumpkin::fq compress_native(const std::array<grumpkin::fq, T>& inputs)
{
    std::vector<grumpkin::fq> in(inputs.begin(), inputs.end());
    return compress_native(in);
}

grumpkin::g1::affine_element commit_native(const std::vector<grumpkin::fq>& inputs);

} // namespace sidon
} // namespace pedersen
} // namespace crypto