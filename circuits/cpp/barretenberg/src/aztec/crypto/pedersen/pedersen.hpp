#pragma once
#include <array>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "./generator_data.hpp"
#include "./fixed_base_scalar_mul.hpp"

namespace crypto {
namespace pedersen {

grumpkin::g1::element hash_single(const barretenberg::fr& in, generator_index_t const& index);

grumpkin::g1::affine_element commit_native(const std::vector<grumpkin::fq>& elements, const size_t hash_index = 0);

grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index = 0);

template <size_t T> grumpkin::fq compress_native(const std::array<grumpkin::fq, T>& inputs)
{
    std::vector<grumpkin::fq> converted(inputs.begin(), inputs.end());
    return commit_native(converted).x;
}

grumpkin::fq compress_native(const std::vector<uint8_t>& input);

} // namespace pedersen
} // namespace crypto
