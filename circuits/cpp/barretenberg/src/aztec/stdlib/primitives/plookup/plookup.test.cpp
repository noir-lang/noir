#include "plookup.hpp"
#include "../byte_array/byte_array.hpp"

#include <stdlib/types/plookup.hpp>

#include <gtest/gtest.h>
#include <plonk/composer/plookup_composer.hpp>

#include <numeric/random/engine.hpp>
#include <crypto/pedersen/sidon_pedersen.hpp>

namespace test_stdlib_plookups {
using namespace barretenberg;
using namespace plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using namespace plonk::stdlib::types::plookup;

TEST(stdlib_plookup, pedersen_lookup_left)
{
    Composer composer = Composer();

    barretenberg::fr input_value = engine.get_random_uint256() & 0xffffffffULL;
    field_ct input = witness_ct(&composer, input_value);

    const auto sequence =
        plonk::stdlib::plookup::read_sequence_from_table(waffle::PLookupMultiTableId::PEDERSEN_LEFT, input);

    std::vector<barretenberg::fr> expected_x;
    std::vector<barretenberg::fr> expected_y;

    const size_t num_lookups =
        (256 + crypto::pedersen::sidon::BITS_PER_TABLE - 1) / crypto::pedersen::sidon::BITS_PER_TABLE;

    EXPECT_EQ(num_lookups, sequence[0].size());

    std::vector<barretenberg::fr> expected_scalars;
    expected_x.resize(num_lookups);
    expected_y.resize(num_lookups);
    expected_scalars.resize(num_lookups);

    {
        const size_t num_rounds = (num_lookups + 2) / 3;
        uint256_t bits(input_value);

        const auto mask = crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE - 1;

        for (size_t i = 0; i < num_rounds; ++i) {
            const auto& table = crypto::pedersen::sidon::get_table(i);
            const size_t index = i * 3;

            uint64_t slice_a = ((bits >> (index * 10)) & mask).data[0];
            expected_x[index] = (table[slice_a].x);
            expected_y[index] = (table[slice_a].y);
            expected_scalars[index] = slice_a;

            uint64_t slice_b = ((bits >> ((index + 1) * 10)) & mask).data[0];
            expected_x[index + 1] = (table[slice_b].x);
            expected_y[index + 1] = (table[slice_b].y);
            expected_scalars[index + 1] = slice_b;

            if (i < 8) {
                uint64_t slice_c = ((bits >> ((index + 2) * 10)) & mask).data[0];
                expected_x[index + 2] = (table[slice_c].x);
                expected_y[index + 2] = (table[slice_c].y);
                expected_scalars[index + 2] = slice_c;
            }
        }
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        expected_scalars[i] += (expected_scalars[i + 1] * crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE);
    }
    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(sequence[0][i].get_value(), expected_scalars[i]);
        EXPECT_EQ(sequence[1][i].get_value(), expected_x[i]);
        EXPECT_EQ(sequence[2][i].get_value(), expected_y[i]);
    }

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, pedersen_lookup_right)
{
    Composer composer = Composer();

    barretenberg::fr input_value = engine.get_random_uint256() & 0xffffffffULL;
    field_ct input = witness_ct(&composer, input_value);

    const auto sequence =
        plonk::stdlib::plookup::read_sequence_from_table(waffle::PLookupMultiTableId::PEDERSEN_RIGHT, input);

    std::vector<barretenberg::fr> expected_x;
    std::vector<barretenberg::fr> expected_y;

    const size_t num_lookups =
        (256 + crypto::pedersen::sidon::BITS_PER_TABLE - 1) / crypto::pedersen::sidon::BITS_PER_TABLE;

    EXPECT_EQ(num_lookups, sequence[0].size());

    std::vector<barretenberg::fr> expected_scalars;
    expected_x.resize(num_lookups);
    expected_y.resize(num_lookups);
    expected_scalars.resize(num_lookups);

    {
        const size_t num_rounds = (num_lookups + 2) / 3;
        uint256_t bits(input_value);

        const auto mask = crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE - 1;

        for (size_t i = 0; i < num_rounds; ++i) {
            const auto& table = crypto::pedersen::sidon::get_table(i + num_rounds);
            const size_t index = i * 3;

            uint64_t slice_a = ((bits >> (index * 10)) & mask).data[0];
            expected_x[index] = (table[slice_a].x);
            expected_y[index] = (table[slice_a].y);
            expected_scalars[index] = slice_a;

            uint64_t slice_b = ((bits >> ((index + 1) * 10)) & mask).data[0];
            expected_x[index + 1] = (table[slice_b].x);
            expected_y[index + 1] = (table[slice_b].y);
            expected_scalars[index + 1] = slice_b;

            if (i < 8) {
                uint64_t slice_c = ((bits >> ((index + 2) * 10)) & mask).data[0];
                expected_x[index + 2] = (table[slice_c].x);
                expected_y[index + 2] = (table[slice_c].y);
                expected_scalars[index + 2] = slice_c;
            }
        }
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        expected_scalars[i] += (expected_scalars[i + 1] * crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE);
    }
    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(sequence[0][i].get_value(), expected_scalars[i]);
        EXPECT_EQ(sequence[1][i].get_value(), expected_x[i]);
        EXPECT_EQ(sequence[2][i].get_value(), expected_y[i]);
    }

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, uint32_xor)
{
    Composer composer = Composer();

    const size_t num_lookups = (32 + 5) / 6;

    uint256_t left_value = (engine.get_random_uint256() & 0xffffffffULL);
    uint256_t right_value = (engine.get_random_uint256() & 0xffffffffULL);

    field_ct left = witness_ct(&composer, barretenberg::fr(left_value));
    field_ct right = witness_ct(&composer, barretenberg::fr(right_value));

    const auto sequence =
        plonk::stdlib::plookup::read_sequence_from_table(waffle::PLookupMultiTableId::UINT32_XOR, left, right, true);

    const auto left_slices = numeric::slice_input(left_value, 1 << 6);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6);
    std::vector<uint256_t> out_expected(num_lookups);
    std::vector<uint256_t> left_expected(num_lookups);
    std::vector<uint256_t> right_expected(num_lookups);

    for (size_t i = 0; i < left_slices.size(); ++i) {
        out_expected[i] = left_slices[i] ^ right_slices[i];
        left_expected[i] = left_slices[i];
        right_expected[i] = right_slices[i];
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        out_expected[i] += out_expected[i + 1] * (1 << 6);
        left_expected[i] += left_expected[i + 1] * (1 << 6);
        right_expected[i] += right_expected[i + 1] * (1 << 6);
    }

    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(sequence[0][i].get_value(), barretenberg::fr(left_expected[i]));
        EXPECT_EQ(sequence[1][i].get_value(), barretenberg::fr(right_expected[i]));
        EXPECT_EQ(sequence[2][i].get_value(), barretenberg::fr(out_expected[i]));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, uint32_and)
{
    Composer composer = Composer();

    const size_t num_lookups = (32 + 5) / 6;

    uint256_t left_value = (engine.get_random_uint256() & 0xffffffffULL);
    uint256_t right_value = (engine.get_random_uint256() & 0xffffffffULL);

    field_ct left = witness_ct(&composer, barretenberg::fr(left_value));
    field_ct right = witness_ct(&composer, barretenberg::fr(right_value));

    const auto sequence =
        plonk::stdlib::plookup::read_sequence_from_table(waffle::PLookupMultiTableId::UINT32_AND, left, right, true);
    const auto left_slices = numeric::slice_input(left_value, 1 << 6, num_lookups);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6, num_lookups);
    std::vector<uint256_t> out_expected(num_lookups);
    std::vector<uint256_t> left_expected(num_lookups);
    std::vector<uint256_t> right_expected(num_lookups);

    for (size_t i = 0; i < left_slices.size(); ++i) {
        out_expected[i] = left_slices[i] & right_slices[i];
        left_expected[i] = left_slices[i];
        right_expected[i] = right_slices[i];
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        out_expected[i] += out_expected[i + 1] * (1 << 6);
        left_expected[i] += left_expected[i + 1] * (1 << 6);
        right_expected[i] += right_expected[i + 1] * (1 << 6);
    }

    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(sequence[0][i].get_value(), barretenberg::fr(left_expected[i]));
        EXPECT_EQ(sequence[1][i].get_value(), barretenberg::fr(right_expected[i]));
        EXPECT_EQ(sequence[2][i].get_value(), barretenberg::fr(out_expected[i]));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

} // namespace test_stdlib_plookups
