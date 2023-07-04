#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <gtest/gtest.h>

#include "../pedersen_hash/pedersen_lookup.hpp"
#include "./pedersen_lookup.hpp"

namespace {
auto& engine = numeric::random::get_debug_engine();
}

auto compute_expected(const grumpkin::fq exponent, size_t generator_offset)
{
    uint256_t bits(exponent);
    std::array<grumpkin::g1::element, 2> accumulators;
    const auto lambda = grumpkin::fr::cube_root_of_unity();
    const auto mask = crypto::pedersen_hash::lookup::PEDERSEN_TABLE_SIZE - 1;

    /**
     * Given an input scalar x, we split it into 9-bit slices:
     * x = ( x_28 || x_27 || ... || x_2 || x_1 || x_0 )
     *
     * Note that the last slice x_28 is a 2-bit slice. Total = 2 + 9 * 28 = 254 bits.
     *
     * Algorithm:
     *     hash = O;
     *     hash += x_0 * G_0  +  x_1 * λ * G_0;
     *     hash += x_2 * G_1  +  x_2 * λ * G_1;
     *     ...
     *     ...
     *     hash += x_26 * G_13  +  x_27 * λ * G_13;
     *     hash += x_27 * G_14;
     *
     * Our lookup tables stores the following:
     *     1 -> (G_0, (λ * G_0))
     *     2 -> (2G_0, 2(λ * G_0))
     *     3 -> (3G_0, 3(λ * G_0))
     *     ...
     *   512 -> (512G_0, 512(λ * G_0))
     */
    for (size_t i = 0; i < (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2); ++i) {
        const auto slice_a = static_cast<size_t>(bits.data[0] & mask) + 1;
        bits >>= crypto::pedersen_hash::lookup::BITS_PER_TABLE;
        const auto slice_b = static_cast<size_t>(bits.data[0] & mask) + 1;

        const auto generator = crypto::pedersen_hash::lookup::get_table_generator(generator_offset + i);

        if (i == 0) {
            accumulators[0] = generator * (lambda * slice_a);
            accumulators[1] = generator * grumpkin::fr(slice_b);
        } else {
            accumulators[0] += (generator * (lambda * slice_a));
            if (i < 14) {
                accumulators[1] += (generator * grumpkin::fr(slice_b));
            }
        }
        bits >>= crypto::pedersen_hash::lookup::BITS_PER_TABLE;
    }
    return (accumulators[0] + accumulators[1]);
}

TEST(pedersen_lookup, zero_one)
{
    auto r =
        crypto::pedersen_commitment::lookup::compress_native({ barretenberg::fr::zero(), barretenberg::fr::one() });
    EXPECT_EQ(format(r), "0x0c5e1ddecd49de44ed5e5798d3f6fb7c71fe3d37f5bee8664cf88a445b5ba0af");
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

    const affine_element result(crypto::pedersen_hash::lookup::hash_single(exponent, false));

    const auto mask = crypto::pedersen_hash::lookup::PEDERSEN_TABLE_SIZE - 1;

    uint256_t bits(exponent);

    const fr lambda = grumpkin::fr::cube_root_of_unity();

    std::array<element, 2> accumulators;

    for (size_t i = 0; i < (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2); ++i) {
        const auto slice_a = static_cast<size_t>(bits.data[0] & mask) + 1;
        bits >>= crypto::pedersen_hash::lookup::BITS_PER_TABLE;
        const auto slice_b = static_cast<size_t>(bits.data[0] & mask) + 1;

        const element generator = crypto::pedersen_hash::lookup::get_table_generator(i);

        if (i == 0) {
            accumulators[0] = generator * (lambda * slice_a);
            accumulators[1] = generator * (slice_b);
        } else {
            accumulators[0] += (generator * (lambda * slice_a));
            if (i < 14) {
                accumulators[1] += (generator * (slice_b));
            }
        }
        bits >>= crypto::pedersen_hash::lookup::BITS_PER_TABLE;
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

    const fq result(crypto::pedersen_hash::lookup::hash_pair(left, right));

    const affine_element expected(compute_expected(left, 0) +
                                  compute_expected(right, (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)));

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

    const auto result = crypto::pedersen_commitment::lookup::merkle_damgard_compress(inputs, iv);

    auto iv_hash = compute_expected((grumpkin::g1::affine_one * fr(iv + 1)).x, 0);
    auto length = compute_expected(fq(m), (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2));
    fq intermediate = affine_element(iv_hash + length).x;
    for (size_t i = 0; i < m; i++) {
        intermediate =
            affine_element(compute_expected(intermediate, 0) +
                           compute_expected(inputs[i], (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)))
                .x;
    }

    EXPECT_EQ(affine_element(result).x, intermediate);
}

TEST(pedersen_lookup, merkle_damgard_compress_multiple_iv)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;
    typedef grumpkin::g1::affine_element affine_element;

    const size_t m = 10;
    std::vector<size_t> ivs;
    std::vector<fq> inputs;
    for (size_t i = 0; i < m; i++) {
        inputs.push_back(engine.get_random_uint256());
        ivs.push_back(engine.get_random_uint8());
    }

    const auto result = crypto::pedersen_commitment::lookup::merkle_damgard_compress(inputs, ivs);

    const size_t initial_iv = 0;
    auto iv_hash = compute_expected((grumpkin::g1::affine_one * fr(initial_iv + 1)).x, 0);

    auto length = compute_expected(fq(m), (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2));
    fq intermediate = affine_element(iv_hash + length).x;

    for (size_t i = 0; i < 2 * m; i++) {
        if ((i & 1) == 0) {
            const auto iv = (grumpkin::g1::affine_one * fr(ivs[i >> 1] + 1)).x;
            intermediate =
                affine_element(compute_expected(intermediate, 0) +
                               compute_expected(iv, (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)))
                    .x;
        } else {
            intermediate = affine_element(compute_expected(intermediate, 0) +
                                          compute_expected(inputs[i >> 1],
                                                           (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)))
                               .x;
        }
    }

    EXPECT_EQ(affine_element(result).x, intermediate);
}

TEST(pedersen_lookup, merkle_damgard_tree_compress)
{
    typedef grumpkin::fq fq;
    typedef grumpkin::fr fr;
    typedef grumpkin::g1::affine_element affine_element;

    const size_t m = 8;
    std::vector<size_t> ivs;
    std::vector<fq> inputs;
    for (size_t i = 0; i < m; i++) {
        inputs.push_back(engine.get_random_uint256());
        ivs.push_back(engine.get_random_uint8());
    }

    const auto result = crypto::pedersen_commitment::lookup::merkle_damgard_tree_compress(inputs, ivs);

    std::vector<fq> temp;
    for (size_t i = 0; i < m; i++) {
        const fq iv_term = (grumpkin::g1::affine_one * fr(ivs[i] + 1)).x;
        temp.push_back(
            affine_element(compute_expected(iv_term, 0) +
                           compute_expected(inputs[i], (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)))
                .x);
    }

    const size_t logm = numeric::get_msb(m);
    for (size_t j = 1; j <= logm; j++) {
        const size_t nodes = (1UL << (logm - j));
        for (size_t i = 0; i < nodes; i++) {
            temp[i] = affine_element(
                          compute_expected(temp[2 * i], 0) +
                          compute_expected(temp[2 * i + 1], (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)))
                          .x;
        }
    }

    EXPECT_EQ(affine_element(result).x,
              affine_element(compute_expected(temp[0], 0) +
                             compute_expected(fq(m), (crypto::pedersen_hash::lookup::NUM_PEDERSEN_TABLES / 2)))
                  .x);
}
