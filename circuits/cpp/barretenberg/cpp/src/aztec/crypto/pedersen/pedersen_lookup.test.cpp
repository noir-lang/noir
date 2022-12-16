#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include <numeric/random/engine.hpp>

#include "./pedersen_lookup.hpp"

namespace {
auto& engine = numeric::random::get_debug_engine();
}

auto compute_expected(const grumpkin::fq exponent, size_t generator_offset)
{
    uint256_t bits(exponent);
    std::array<grumpkin::g1::element, 2> accumulators;
    const auto lambda = grumpkin::fr::cube_root_of_unity();
    const auto mask = crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE - 1;

    for (size_t i = 0; i < 15; ++i) {
        const auto slice_a = static_cast<size_t>(bits.data[0] & mask) + 1;
        bits >>= crypto::pedersen::lookup::BITS_PER_TABLE;
        const auto slice_b = static_cast<size_t>(bits.data[0] & mask) + 1;

        const auto generator = crypto::pedersen::lookup::get_table_generator(generator_offset + i);

        if (i == 0) {
            accumulators[0] = generator * (lambda * slice_a);
            accumulators[1] = generator * grumpkin::fr(slice_b);
        } else {
            accumulators[0] += (generator * (lambda * slice_a));
            if (i < 14) {
                accumulators[1] += (generator * grumpkin::fr(slice_b));
            }
        }
        bits >>= crypto::pedersen::lookup::BITS_PER_TABLE;
    }
    return (accumulators[0] + accumulators[1]);
}

TEST(pedersen_lookup, endomorphism_test)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;

    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;

    fr exponent = engine.get_random_uint256();

    const auto beta = fq::cube_root_of_unity();

    const auto lambda = fr::cube_root_of_unity();

    const element P = grumpkin::g1::one;

    affine_element base(P * exponent);
    affine_element first(P * (exponent * lambda));
    affine_element second(P * (exponent * (lambda + 1)));
    EXPECT_EQ(base.x * beta, first.x);
    EXPECT_EQ(base.x * beta.sqr(), second.x);
    EXPECT_EQ(base.y, first.y);
    EXPECT_EQ(-base.y, second.y);
}

TEST(pedersen_lookup, hash_single)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;

    const fq exponent = engine.get_random_uint256();

    const affine_element result(crypto::pedersen::lookup::hash_single(exponent, false));

    const auto mask = crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE - 1;

    uint256_t bits(exponent);

    const fr lambda = grumpkin::fr::cube_root_of_unity();

    std::array<element, 2> accumulators;

    for (size_t i = 0; i < 15; ++i) {
        const auto slice_a = static_cast<size_t>(bits.data[0] & mask) + 1;
        bits >>= crypto::pedersen::lookup::BITS_PER_TABLE;
        const auto slice_b = static_cast<size_t>(bits.data[0] & mask) + 1;

        const element generator = crypto::pedersen::lookup::get_table_generator(i);

        if (i == 0) {
            accumulators[0] = generator * (lambda * slice_a);
            accumulators[1] = generator * (slice_b);
        } else {
            accumulators[0] += (generator * (lambda * slice_a));
            if (i < 14) {
                accumulators[1] += (generator * (slice_b));
            }
        }
        bits >>= crypto::pedersen::lookup::BITS_PER_TABLE;
    }

    const affine_element expected(accumulators[0] + accumulators[1]);

    EXPECT_EQ(result, expected);
}

TEST(pedersen_lookup, hash_pair)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::g1::affine_element affine_element;

    const fq left = engine.get_random_uint256();
    const fq right = engine.get_random_uint256();

    const fq result(crypto::pedersen::lookup::hash_pair(left, right));

    const affine_element expected(compute_expected(left, 0) + compute_expected(right, 15));

    EXPECT_EQ(result, expected.x);
}

TEST(pedersen_lookup, merkle_damgard_compress)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;
    typedef grumpkin::g1::affine_element affine_element;

    const size_t m = 3, iv = 10;
    std::vector<fq> inputs;
    for (size_t i = 0; i < m; i++) {
        inputs.push_back(engine.get_random_uint256());
    }

    const auto result = crypto::pedersen::lookup::merkle_damgard_compress(inputs, iv);

    fq intermediate = (grumpkin::g1::affine_one * fr(iv + 1)).x;
    for (size_t i = 0; i < m; i++) {
        intermediate = affine_element(compute_expected(intermediate, 0) + compute_expected(inputs[i], 15)).x;
    }

    EXPECT_EQ(affine_element(result).x,
              affine_element(compute_expected(intermediate, 0) + compute_expected(fq(m), 15)).x);
}
