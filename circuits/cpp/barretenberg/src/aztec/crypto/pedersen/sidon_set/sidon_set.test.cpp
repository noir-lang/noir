#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include <numeric/random/engine.hpp>

#include "./sidon_set.hpp"

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(sidon, compute_nearest_safe_prime)
{
    uint64_t inputs[8]{ 4, 14, 220, 1000, 1002, 2048, 7473, 8096 };
    uint64_t expecteds[8]{ 7, 19, 223, 1019, 1019, 2063, 7487, 8111 };

    for (size_t i = 0; i < 8; ++i) {
        uint64_t result = crypto::pedersen::sidon::compute_nearest_safe_prime(inputs[i]);
        EXPECT_EQ(result, expecteds[i]);
    }
}

TEST(sidon, compute_prime_factors)
{
    uint64_t input = 8192;
    uint64_t expected[1]{ 2 };
    auto factors = crypto::pedersen::sidon::compute_prime_factors(input);
    EXPECT_EQ(factors.size(), 1U);
    for (size_t i = 0; i < 1; ++i) {
        EXPECT_EQ(factors[i], expected[i]);
    }

    input = 16383;

    uint64_t expected2[3]{ 3, 43, 127 };
    factors = crypto::pedersen::sidon::compute_prime_factors(input);
    EXPECT_EQ(factors.size(), 3U);
    EXPECT_EQ(factors[0], expected2[0]);
    EXPECT_EQ(factors[1], expected2[1]);
    EXPECT_EQ(factors[2], expected2[2]);
}

TEST(sidon, compute_generator)
{

    constexpr uint64_t set_size = 1000;
    constexpr uint64_t q = 1019;

    typedef barretenberg::field<crypto::pedersen::sidon::SidonFqParams<q>> fq;
    typedef barretenberg::field2<fq, crypto::pedersen::sidon::SidonFq2Params<fq>> fq2;

    const auto result = crypto::pedersen::sidon::get_sidon_generator<set_size>();

    fq2 accumulator = result;
    for (size_t i = 1; i < q * q - 1; ++i) {
        EXPECT_EQ(accumulator == fq2::one(), false);
        accumulator *= result;
    }
    EXPECT_EQ(accumulator, fq2::one());
}

TEST(sidon, compute_sidon_set)
{
    constexpr uint64_t set_size = 1000;

    const auto set = crypto::pedersen::sidon::compute_sidon_set<set_size>();

    EXPECT_EQ(set.size(), set_size);

    std::vector<uint64_t> sums;
    for (size_t i = 0; i < set_size; ++i) {
        for (size_t j = i; j < set_size; ++j) {
            sums.push_back(set[i] + set[j]);
        }
    }

    std::sort(sums.begin(), sums.end());

    for (size_t i = 1; i < sums.size(); ++i) {
        EXPECT_EQ(sums[i] != sums[i - 1], true);
    }
}