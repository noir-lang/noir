#include <stdlib/primitives/plookup/plookup.hpp>
#include "ultra_composer.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include <numeric/uintx/uintx.hpp>
#include "../proof_system/widgets/transition_widgets/create_dummy_transcript.hpp"
#include "../proof_system/widgets/random_widgets/plookup_widget.hpp"
#include "./plookup_tables/sha256.hpp"

using namespace barretenberg;

namespace ultra_composer_tests {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using plookup::ColumnIdx;
using plookup::MultiTableId;

std::vector<uint32_t> add_variables(waffle::UltraComposer& composer, std::vector<fr> variables)
{
    std::vector<uint32_t> res;
    for (size_t i = 0; i < variables.size(); i++) {
        res.emplace_back(composer.add_variable(variables[i]));
    }
    return res;
}
TEST(ultra_composer, create_gates_from_plookup_accumulators)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

    barretenberg::fr input_value = fr::random_element();
    const fr input_hi = uint256_t(input_value).slice(126, 256);
    const fr input_lo = uint256_t(input_value).slice(0, 126);
    const auto input_hi_index = composer.add_variable(input_hi);
    const auto input_lo_index = composer.add_variable(input_lo);

    const auto sequence_data_hi = plookup::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_HI, input_hi);
    const auto sequence_data_lo = plookup::get_lookup_accumulators(MultiTableId::PEDERSEN_LEFT_LO, input_lo);

    const auto lookup_witnesses_hi = composer.create_gates_from_plookup_accumulators(
        MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
    const auto lookup_witnesses_lo = composer.create_gates_from_plookup_accumulators(
        MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);

    std::vector<barretenberg::fr> expected_x;
    std::vector<barretenberg::fr> expected_y;

    const size_t num_lookups_hi =
        (128 + crypto::pedersen::lookup::BITS_PER_TABLE) / crypto::pedersen::lookup::BITS_PER_TABLE;
    const size_t num_lookups_lo = 126 / crypto::pedersen::lookup::BITS_PER_TABLE;
    const size_t num_lookups = num_lookups_hi + num_lookups_lo;

    EXPECT_EQ(num_lookups_hi, lookup_witnesses_hi[ColumnIdx::C1].size());
    EXPECT_EQ(num_lookups_lo, lookup_witnesses_lo[ColumnIdx::C1].size());

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

            uint64_t slice_a = ((bits >> (index * 9)) & mask).data[0];
            expected_x[index] = (table[(size_t)slice_a].x);
            expected_y[index] = (table[(size_t)slice_a].y);
            expected_scalars[index] = slice_a;

            if (i < 14) {
                uint64_t slice_b = ((bits >> ((index + 1) * 9)) & mask).data[0];
                expected_x[index + 1] = (table[(size_t)slice_b].x);
                expected_y[index + 1] = (table[(size_t)slice_b].y);
                expected_scalars[index + 1] = slice_b;
            }
        }
    }

    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        expected_scalars[i] += (expected_scalars[i + 1] * crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }

    size_t hi_shift = 126;
    const fr hi_cumulative = composer.get_variable(lookup_witnesses_hi[ColumnIdx::C1][0]);
    for (size_t i = 0; i < num_lookups_lo; ++i) {
        const fr hi_mult = fr(uint256_t(1) << hi_shift);
        EXPECT_EQ(composer.get_variable(lookup_witnesses_lo[ColumnIdx::C1][i]) + (hi_cumulative * hi_mult),
                  expected_scalars[i]);
        EXPECT_EQ(composer.get_variable(lookup_witnesses_lo[ColumnIdx::C2][i]), expected_x[i]);
        EXPECT_EQ(composer.get_variable(lookup_witnesses_lo[ColumnIdx::C3][i]), expected_y[i]);
        hi_shift -= crypto::pedersen::lookup::BITS_PER_TABLE;
    }

    for (size_t i = 0; i < num_lookups_hi; ++i) {
        EXPECT_EQ(composer.get_variable(lookup_witnesses_hi[ColumnIdx::C1][i]), expected_scalars[i + num_lookups_lo]);
        EXPECT_EQ(composer.get_variable(lookup_witnesses_hi[ColumnIdx::C2][i]), expected_x[i + num_lookups_lo]);
        EXPECT_EQ(composer.get_variable(lookup_witnesses_hi[ColumnIdx::C3][i]), expected_y[i + num_lookups_lo]);
    }

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(ultra_composer, test_no_lookup_proof)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

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

