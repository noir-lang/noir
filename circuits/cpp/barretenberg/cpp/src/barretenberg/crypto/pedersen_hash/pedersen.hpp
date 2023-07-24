#pragma once
#include "../generators/fixed_base_scalar_mul.hpp"
#include "../generators/generator_data.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <array>

namespace crypto {
namespace pedersen_hash {

grumpkin::g1::element hash_single(const barretenberg::fr& in, generators::generator_index_t const& index);

grumpkin::fq hash_multiple(const std::vector<grumpkin::fq>& inputs, const size_t hash_index = 0);

} // namespace pedersen_hash
} // namespace crypto
