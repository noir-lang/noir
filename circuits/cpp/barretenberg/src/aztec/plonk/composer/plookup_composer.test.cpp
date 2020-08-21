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
std::vector<uint32_t> add_variables(waffle::PLookupComposer& composer, std::vector<fr> variables)
{
    std::vector<uint32_t> res;
    for (size_t i = 0; i < variables.size(); i++) {
        res.emplace_back(composer.add_variable(variables[i]));
    }
    return res;
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

TEST(plookup_composer, non_trivial_tag_permutation)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    fr a = fr::random_element();
    // fr b = -a;

    auto a_idx = composer.add_public_variable(a);
    // auto b_idx = composer.add_variable(b);
    // auto c_idx = composer.add_variable(b);
    // auto d_idx = composer.add_variable(a);

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });

    composer.create_tag(1, 1);
    // composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    // composer.assign_tag(b_idx, 1);
    // composer.assign_tag(c_idx, 2);
    // composer.assign_tag(d_idx, 2);

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}
TEST(plookup_composer, non_trivial_tag_permutation_and_cycles)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
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
    composer.create_dummy_gate();
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

TEST(plookup_composer, bad_tag_permutation)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
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
TEST(plookup_composer, bad_tag_turbo_permutation)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
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

TEST(plookup_composer, sort_widget)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
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

TEST(plookup_composer, sort_with_edges_gate)
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
        waffle::PLookupComposer composer = waffle::PLookupComposer();
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
        waffle::PLookupComposer composer = waffle::PLookupComposer();
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
        waffle::PLookupComposer composer = waffle::PLookupComposer();
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
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        auto a_idx = composer.add_variable(a);
        // auto b_idx = composer.add_variable(b);
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
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        auto idx = add_variables(composer, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                             26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });
        composer.create_sort_constraint_with_edges(idx, 1, 45);
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
        // auto new_idx = composer.add_variable(47);
        // idx = add_variables(1,2,5,6,7,10,11,13,16,17,20,22,22,25,26,29,29,32,32,33,35,38,39,39,42,42,43);
        composer.create_sort_constraint_with_edges(idx, 1, 29);
        prover = composer.create_prover();
        verifier = composer.create_verifier();
        proof = prover.construct_proof();

        result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
}
TEST(plookup_composer, range_constraint)
{
    {
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        auto indices = add_variables(composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 8);
        }
        // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
        composer.create_sort_constraint(indices);
        composer.process_range_lists();
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        auto indices = add_variables(composer, { 1, 2, 3, 4, 5, 6, 8, 25 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 8);
        }
        composer.create_sort_constraint(indices);
        composer.process_range_lists();
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    {
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        auto indices =
            add_variables(composer, { 1, 2, 3, 4, 5, 6, 10, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 19, 51 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 128);
        }
        composer.create_dummy_constraints(indices);
        composer.process_range_lists();
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        auto indices =
            add_variables(composer, { 1, 2, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_new_range_constraint(indices[i], 79);
        }
        composer.create_dummy_constraints(indices);
        composer.process_range_lists();
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
}
TEST(plookup_composer, range_with_gates)
{

    waffle::PLookupComposer composer = waffle::PLookupComposer();
    auto idx = add_variables(composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
    for (size_t i = 0; i < idx.size(); i++) {
        composer.create_new_range_constraint(idx[i], 8);
    }
    // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
    composer.process_range_lists();

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
TEST(plookup_composer, sort_widget_complex)
{
    {

        waffle::PLookupComposer composer = waffle::PLookupComposer();
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

        waffle::PLookupComposer composer = waffle::PLookupComposer();
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
TEST(plookup_composer, sort_widget_neg)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
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

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, false);
}
TEST(plookup_composer, composed_range_constraint)
{

    {
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        uint256_t a = 1;
        auto b = a << 35;

        auto a_idx = composer.add_variable(fr(b));
        composer.decompose_into_default_range(a_idx, 48);

        composer.process_range_lists();
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::PLookupComposer composer = waffle::PLookupComposer();
        uint256_t a = 1;
        uint256_t b = a << 35;
        auto a_idx = composer.add_variable(fr(b));
        composer.decompose_into_default_range(a_idx, 48);

        composer.create_add_gate({ a_idx, composer.zero_idx, composer.zero_idx,  1, 0, 0,-fr(b) });
        
        composer.process_range_lists();
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
}