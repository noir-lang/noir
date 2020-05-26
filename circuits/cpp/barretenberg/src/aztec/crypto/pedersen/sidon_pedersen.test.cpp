#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include <numeric/random/engine.hpp>

#include "./sidon_pedersen.hpp"

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(sidon_pedersen, endomorphism_test)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;

    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;

    fr exponent = engine.get_random_uint256();

    const auto beta = fq::beta();

    const auto lambda = fr::beta();

    const element P = grumpkin::g1::one;

    affine_element base(P * exponent);
    affine_element first(P * (exponent * lambda));
    affine_element second(P * (exponent * (lambda + 1)));
    EXPECT_EQ(base.x * beta, first.x);
    EXPECT_EQ(base.x * beta.sqr(), second.x);
    EXPECT_EQ(base.y, first.y);
    EXPECT_EQ(-base.y, second.y);
}

TEST(sidon_pedersen, compress_single)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;

    const fq exponent = engine.get_random_uint256();

    const affine_element result(crypto::pedersen::sidon::compress_single(exponent, false));

    const auto& sidon_set = crypto::pedersen::sidon::get_sidon_set();

    const auto mask = crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE - 1;

    uint256_t bits(exponent);

    const fr lambda = grumpkin::fr::beta();

    std::array<element, 3> accumulators;

    for (size_t i = 0; i < 9; ++i) {
        uint64_t slice_a = sidon_set[static_cast<size_t>(bits.data[0] & mask)];
        bits >>= crypto::pedersen::sidon::BITS_PER_TABLE;
        uint64_t slice_b = sidon_set[static_cast<size_t>(bits.data[0] & mask)];
        bits >>= crypto::pedersen::sidon::BITS_PER_TABLE;
        uint64_t slice_c = sidon_set[static_cast<size_t>(bits.data[0] & mask)];

        const element generator = crypto::pedersen::sidon::get_table_generator(9 - i - 1);

        if (i == 0) {
            accumulators[0] = generator * slice_a;
            accumulators[1] = generator * (lambda * slice_b);
            accumulators[2] = generator * ((lambda + 1) * slice_c);
        } else {
            accumulators[0] += (generator * slice_a);
            accumulators[1] += (generator * (lambda * slice_b));
            accumulators[2] += (generator * ((lambda + 1) * slice_c));
        }
        bits >>= crypto::pedersen::sidon::BITS_PER_TABLE;
    }

    const affine_element expected(accumulators[0] + accumulators[1] + accumulators[2]);

    EXPECT_EQ(result, expected);
}

TEST(sidon_pedersen, compress)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;

    const fq left = engine.get_random_uint256();
    const fq right = engine.get_random_uint256();

    const fq result(crypto::pedersen::sidon::compress(left, right));

    const auto& sidon_set = crypto::pedersen::sidon::get_sidon_set();

    const auto compute_expected = [&sidon_set](fq exponent, size_t generator_offset) {
        uint256_t bits(exponent);
        std::array<element, 3> accumulators;
        const fr lambda = grumpkin::fr::beta();
        const auto mask = crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE - 1;

        for (size_t i = 0; i < 9; ++i) {
            uint64_t slice_a = sidon_set[static_cast<size_t>(bits.data[0] & mask)];
            bits >>= crypto::pedersen::sidon::BITS_PER_TABLE;
            uint64_t slice_b = sidon_set[static_cast<size_t>(bits.data[0] & mask)];
            bits >>= crypto::pedersen::sidon::BITS_PER_TABLE;
            uint64_t slice_c = sidon_set[static_cast<size_t>(bits.data[0] & mask)];

            const element generator = crypto::pedersen::sidon::get_table_generator(generator_offset + i);

            if (i == 0) {
                accumulators[0] = generator * slice_a;
                accumulators[1] = generator * (lambda * slice_b);
                accumulators[2] = generator * ((lambda + 1) * slice_c);
            } else {
                accumulators[0] += (generator * slice_a);
                accumulators[1] += (generator * (lambda * slice_b));
                accumulators[2] += (generator * ((lambda + 1) * slice_c));
            }
            bits >>= crypto::pedersen::sidon::BITS_PER_TABLE;
        }
        return (accumulators[0] + accumulators[1] + accumulators[2]);
    };

    const affine_element expected(compute_expected(left, 0) + compute_expected(right, 9));

    EXPECT_EQ(result, expected.x);
}
