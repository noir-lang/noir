#pragma once

#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace crypto {
namespace pedersen_commitment {
namespace lookup {

grumpkin::g1::element merkle_damgard_compress(const std::vector<grumpkin::fq>& inputs, const size_t iv);
grumpkin::g1::element merkle_damgard_compress(const std::vector<grumpkin::fq>& inputs, const std::vector<size_t>& ivs);
grumpkin::g1::element merkle_damgard_tree_compress(const std::vector<grumpkin::fq>& inputs,
                                                   const std::vector<size_t>& ivs);

grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index = 0);
grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs, const std::vector<size_t>& hash_indices);
std::vector<uint8_t> compress_native(const std::vector<uint8_t>& input, const size_t hash_index = 0);

grumpkin::fq compress_native_buffer_to_field(const std::vector<uint8_t>& input, const size_t hash_index = 0);

template <size_t T> grumpkin::fq compress_native(const std::array<grumpkin::fq, T>& inputs)
{
    std::vector<grumpkin::fq> in(inputs.begin(), inputs.end());
    return compress_native(in);
}

grumpkin::g1::affine_element commit_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index = 0);
grumpkin::g1::affine_element commit_native(const std::vector<grumpkin::fq>& inputs,
                                           const std::vector<size_t>& hash_indices);

} // namespace lookup
} // namespace pedersen_commitment
} // namespace crypto