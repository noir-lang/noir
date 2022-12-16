#include "plookup.hpp"
#include "../byte_array/byte_array.hpp"
#include <gtest/gtest.h>
#include <plonk/composer/ultra_composer.hpp>
#include <numeric/random/engine.hpp>
#include <numeric/bitop/rotate.hpp>
#include <crypto/pedersen/pedersen_lookup.hpp>
#include <stdlib/primitives/biggroup/biggroup.hpp>
#include <stdlib/primitives/bigfield/bigfield.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/curves/secp256k1.hpp>

namespace test_stdlib_plookups {
using namespace barretenberg;
using namespace plonk;
using namespace plookup;

// Defining ultra-specific types for local testing.
using Composer = waffle::UltraComposer;
using field_ct = stdlib::field_t<Composer>;
using witness_ct = stdlib::witness_t<Composer>;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using stdlib::plookup_read;

TEST(stdlib_plookup, pedersen_lookup_left)
{
    Composer composer = Composer();

    barretenberg::fr input_value = fr::random_element();
    field_ct input_hi = witness_ct(&composer, uint256_t(input_value).slice(126, 256));
    field_ct input_lo = witness_ct(&composer, uint256_t(input_value).slice(0, 126));

    const auto lookup_hi = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_HI, input_hi);
    const auto lookup_lo = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_LO, input_lo);

    std::vector<barretenberg::fr> expected_x;
    std::vector<barretenberg::fr> expected_y;

    const size_t num_lookups_hi =
        (128 + crypto::pedersen::lookup::BITS_PER_TABLE) / crypto::pedersen::lookup::BITS_PER_TABLE;
    const size_t num_lookups_lo = 126 / crypto::pedersen::lookup::BITS_PER_TABLE;

    EXPECT_EQ(num_lookups_hi, lookup_hi[ColumnIdx::C1].size());
    EXPECT_EQ(num_lookups_lo, lookup_lo[ColumnIdx::C1].size());

    const size_t num_lookups = num_lookups_hi + num_lookups_lo;
    std::vector<barretenberg::fr> expected_scalars;
    expected_x.resize(num_lookups);
    expected_y.resize(num_lookups);
    expected_scalars.resize(num_lookups);

    {
        const size_t num_rounds = (num_lookups + 1) / 2;
        uint256_t bits(input_value);

        const auto mask = crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE - 1;

        for (size_t i = 0; i < num_rounds; ++i) {
            const auto& table = crypto::pedersen::lookup::get_table(i);
            const size_t index = i * 2;

            size_t slice_a =
                static_cast<size_t>(((bits >> (index * crypto::pedersen::lookup::BITS_PER_TABLE)) & mask).data[0]);
            expected_x[index] = (table[slice_a].x);
            expected_y[index] = (table[slice_a].y);
            expected_scalars[index] = slice_a;

            if (i < 14) {
                size_t slice_b = static_cast<size_t>(
                    ((bits >> ((index + 1) * crypto::pedersen::lookup::BITS_PER_TABLE)) & mask).data[0]);
                expected_x[index + 1] = (table[slice_b].x);
                expected_y[index + 1] = (table[slice_b].y);
                expected_scalars[index + 1] = slice_b;
            }
        }
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        expected_scalars[i] += (expected_scalars[i + 1] * crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }
    size_t hi_shift = 126;
    const fr hi_cumulative = lookup_hi[ColumnIdx::C1][0].get_value();
    for (size_t i = 0; i < num_lookups_lo; ++i) {
        const fr hi_mult = fr(uint256_t(1) << hi_shift);
        EXPECT_EQ(lookup_lo[ColumnIdx::C1][i].get_value() + (hi_cumulative * hi_mult), expected_scalars[i]);
        EXPECT_EQ(lookup_lo[ColumnIdx::C2][i].get_value(), expected_x[i]);
        EXPECT_EQ(lookup_lo[ColumnIdx::C3][i].get_value(), expected_y[i]);
        hi_shift -= crypto::pedersen::lookup::BITS_PER_TABLE;
    }
    for (size_t i = 0; i < num_lookups_hi; ++i) {
        EXPECT_EQ(lookup_hi[ColumnIdx::C1][i].get_value(), expected_scalars[i + num_lookups_lo]);
        EXPECT_EQ(lookup_hi[ColumnIdx::C2][i].get_value(), expected_x[i + num_lookups_lo]);
        EXPECT_EQ(lookup_hi[ColumnIdx::C3][i].get_value(), expected_y[i + num_lookups_lo]);
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

    barretenberg::fr input_value = fr::random_element();
    field_ct input_hi = witness_ct(&composer, uint256_t(input_value).slice(126, 256));
    field_ct input_lo = witness_ct(&composer, uint256_t(input_value).slice(0, 126));

    const auto lookup_hi = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_RIGHT_HI, input_hi);
    const auto lookup_lo = plookup_read::get_lookup_accumulators(MultiTableId::PEDERSEN_RIGHT_LO, input_lo);

    std::vector<barretenberg::fr> expected_x;
    std::vector<barretenberg::fr> expected_y;

    const size_t num_lookups_hi =
        (128 + crypto::pedersen::lookup::BITS_PER_TABLE) / crypto::pedersen::lookup::BITS_PER_TABLE;
    const size_t num_lookups_lo = 126 / crypto::pedersen::lookup::BITS_PER_TABLE;

    EXPECT_EQ(num_lookups_hi, lookup_hi[ColumnIdx::C1].size());
    EXPECT_EQ(num_lookups_lo, lookup_lo[ColumnIdx::C1].size());

    const size_t num_lookups = num_lookups_hi + num_lookups_lo;
    std::vector<barretenberg::fr> expected_scalars;
    expected_x.resize(num_lookups);
    expected_y.resize(num_lookups);
    expected_scalars.resize(num_lookups);

    {
        const size_t num_rounds = (num_lookups + 1) / 2;
        uint256_t bits(input_value);

        const auto mask = crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE - 1;

        for (size_t i = 0; i < num_rounds; ++i) {
            const auto& table = crypto::pedersen::lookup::get_table(i + num_rounds);
            const size_t index = i * 2;

            size_t slice_a =
                static_cast<size_t>(((bits >> (index * crypto::pedersen::lookup::BITS_PER_TABLE)) & mask).data[0]);
            expected_x[index] = (table[slice_a].x);
            expected_y[index] = (table[slice_a].y);
            expected_scalars[index] = slice_a;

            if (i < 14) {
                size_t slice_b = static_cast<size_t>(
                    ((bits >> ((index + 1) * crypto::pedersen::lookup::BITS_PER_TABLE)) & mask).data[0]);
                expected_x[index + 1] = (table[slice_b].x);
                expected_y[index + 1] = (table[slice_b].y);
                expected_scalars[index + 1] = slice_b;
            }
        }
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        expected_scalars[i] += (expected_scalars[i + 1] * crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }
    size_t hi_shift = 126;
    const fr hi_cumulative = lookup_hi[ColumnIdx::C1][0].get_value();
    for (size_t i = 0; i < num_lookups_lo; ++i) {
        const fr hi_mult = fr(uint256_t(1) << hi_shift);
        EXPECT_EQ(lookup_lo[ColumnIdx::C1][i].get_value() + (hi_cumulative * hi_mult), expected_scalars[i]);
        EXPECT_EQ(lookup_lo[ColumnIdx::C2][i].get_value(), expected_x[i]);
        EXPECT_EQ(lookup_lo[ColumnIdx::C3][i].get_value(), expected_y[i]);
        hi_shift -= crypto::pedersen::lookup::BITS_PER_TABLE;
    }
    for (size_t i = 0; i < num_lookups_hi; ++i) {
        EXPECT_EQ(lookup_hi[ColumnIdx::C1][i].get_value(), expected_scalars[i + num_lookups_lo]);
        EXPECT_EQ(lookup_hi[ColumnIdx::C2][i].get_value(), expected_x[i + num_lookups_lo]);
        EXPECT_EQ(lookup_hi[ColumnIdx::C3][i].get_value(), expected_y[i + num_lookups_lo]);
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

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::UINT32_XOR, left, right, true);

    const auto left_slices = numeric::slice_input(left_value, 1 << 6, num_lookups);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6, num_lookups);

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
        EXPECT_EQ(lookup[ColumnIdx::C1][i].get_value(), barretenberg::fr(left_expected[i]));
        EXPECT_EQ(lookup[ColumnIdx::C2][i].get_value(), barretenberg::fr(right_expected[i]));
        EXPECT_EQ(lookup[ColumnIdx::C3][i].get_value(), barretenberg::fr(out_expected[i]));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, blake2s_xor_rotate_16)
{
    Composer composer = Composer();

    const size_t num_lookups = 6;

    uint256_t left_value = (engine.get_random_uint256() & 0xffffffffULL);
    uint256_t right_value = (engine.get_random_uint256() & 0xffffffffULL);

    field_ct left = witness_ct(&composer, barretenberg::fr(left_value));
    field_ct right = witness_ct(&composer, barretenberg::fr(right_value));

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::BLAKE_XOR_ROTATE_16, left, right, true);

    const auto left_slices = numeric::slice_input(left_value, 1 << 6, num_lookups);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6, num_lookups);

    std::vector<barretenberg::fr> out_expected(num_lookups);
    std::vector<barretenberg::fr> left_expected(num_lookups);
    std::vector<barretenberg::fr> right_expected(num_lookups);

    for (size_t i = 0; i < left_slices.size(); ++i) {
        if (i == 2) {
            uint32_t a = static_cast<uint32_t>(left_slices[i]);
            uint32_t b = static_cast<uint32_t>(right_slices[i]);
            uint32_t c = numeric::rotate32(a ^ b, 4);
            out_expected[i] = uint256_t(c);
        } else {
            out_expected[i] = uint256_t(left_slices[i]) ^ uint256_t(right_slices[i]);
        }
        left_expected[i] = left_slices[i];
        right_expected[i] = right_slices[i];
    }

    /*
     * The following out coefficients are the the ones multiplied for computing the cumulative intermediate terms
     * in the expected output. If the column_3_coefficients for this table are (a0, a1, ..., a5), then the
     * out_coefficients must be (a5/a4, a4/a3, a3/a2, a2/a1, a1/a0). Note that these are stored in reverse orde
     * for simplicity.
     */
    std::vector<barretenberg::fr> out_coefficients{
        (1 << 6), (barretenberg::fr(1) / barretenberg::fr(1 << 22)), (1 << 2), (1 << 6), (1 << 6)
    };

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        out_expected[i] += out_expected[i + 1] * out_coefficients[i];
        left_expected[i] += left_expected[i + 1] * (1 << 6);
        right_expected[i] += right_expected[i + 1] * (1 << 6);
    }

    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(lookup[ColumnIdx::C1][i].get_value(), left_expected[i]);
        EXPECT_EQ(lookup[ColumnIdx::C2][i].get_value(), right_expected[i]);
        EXPECT_EQ(lookup[ColumnIdx::C3][i].get_value(), out_expected[i]);
    }

    /*
     * Note that we multiply the output of the lookup table (lookup[Column::Idx}0]) by 2^{16} because
     * while defining the table we had set the coefficient of s0 to 1, so to correct that, we need to multiply by a
     * constant.
     */
    auto mul_constant = barretenberg::fr(1 << 16);
    barretenberg::fr lookup_output = lookup[ColumnIdx::C3][0].get_value() * mul_constant;
    uint32_t xor_rotate_output = numeric::rotate32(uint32_t(left_value) ^ uint32_t(right_value), 16);
    EXPECT_EQ(barretenberg::fr(uint256_t(xor_rotate_output)), lookup_output);

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, blake2s_xor_rotate_8)
{
    Composer composer = Composer();

    const size_t num_lookups = 6;

    uint256_t left_value = (engine.get_random_uint256() & 0xffffffffULL);
    uint256_t right_value = (engine.get_random_uint256() & 0xffffffffULL);

    field_ct left = witness_ct(&composer, barretenberg::fr(left_value));
    field_ct right = witness_ct(&composer, barretenberg::fr(right_value));

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::BLAKE_XOR_ROTATE_8, left, right, true);

    const auto left_slices = numeric::slice_input(left_value, 1 << 6, num_lookups);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6, num_lookups);

    std::vector<barretenberg::fr> out_expected(num_lookups);
    std::vector<barretenberg::fr> left_expected(num_lookups);
    std::vector<barretenberg::fr> right_expected(num_lookups);

    for (size_t i = 0; i < left_slices.size(); ++i) {
        if (i == 1) {
            uint32_t a = static_cast<uint32_t>(left_slices[i]);
            uint32_t b = static_cast<uint32_t>(right_slices[i]);
            uint32_t c = numeric::rotate32(a ^ b, 2);
            out_expected[i] = uint256_t(c);
        } else {
            out_expected[i] = uint256_t(left_slices[i]) ^ uint256_t(right_slices[i]);
        }
        left_expected[i] = left_slices[i];
        right_expected[i] = right_slices[i];
    }

    auto mul_constant = barretenberg::fr(1 << 24);
    std::vector<barretenberg::fr> out_coefficients{
        (barretenberg::fr(1) / mul_constant), (1 << 4), (1 << 6), (1 << 6), (1 << 6)
    };

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        out_expected[i] += out_expected[i + 1] * out_coefficients[i];
        left_expected[i] += left_expected[i + 1] * (1 << 6);
        right_expected[i] += right_expected[i + 1] * (1 << 6);
    }

    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(lookup[ColumnIdx::C1][i].get_value(), left_expected[i]);
        EXPECT_EQ(lookup[ColumnIdx::C2][i].get_value(), right_expected[i]);
        EXPECT_EQ(lookup[ColumnIdx::C3][i].get_value(), out_expected[i]);
    }

    barretenberg::fr lookup_output = lookup[ColumnIdx::C3][0].get_value() * mul_constant;
    uint32_t xor_rotate_output = numeric::rotate32(uint32_t(left_value) ^ uint32_t(right_value), 8);
    EXPECT_EQ(barretenberg::fr(uint256_t(xor_rotate_output)), lookup_output);

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, blake2s_xor_rotate_7)
{
    Composer composer = Composer();

    const size_t num_lookups = 6;

    uint256_t left_value = (engine.get_random_uint256() & 0xffffffffULL);
    uint256_t right_value = (engine.get_random_uint256() & 0xffffffffULL);

    field_ct left = witness_ct(&composer, barretenberg::fr(left_value));
    field_ct right = witness_ct(&composer, barretenberg::fr(right_value));

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::BLAKE_XOR_ROTATE_7, left, right, true);

    const auto left_slices = numeric::slice_input(left_value, 1 << 6, num_lookups);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6, num_lookups);

    std::vector<barretenberg::fr> out_expected(num_lookups);
    std::vector<barretenberg::fr> left_expected(num_lookups);
    std::vector<barretenberg::fr> right_expected(num_lookups);

    for (size_t i = 0; i < left_slices.size(); ++i) {
        if (i == 1) {
            uint32_t a = static_cast<uint32_t>(left_slices[i]);
            uint32_t b = static_cast<uint32_t>(right_slices[i]);
            uint32_t c = numeric::rotate32(a ^ b, 1);
            out_expected[i] = uint256_t(c);
        } else {
            out_expected[i] = uint256_t(left_slices[i]) ^ uint256_t(right_slices[i]);
        }
        left_expected[i] = left_slices[i];
        right_expected[i] = right_slices[i];
    }

    auto mul_constant = barretenberg::fr(1 << 25);
    std::vector<barretenberg::fr> out_coefficients{
        (barretenberg::fr(1) / mul_constant), (1 << 5), (1 << 6), (1 << 6), (1 << 6)
    };

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        out_expected[i] += out_expected[i + 1] * out_coefficients[i];
        left_expected[i] += left_expected[i + 1] * (1 << 6);
        right_expected[i] += right_expected[i + 1] * (1 << 6);
    }

    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(lookup[ColumnIdx::C1][i].get_value(), left_expected[i]);
        EXPECT_EQ(lookup[ColumnIdx::C2][i].get_value(), right_expected[i]);
        EXPECT_EQ(lookup[ColumnIdx::C3][i].get_value(), out_expected[i]);
    }

    barretenberg::fr lookup_output = lookup[ColumnIdx::C3][0].get_value() * mul_constant;
    uint32_t xor_rotate_output = numeric::rotate32(uint32_t(left_value) ^ uint32_t(right_value), 7);
    EXPECT_EQ(barretenberg::fr(uint256_t(xor_rotate_output)), lookup_output);

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, blake2s_xor)
{
    Composer composer = Composer();

    const size_t num_lookups = 6;

    uint256_t left_value = (engine.get_random_uint256() & 0xffffffffULL);
    uint256_t right_value = (engine.get_random_uint256() & 0xffffffffULL);

    field_ct left = witness_ct(&composer, barretenberg::fr(left_value));
    field_ct right = witness_ct(&composer, barretenberg::fr(right_value));

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::BLAKE_XOR, left, right, true);

    const auto left_slices = numeric::slice_input(left_value, 1 << 6, num_lookups);
    const auto right_slices = numeric::slice_input(right_value, 1 << 6, num_lookups);

    std::vector<uint256_t> out_expected(num_lookups);
    std::vector<uint256_t> left_expected(num_lookups);
    std::vector<uint256_t> right_expected(num_lookups);

    for (size_t i = 0; i < left_slices.size(); ++i) {
        out_expected[i] = left_slices[i] ^ right_slices[i];
        left_expected[i] = left_slices[i];
        right_expected[i] = right_slices[i];
    }

    // Compute ror(a ^ b, 12) from lookup table.
    // t0 = 2^30 a5 + 2^24 a4 + 2^18 a3 + 2^12 a2 + 2^6 a1 + a0
    // t1 = 2^24 a5 + 2^18 a4 + 2^12 a3 + 2^6 a2 + a1
    // t2 = 2^18 a5 + 2^12 a4 + 2^6 a3 + a2
    // t3 = 2^12 a5 + 2^6 a4 + a3
    // t4 = 2^6 a5 + a4
    // t5 = a5
    //
    // output = (t0 - 2^12 t2) * 2^{32 - 12} + t2
    barretenberg::fr lookup_output = lookup[ColumnIdx::C3][2].get_value();
    barretenberg::fr t2_term = barretenberg::fr(1 << 12) * lookup[ColumnIdx::C3][2].get_value();
    lookup_output += barretenberg::fr(1 << 20) * (lookup[ColumnIdx::C3][0].get_value() - t2_term);

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        out_expected[i] += out_expected[i + 1] * (1 << 6);
        left_expected[i] += left_expected[i + 1] * (1 << 6);
        right_expected[i] += right_expected[i + 1] * (1 << 6);
    }

    //
    // The following checks if the xor output rotated by 12 can be computed correctly from basic blake2s_xor.
    //
    auto xor_rotate_output = numeric::rotate32(uint32_t(left_value) ^ uint32_t(right_value), 12);
    EXPECT_EQ(barretenberg::fr(uint256_t(xor_rotate_output)), lookup_output);

    for (size_t i = 0; i < num_lookups; ++i) {
        EXPECT_EQ(lookup[ColumnIdx::C1][i].get_value(), barretenberg::fr(left_expected[i]));
        EXPECT_EQ(lookup[ColumnIdx::C2][i].get_value(), barretenberg::fr(right_expected[i]));
        EXPECT_EQ(lookup[ColumnIdx::C3][i].get_value(), barretenberg::fr(out_expected[i]));
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

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::UINT32_AND, left, right, true);
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
        EXPECT_EQ(lookup[ColumnIdx::C1][i].get_value(), barretenberg::fr(left_expected[i]));
        EXPECT_EQ(lookup[ColumnIdx::C2][i].get_value(), barretenberg::fr(right_expected[i]));
        EXPECT_EQ(lookup[ColumnIdx::C3][i].get_value(), barretenberg::fr(out_expected[i]));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(stdlib_plookup, secp256k1_generator)
{
    using curve = stdlib::secp256k1<Composer>;
    Composer composer = Composer();

    uint256_t input_value = (engine.get_random_uint256() >> 128);

    uint64_t wnaf_entries[18] = { 0 };
    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<129, 1, 8>(&input_value.data[0], &wnaf_entries[0], skew, 0);

    std::vector<uint64_t> naf_values;
    for (size_t i = 0; i < 17; ++i) {
        bool predicate = bool((wnaf_entries[i] >> 31U) & 1U);
        uint64_t offset_entry;
        if (predicate) {
            offset_entry = (127 - (wnaf_entries[i] & 0xffffff));
        } else {
            offset_entry = (128 + (wnaf_entries[i] & 0xffffff));
        }
        naf_values.emplace_back(offset_entry);
    }

    std::vector<field_ct> circuit_naf_values;
    for (size_t i = 0; i < naf_values.size(); ++i) {
        circuit_naf_values.emplace_back(witness_ct(&composer, naf_values[i]));
    }

    std::vector<field_ct> accumulators;
    for (size_t i = 0; i < naf_values.size(); ++i) {
        field_ct t1 = (circuit_naf_values[naf_values.size() - 1 - i]) * field_ct(uint256_t(1) << (i * 8 + 1));
        field_ct t2 = field_ct(255) * field_ct(uint256_t(1) << (i * 8));
        accumulators.emplace_back(t1 - t2);
    }
    field_ct accumulator_field = field_ct::accumulate(accumulators);
    EXPECT_EQ(accumulator_field.get_value(), barretenberg::fr(input_value) + barretenberg::fr(skew));

    for (size_t i = 0; i < 256; ++i) {
        field_ct index(witness_ct(&composer, barretenberg::fr(i)));
        const auto xlo = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_XLO, index);
        const auto xhi = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_XHI, index);
        const auto ylo = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_YLO, index);
        const auto yhi = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_YHI, index);
        curve::fq_ct x = curve::fq_ct(xlo.first, xlo.second, xhi.first, xhi.second);
        curve::fq_ct y = curve::fq_ct(ylo.first, ylo.second, yhi.first, yhi.second);

        const auto res = curve::g1_ct(x, y).get_value();
        curve::fr scalar(i);
        scalar = scalar + scalar;
        scalar = scalar - 255;
        curve::g1::affine_element expec(curve::g1::one * scalar);

        EXPECT_EQ(res, expec);
    }
    curve::g1_ct accumulator;
    {
        const auto xlo = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_XLO, circuit_naf_values[0]);
        const auto xhi = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_XHI, circuit_naf_values[0]);
        const auto ylo = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_YLO, circuit_naf_values[0]);
        const auto yhi = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_YHI, circuit_naf_values[0]);

        curve::fq_ct x = curve::fq_ct(xlo.first, xlo.second, xhi.first, xhi.second);
        curve::fq_ct y = curve::fq_ct(ylo.first, ylo.second, yhi.first, yhi.second);
        accumulator = curve::g1_ct(x, y);
    }
    for (size_t i = 1; i < circuit_naf_values.size(); ++i) {
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();

        const auto xlo = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_XLO, circuit_naf_values[i]);
        const auto xhi = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_XHI, circuit_naf_values[i]);
        const auto ylo = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_YLO, circuit_naf_values[i]);
        const auto yhi = plookup_read::read_pair_from_table(MultiTableId::SECP256K1_YHI, circuit_naf_values[i]);
        curve::fq_ct x = curve::fq_ct(xlo.first, xlo.second, xhi.first, xhi.second);
        curve::fq_ct y = curve::fq_ct(ylo.first, ylo.second, yhi.first, yhi.second);
        accumulator = accumulator.montgomery_ladder(curve::g1_ct(x, y));
    }

    if (skew) {
        accumulator = accumulator - curve::g1_ct::one(&composer);
    }

    curve::g1::affine_element result = accumulator.get_value();
    curve::g1::affine_element expected(curve::g1::one * input_value);
    EXPECT_EQ(result, expected);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stdlib_plookups
