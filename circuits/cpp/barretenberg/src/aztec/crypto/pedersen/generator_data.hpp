#pragma once
#include <array>
#include <tuple>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {

struct generator_index_t {
    size_t index;
    size_t sub_index;
    bool operator<(const generator_index_t& y) const
    {
        return std::tie(index, sub_index) < std::tie(y.index, y.sub_index);
    }
};

static constexpr generator_index_t DEFAULT_GEN_1 = { 0, 0 };
static constexpr generator_index_t DEFAULT_GEN_2 = { 0, 1 };
static constexpr generator_index_t DEFAULT_GEN_3 = { 0, 2 };
static constexpr generator_index_t DEFAULT_GEN_4 = { 0, 3 };

struct fixed_base_ladder {
    grumpkin::g1::affine_element one;
    grumpkin::g1::affine_element three;
    grumpkin::fq q_x_1;
    grumpkin::fq q_x_2;
    grumpkin::fq q_y_1;
    grumpkin::fq q_y_2;
};

/**
 * The number of bits in each precomputed lookup table. Regular pedersen hashes use 254 bits, some other
 * fixed-base scalar mul subroutines (e.g. verifying schnorr signatures) use 256 bits.
 *
 * When representing an n-bit integer via a WNAF with a window size of b-bits,
 * one requires a minimum of min = (n/b + 1) windows to represent any integer
 * (The last "window" will essentially be a bit saying if the integer is odd or even)
 * if n = 256 and b = 2, min = 129 windows
 */
constexpr size_t bit_length = 256;
constexpr size_t quad_length = bit_length / 2 + 1;
typedef std::array<fixed_base_ladder, quad_length> ladder_t;

struct generator_data {
    grumpkin::g1::affine_element generator;
    grumpkin::g1::affine_element aux_generator;
    grumpkin::g1::affine_element skew_generator;
    ladder_t ladder;
    ladder_t aux_ladder;
    ladder_t hash_ladder;

    const fixed_base_ladder* get_ladder(size_t num_bits) const;
    const fixed_base_ladder* get_hash_ladder(size_t num_bits) const;
};

void init_generator_data();
const fixed_base_ladder* get_g1_ladder(const size_t num_bits);
generator_data const& get_generator_data(generator_index_t index);

} // namespace pedersen
} // namespace crypto
