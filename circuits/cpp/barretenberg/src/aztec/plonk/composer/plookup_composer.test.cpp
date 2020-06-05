#include "plookup_composer.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include "../proof_system/widgets/transition_widgets/create_dummy_transcript.hpp"
#include "../proof_system/widgets/random_widgets/plookup_widget.hpp"

#include "./plookup_tables/sha256.hpp"

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(plookup_composer, read_sequence_from_multi_table)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    barretenberg::fr input_value = engine.get_random_uint256() & 0xffffffffULL;
    const auto input_index = composer.add_variable(input_value);

    const auto sequence_data =
        waffle::plookup::get_table_values(waffle::PLookupMultiTableId::PEDERSEN_LEFT, input_value);

    const auto sequence_indices =
        composer.read_sequence_from_multi_table(waffle::PLookupMultiTableId::PEDERSEN_LEFT, sequence_data, input_index);

    std::vector<barretenberg::fr> expected_x;
    std::vector<barretenberg::fr> expected_y;

    const size_t num_lookups =
        (256 + crypto::pedersen::sidon::BITS_PER_TABLE - 1) / crypto::pedersen::sidon::BITS_PER_TABLE;

    EXPECT_EQ(num_lookups, sequence_indices[0].size());

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
        EXPECT_EQ(composer.get_variable(sequence_indices[0][i]), expected_scalars[i]);
        EXPECT_EQ(composer.get_variable(sequence_indices[1][i]), expected_x[i]);
        EXPECT_EQ(composer.get_variable(sequence_indices[2][i]), expected_y[i]);
    }

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(plookup_composer, test_no_lookup_proof)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));
            uint32_t result_idx = composer.add_variable(fr(left ^ right));

            uint32_t add_idx = composer.add_variable(fr(left) + fr(right) + composer.get_variable(result_idx));
            composer.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(plookup_composer, test_elliptic_gate)
{
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    affine_element p1 = crypto::pedersen::get_generator(0);
    affine_element p2 = crypto::pedersen::get_generator(1);
    affine_element p3(element(p1) + element(p2));

    uint32_t x1 = composer.add_variable(p1.x);
    uint32_t y1 = composer.add_variable(p1.y);
    uint32_t x2 = composer.add_variable(p2.x);
    uint32_t y2 = composer.add_variable(p2.y);
    uint32_t x3 = composer.add_variable(p3.x);
    uint32_t y3 = composer.add_variable(p3.y);

    waffle::ecc_add_gate gate{ x1, y1, x2, y2, x3, y3, 1, 1 };
    composer.create_ecc_add_gate(gate);

    affine_element p2_endo = p2;
    p2_endo.x *= grumpkin::fq::beta();
    p3 = affine_element(element(p1) + element(p2_endo));
    x3 = composer.add_variable(p3.x);
    y3 = composer.add_variable(p3.y);
    gate = waffle::ecc_add_gate{ x1, y1, x2, y2, x3, y3, grumpkin::fq::beta(), 1 };
    composer.create_ecc_add_gate(gate);

    p2_endo.x *= grumpkin::fq::beta();
    p3 = affine_element(element(p1) - element(p2_endo));
    x3 = composer.add_variable(p3.x);
    y3 = composer.add_variable(p3.y);
    gate = waffle::ecc_add_gate{ x1, y1, x2, y2, x3, y3, grumpkin::fq::beta().sqr(), -1 };
    composer.create_ecc_add_gate(gate);

    composer.create_dummy_gate();
    composer.create_dummy_gate();

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}