TEST(ultra_composer, test_elliptic_gate)
{
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;
    waffle::UltraComposer composer = waffle::UltraComposer();

    affine_element p1 = crypto::pedersen::get_generator_data({ 0, 0 }).generator;

    affine_element p2 = crypto::pedersen::get_generator_data({ 0, 1 }).generator;
    affine_element p3(element(p1) + element(p2));

    uint32_t x1 = composer.add_variable(p1.x);
    uint32_t y1 = composer.add_variable(p1.y);
    uint32_t x2 = composer.add_variable(p2.x);
    uint32_t y2 = composer.add_variable(p2.y);
    uint32_t x3 = composer.add_variable(p3.x);
    uint32_t y3 = composer.add_variable(p3.y);

    waffle::ecc_add_gate gate{ x1, y1, x2, y2, x3, y3, 1, 1 };
    composer.create_ecc_add_gate(gate);

    grumpkin::fq beta = grumpkin::fq::cube_root_of_unity();
    affine_element p2_endo = p2;
    p2_endo.x *= beta;
    p3 = affine_element(element(p1) + element(p2_endo));
    x3 = composer.add_variable(p3.x);
    y3 = composer.add_variable(p3.y);
    gate = waffle::ecc_add_gate{ x1, y1, x2, y2, x3, y3, beta, 1 };
    composer.create_ecc_add_gate(gate);

    p2_endo.x *= beta;
    p3 = affine_element(element(p1) - element(p2_endo));
    x3 = composer.add_variable(p3.x);
    y3 = composer.add_variable(p3.y);
    gate = waffle::ecc_add_gate{ x1, y1, x2, y2, x3, y3, beta.sqr(), -1 };
    composer.create_ecc_add_gate(gate);

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(ultra_composer, non_trivial_tag_permutation)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(b);
    auto d_idx = composer.add_variable(a);

    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });

    composer.create_tag(1, 2);
    composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    composer.assign_tag(b_idx, 1);
    composer.assign_tag(c_idx, 2);
    composer.assign_tag(d_idx, 2);

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}
TEST(ultra_composer, non_trivial_tag_permutation_and_cycles)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    fr a = fr::random_element();
    fr c = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(a);
    composer.assert_equal(a_idx, b_idx);
    auto c_idx = composer.add_variable(c);
    auto d_idx = composer.add_variable(c);
    composer.assert_equal(c_idx, d_idx);
    auto e_idx = composer.add_variable(a);
    auto f_idx = composer.add_variable(a);
    composer.assert_equal(e_idx, f_idx);
    auto g_idx = composer.add_variable(c);
    auto h_idx = composer.add_variable(c);
    composer.assert_equal(g_idx, h_idx);

    composer.create_tag(1, 2);
    composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    composer.assign_tag(c_idx, 1);
    composer.assign_tag(e_idx, 2);
    composer.assign_tag(g_idx, 2);

    composer.create_add_gate({ b_idx, a_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ c_idx, g_idx, composer.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ e_idx, f_idx, composer.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(ultra_composer, bad_tag_permutation)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(b);
    auto d_idx = composer.add_variable(a + 1);

    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, 1, 1, 0, 0 });
    composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, 1, 1, 0, -1 });

    composer.create_tag(1, 2);
    composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    composer.assign_tag(b_idx, 1);
    composer.assign_tag(c_idx, 2);
    composer.assign_tag(d_idx, 2);
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, false);
}

// same as above but with turbocomposer to check reason of failue is really tag mismatch
TEST(ultra_composer, bad_tag_turbo_permutation)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(b);
    auto d_idx = composer.add_variable(a + 1);

    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, 1, 1, 0, 0 });
    composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, 1, 1, 0, -1 });

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(ultra_composer, sort_widget)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(4);

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(c);
    auto d_idx = composer.add_variable(d);
    composer.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(ultra_composer, sort_with_edges_gate)
{

    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(4);
    fr e = fr(5);
    fr f = fr(6);
    fr g = fr(7);
    fr h = fr(8);

    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto a_idx = composer.add_variable(a);
        auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        composer.create_sort_constraint_with_edges({ a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, a, h);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
    }

    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto a_idx = composer.add_variable(a);
        auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        composer.create_sort_constraint_with_edges({ a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, a, g);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto a_idx = composer.add_variable(a);
        auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        composer.create_sort_constraint_with_edges({ a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, b, h);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto a_idx = composer.add_variable(a);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        auto b2_idx = composer.add_variable(fr(15));
        composer.create_sort_constraint_with_edges({ a_idx, b2_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, b, h);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto idx = add_variables(composer, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                             26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });
        composer.create_sort_constraint_with_edges(idx, 1, 45);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto idx = add_variables(composer, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                             26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });

        composer.create_sort_constraint_with_edges(idx, 1, 29);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
}

TEST(ultra_composer, range_constraint)
{
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto indices = add_variables(composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 8);
        }
        // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
        composer.create_sort_constraint(indices);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto indices = add_variables(composer, { 3 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 3);
        }
        // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
        composer.create_dummy_constraints(indices);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto indices = add_variables(composer, { 1, 2, 3, 4, 5, 6, 8, 25 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 8);
        }
        composer.create_sort_constraint(indices);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto indices =
            add_variables(composer, { 1, 2, 3, 4, 5, 6, 10, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 19, 51 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 128);
        }
        composer.create_dummy_constraints(indices);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto indices =
            add_variables(composer, { 1, 2, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 79);
        }
        composer.create_dummy_constraints(indices);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    {
        waffle::UltraComposer composer = waffle::UltraComposer();
        auto indices =
            add_variables(composer, { 1, 0, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 79);
        }
        composer.create_dummy_constraints(indices);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
}

TEST(ultra_composer, range_with_gates)
{

    waffle::UltraComposer composer = waffle::UltraComposer();
    auto idx = add_variables(composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
    for (size_t i = 0; i < idx.size(); i++) {
        composer.create_new_range_constraint(idx[i], 8);
    }

    composer.create_add_gate({ idx[0], idx[1], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -3 });
    composer.create_add_gate({ idx[2], idx[3], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -7 });
    composer.create_add_gate({ idx[4], idx[5], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -11 });
    composer.create_add_gate({ idx[6], idx[7], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -15 });
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
TEST(ultra_composer, sort_widget_complex)
{
    {

        waffle::UltraComposer composer = waffle::UltraComposer();
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 11, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(composer.add_variable(a[i]));
        composer.create_sort_constraint(ind);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
    }
    {

        waffle::UltraComposer composer = waffle::UltraComposer();
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 16, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(composer.add_variable(a[i]));
        composer.create_sort_constraint(ind);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
}
TEST(ultra_composer, sort_widget_neg)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(8);

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(c);
    auto d_idx = composer.add_variable(d);
    composer.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, false);
}
TEST(ultra_composer, composed_range_constraint)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    auto c = fr::random_element();
    auto d = uint256_t(c).slice(0, 133);
    auto e = fr(d);
    auto a_idx = composer.add_variable(fr(e));
    composer.create_add_gate({ a_idx, composer.zero_idx, composer.zero_idx, 1, 0, 0, -fr(e) });
    composer.decompose_into_default_range(a_idx, 134);
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(ultra_composer, non_native_field_multiplication)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

    fq a = fq::random_element();
    fq b = fq::random_element();
    uint256_t modulus = fq::modulus;

    uint1024_t a_big = uint512_t(uint256_t(a));
    uint1024_t b_big = uint512_t(uint256_t(b));
    uint1024_t p_big = uint512_t(uint256_t(modulus));

    uint1024_t q_big = (a_big * b_big) / p_big;
    uint1024_t r_big = (a_big * b_big) % p_big;

    uint256_t q(q_big.lo.lo);
    uint256_t r(r_big.lo.lo);

    const auto split_into_limbs = [&](const uint512_t& input) {
        constexpr size_t NUM_BITS = 68;
        std::array<fr, 5> limbs;
        limbs[0] = input.slice(0, NUM_BITS).lo;
        limbs[1] = input.slice(NUM_BITS * 1, NUM_BITS * 2).lo;
        limbs[2] = input.slice(NUM_BITS * 2, NUM_BITS * 3).lo;
        limbs[3] = input.slice(NUM_BITS * 3, NUM_BITS * 4).lo;
        limbs[4] = fr(input.lo);
        return limbs;
    };

    const auto get_limb_witness_indices = [&](const std::array<fr, 5>& limbs) {
        std::array<uint32_t, 5> limb_indices;
        limb_indices[0] = composer.add_variable(limbs[0]);
        limb_indices[1] = composer.add_variable(limbs[1]);
        limb_indices[2] = composer.add_variable(limbs[2]);
        limb_indices[3] = composer.add_variable(limbs[3]);
        limb_indices[4] = composer.add_variable(limbs[4]);
        return limb_indices;
    };
    const uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (68 * 4);
    auto modulus_limbs = split_into_limbs(BINARY_BASIS_MODULUS - uint512_t(modulus));

    const auto a_indices = get_limb_witness_indices(split_into_limbs(uint256_t(a)));
    const auto b_indices = get_limb_witness_indices(split_into_limbs(uint256_t(b)));
    const auto q_indices = get_limb_witness_indices(split_into_limbs(uint256_t(q)));
    const auto r_indices = get_limb_witness_indices(split_into_limbs(uint256_t(r)));

    waffle::UltraComposer::non_native_field_witnesses inputs{
        a_indices, b_indices, q_indices, r_indices, modulus_limbs, fr(uint256_t(modulus)),
    };
    const auto [lo_1_idx, hi_1_idx] = composer.evaluate_non_native_field_multiplication(inputs);
    composer.range_constrain_two_limbs(lo_1_idx, hi_1_idx, 70, 70);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(ultra_composer, rom)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

    uint32_t rom_values[8]{
        composer.add_variable(fr::random_element()), composer.add_variable(fr::random_element()),
        composer.add_variable(fr::random_element()), composer.add_variable(fr::random_element()),
        composer.add_variable(fr::random_element()), composer.add_variable(fr::random_element()),
        composer.add_variable(fr::random_element()), composer.add_variable(fr::random_element()),
    };

    size_t rom_id = composer.create_ROM_array(8);

    for (size_t i = 0; i < 8; ++i) {
        composer.set_ROM_element(rom_id, i, rom_values[i]);
    }

    uint32_t a_idx = composer.read_ROM_array(rom_id, composer.add_variable(5));
    EXPECT_EQ(a_idx != rom_values[5], true);
    uint32_t b_idx = composer.read_ROM_array(rom_id, composer.add_variable(4));
    uint32_t c_idx = composer.read_ROM_array(rom_id, composer.add_variable(1));

    const auto d_value = composer.get_variable(a_idx) + composer.get_variable(b_idx) + composer.get_variable(c_idx);
    uint32_t d_idx = composer.add_variable(d_value);

    composer.create_big_add_gate({
        a_idx,
        b_idx,
        c_idx,
        d_idx,
        1,
        1,
        1,
        -1,
        0,
    });

    auto prover = composer.create_prover();
    std::cout << "prover n = " << composer.n << std::endl;

    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
} // namespace ultra_composer_tests