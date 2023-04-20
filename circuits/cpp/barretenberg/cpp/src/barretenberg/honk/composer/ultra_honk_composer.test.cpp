#include "ultra_honk_composer.hpp"
#include "barretenberg/common/log.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/honk/sumcheck/relations/relation.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include <cstddef>
#include <cstdint>
#include "barretenberg/honk/proof_system/prover.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_round.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_computation_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_initialization_relation.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"

// TODO(luke): TEMPORARY; for testing only (comparison with Ultra Plonk composers)
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/composer/splitting_tmp/ultra_plonk_composer.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"

#include <gtest/gtest.h>
#include <string>

using namespace proof_system::honk;

namespace test_ultra_honk_composer {

std::vector<uint32_t> add_variables(auto& composer, std::vector<fr> variables)
{
    std::vector<uint32_t> res;
    for (size_t i = 0; i < variables.size(); i++) {
        res.emplace_back(composer.add_variable(variables[i]));
    }
    return res;
}

/**
 * @brief TEMPORARY method for checking consistency of polynomials computed by Ultra Plonk/Honk composers
 *
 * @param honk_prover
 * @param plonk_prover
 */
void verify_consistency(honk::UltraProver& honk_prover, plonk::UltraProver& plonk_prover)
{
    auto& honk_store = honk_prover.key->polynomial_store;
    auto& plonk_store = plonk_prover.key->polynomial_store;

    // Check that all selectors and table polynomials agree (aside from the final element which will differ
    // due to not enforcing non-zero polynomials in Honk).
    for (auto& entry : honk_store) {
        std::string key = entry.first;
        bool is_selector = (key.find("q_") != std::string::npos) || (key.find("table_type") != std::string::npos);
        bool is_table = (key.find("table_value_") != std::string::npos);
        if (plonk_store.contains(key) && (is_selector || is_table)) {
            // check equality for all but final entry
            for (size_t i = 0; i < honk_store.get(key).size() - 1; ++i) {
                ASSERT_EQ(honk_store.get(key)[i], plonk_store.get(key)[i]);
            }
        }
    }

    // Check that sorted witness-table polynomials agree
    for (auto& entry : honk_store) {
        std::string key = entry.first;
        bool is_sorted_table = (key.find("s_") != std::string::npos);
        if (plonk_store.contains(key) && is_sorted_table) {
            ASSERT_EQ(honk_store.get(key), plonk_store.get(key));
        }
    }

    // Check that all wires agree
    // Note: for Honk, wires are owned directly by the prover. For Plonk they are stored in the key.
    for (size_t i = 0; i < 4; ++i) {
        std::string label = "w_" + std::to_string(i + 1) + "_lagrange";
        ASSERT_EQ(honk_prover.wire_polynomials[i], plonk_prover.key->polynomial_store.get(label));
    }
}

/**
 * @brief TEMPORARY (verbose) method for checking consistency of polynomials computed by Ultra Plonk/Honk composers
 *
 * @param honk_prover
 * @param plonk_prover
 */
void check_consistency(honk::UltraProver& honk_prover, plonk::UltraProver& plonk_prover)
{
    auto& honk_store = honk_prover.key->polynomial_store;
    auto& plonk_store = plonk_prover.key->polynomial_store;
    for (auto& entry : honk_store) {
        std::string key = entry.first;
        if (plonk_store.contains(key)) {

            bool polys_equal = (honk_store.get(key) == plonk_store.get(key));
            if (polys_equal) {
                info("Equal: ", key);
            }
            if (!polys_equal) {
                info("UNEQUAL: ", key);
            }
        }
    }

    for (size_t i = 0; i < 4; ++i) {
        std::string label = "w_" + std::to_string(i + 1) + "_lagrange";
        bool wire_equal = (honk_prover.wire_polynomials[i] == plonk_prover.key->polynomial_store.get(label));
        if (wire_equal) {
            info("Wire Equal: ", i);
        }
        if (!wire_equal) {
            info("Wire UNEQUAL: ", i);
        }
    }

    // std::string label = "w_1_lagrange";
    // for (size_t i = 0; i < plonk_store.get(label).size(); ++i) {
    //     auto val_honk = honk_prover.wire_polynomials[0][i];
    //     // auto val_honk = honk_store.get(label)[i];
    //     auto val_plonk = plonk_store.get(label)[i];
    //     if (val_honk != val_plonk) {
    //         info("UNEQUAL index = ", i);
    //         info("honk: ",val_honk);
    //         info("plonk: ", val_plonk);
    //     }
    // }
}

TEST(UltraHonkComposer, create_gates_from_plookup_accumulators)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    barretenberg::fr input_value = fr::random_element();
    {

        const fr input_hi = uint256_t(input_value).slice(126, 256);
        const fr input_lo = uint256_t(input_value).slice(0, 126);
        const auto input_hi_index = honk_composer.add_variable(input_hi);
        const auto input_lo_index = honk_composer.add_variable(input_lo);

        const auto sequence_data_hi =
            plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_HI, input_hi);
        const auto sequence_data_lo =
            plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_LO, input_lo);

        const auto lookup_witnesses_hi = honk_composer.create_gates_from_plookup_accumulators(
            plookup::MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
        const auto lookup_witnesses_lo = honk_composer.create_gates_from_plookup_accumulators(
            plookup::MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);
    }
    {
        const fr input_hi = uint256_t(input_value).slice(126, 256);
        const fr input_lo = uint256_t(input_value).slice(0, 126);
        const auto input_hi_index = plonk_composer.add_variable(input_hi);
        const auto input_lo_index = plonk_composer.add_variable(input_lo);

        const auto sequence_data_hi =
            plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_HI, input_hi);
        const auto sequence_data_lo =
            plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_LO, input_lo);

        const auto lookup_witnesses_hi = plonk_composer.create_gates_from_plookup_accumulators(
            plookup::MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
        const auto lookup_witnesses_lo = plonk_composer.create_gates_from_plookup_accumulators(
            plookup::MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

/**
 * @brief Build UltraHonkComposer
 *
 */
TEST(UltraHonkComposer, test_no_lookup_proof)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    size_t MM = 4;
    for (size_t i = 0; i < MM; ++i) {
        for (size_t j = 0; j < MM; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = honk_composer.add_variable(fr(left));
            uint32_t right_idx = honk_composer.add_variable(fr(right));
            uint32_t result_idx = honk_composer.add_variable(fr(left ^ right));

            uint32_t add_idx =
                honk_composer.add_variable(fr(left) + fr(right) + honk_composer.get_variable(result_idx));
            honk_composer.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    for (size_t i = 0; i < MM; ++i) {
        for (size_t j = 0; j < MM; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = plonk_composer.add_variable(fr(left));
            uint32_t right_idx = plonk_composer.add_variable(fr(right));
            uint32_t result_idx = plonk_composer.add_variable(fr(left ^ right));

            uint32_t add_idx =
                plonk_composer.add_variable(fr(left) + fr(right) + plonk_composer.get_variable(result_idx));
            plonk_composer.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, test_elliptic_gate)
{
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;

    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        affine_element p1 = crypto::generators::get_generator_data({ 0, 0 }).generator;
        affine_element p2 = crypto::generators::get_generator_data({ 0, 1 }).generator;
        affine_element p3(element(p1) + element(p2));

        uint32_t x1 = honk_composer.add_variable(p1.x);
        uint32_t y1 = honk_composer.add_variable(p1.y);
        uint32_t x2 = honk_composer.add_variable(p2.x);
        uint32_t y2 = honk_composer.add_variable(p2.y);
        uint32_t x3 = honk_composer.add_variable(p3.x);
        uint32_t y3 = honk_composer.add_variable(p3.y);

        ecc_add_gate gate{ x1, y1, x2, y2, x3, y3, 1, 1 };
        honk_composer.create_ecc_add_gate(gate);

        grumpkin::fq beta = grumpkin::fq::cube_root_of_unity();
        affine_element p2_endo = p2;
        p2_endo.x *= beta;
        p3 = affine_element(element(p1) + element(p2_endo));
        x3 = honk_composer.add_variable(p3.x);
        y3 = honk_composer.add_variable(p3.y);
        gate = ecc_add_gate{ x1, y1, x2, y2, x3, y3, beta, 1 };
        honk_composer.create_ecc_add_gate(gate);

        p2_endo.x *= beta;
        p3 = affine_element(element(p1) - element(p2_endo));
        x3 = honk_composer.add_variable(p3.x);
        y3 = honk_composer.add_variable(p3.y);
        gate = ecc_add_gate{ x1, y1, x2, y2, x3, y3, beta.sqr(), -1 };
        honk_composer.create_ecc_add_gate(gate);
    }
    {
        affine_element p1 = crypto::generators::get_generator_data({ 0, 0 }).generator;
        affine_element p2 = crypto::generators::get_generator_data({ 0, 1 }).generator;
        affine_element p3(element(p1) + element(p2));

        uint32_t x1 = plonk_composer.add_variable(p1.x);
        uint32_t y1 = plonk_composer.add_variable(p1.y);
        uint32_t x2 = plonk_composer.add_variable(p2.x);
        uint32_t y2 = plonk_composer.add_variable(p2.y);
        uint32_t x3 = plonk_composer.add_variable(p3.x);
        uint32_t y3 = plonk_composer.add_variable(p3.y);

        ecc_add_gate gate{ x1, y1, x2, y2, x3, y3, 1, 1 };
        plonk_composer.create_ecc_add_gate(gate);

        grumpkin::fq beta = grumpkin::fq::cube_root_of_unity();
        affine_element p2_endo = p2;
        p2_endo.x *= beta;
        p3 = affine_element(element(p1) + element(p2_endo));
        x3 = plonk_composer.add_variable(p3.x);
        y3 = plonk_composer.add_variable(p3.y);
        gate = ecc_add_gate{ x1, y1, x2, y2, x3, y3, beta, 1 };
        plonk_composer.create_ecc_add_gate(gate);

        p2_endo.x *= beta;
        p3 = affine_element(element(p1) - element(p2_endo));
        x3 = plonk_composer.add_variable(p3.x);
        y3 = plonk_composer.add_variable(p3.y);
        gate = ecc_add_gate{ x1, y1, x2, y2, x3, y3, beta.sqr(), -1 };
        plonk_composer.create_ecc_add_gate(gate);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, non_trivial_tag_permutation)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    fr a = fr::random_element();
    {
        fr b = -a;

        auto a_idx = honk_composer.add_variable(a);
        auto b_idx = honk_composer.add_variable(b);
        auto c_idx = honk_composer.add_variable(b);
        auto d_idx = honk_composer.add_variable(a);

        honk_composer.create_add_gate(
            { a_idx, b_idx, honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), fr::zero() });
        honk_composer.create_add_gate(
            { c_idx, d_idx, honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), fr::zero() });

        honk_composer.create_tag(1, 2);
        honk_composer.create_tag(2, 1);

        honk_composer.assign_tag(a_idx, 1);
        honk_composer.assign_tag(b_idx, 1);
        honk_composer.assign_tag(c_idx, 2);
        honk_composer.assign_tag(d_idx, 2);
    }
    {
        fr b = -a;

        auto a_idx = plonk_composer.add_variable(a);
        auto b_idx = plonk_composer.add_variable(b);
        auto c_idx = plonk_composer.add_variable(b);
        auto d_idx = plonk_composer.add_variable(a);

        plonk_composer.create_add_gate(
            { a_idx, b_idx, plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });
        plonk_composer.create_add_gate(
            { c_idx, d_idx, plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });

        plonk_composer.create_tag(1, 2);
        plonk_composer.create_tag(2, 1);

        plonk_composer.assign_tag(a_idx, 1);
        plonk_composer.assign_tag(b_idx, 1);
        plonk_composer.assign_tag(c_idx, 2);
        plonk_composer.assign_tag(d_idx, 2);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, non_trivial_tag_permutation_and_cycles)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    fr a = fr::random_element();
    {
        fr c = -a;

        auto a_idx = honk_composer.add_variable(a);
        auto b_idx = honk_composer.add_variable(a);
        honk_composer.assert_equal(a_idx, b_idx);
        auto c_idx = honk_composer.add_variable(c);
        auto d_idx = honk_composer.add_variable(c);
        honk_composer.assert_equal(c_idx, d_idx);
        auto e_idx = honk_composer.add_variable(a);
        auto f_idx = honk_composer.add_variable(a);
        honk_composer.assert_equal(e_idx, f_idx);
        auto g_idx = honk_composer.add_variable(c);
        auto h_idx = honk_composer.add_variable(c);
        honk_composer.assert_equal(g_idx, h_idx);

        honk_composer.create_tag(1, 2);
        honk_composer.create_tag(2, 1);

        honk_composer.assign_tag(a_idx, 1);
        honk_composer.assign_tag(c_idx, 1);
        honk_composer.assign_tag(e_idx, 2);
        honk_composer.assign_tag(g_idx, 2);

        honk_composer.create_add_gate(
            { b_idx, a_idx, honk_composer.get_zero_idx(), fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
        honk_composer.create_add_gate(
            { c_idx, g_idx, honk_composer.get_zero_idx(), fr::one(), -fr::one(), fr::zero(), fr::zero() });
        honk_composer.create_add_gate(
            { e_idx, f_idx, honk_composer.get_zero_idx(), fr::one(), -fr::one(), fr::zero(), fr::zero() });
    }
    {
        fr c = -a;

        auto a_idx = plonk_composer.add_variable(a);
        auto b_idx = plonk_composer.add_variable(a);
        plonk_composer.assert_equal(a_idx, b_idx);
        auto c_idx = plonk_composer.add_variable(c);
        auto d_idx = plonk_composer.add_variable(c);
        plonk_composer.assert_equal(c_idx, d_idx);
        auto e_idx = plonk_composer.add_variable(a);
        auto f_idx = plonk_composer.add_variable(a);
        plonk_composer.assert_equal(e_idx, f_idx);
        auto g_idx = plonk_composer.add_variable(c);
        auto h_idx = plonk_composer.add_variable(c);
        plonk_composer.assert_equal(g_idx, h_idx);

        plonk_composer.create_tag(1, 2);
        plonk_composer.create_tag(2, 1);

        plonk_composer.assign_tag(a_idx, 1);
        plonk_composer.assign_tag(c_idx, 1);
        plonk_composer.assign_tag(e_idx, 2);
        plonk_composer.assign_tag(g_idx, 2);

        plonk_composer.create_add_gate(
            { b_idx, a_idx, plonk_composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
        plonk_composer.create_add_gate(
            { c_idx, g_idx, plonk_composer.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });
        plonk_composer.create_add_gate(
            { e_idx, f_idx, plonk_composer.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, bad_tag_permutation)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    fr a = fr::random_element();
    {
        fr b = -a;

        auto a_idx = honk_composer.add_variable(a);
        auto b_idx = honk_composer.add_variable(b);
        auto c_idx = honk_composer.add_variable(b);
        auto d_idx = honk_composer.add_variable(a + 1);

        honk_composer.create_add_gate({ a_idx, b_idx, honk_composer.get_zero_idx(), 1, 1, 0, 0 });
        honk_composer.create_add_gate({ c_idx, d_idx, honk_composer.get_zero_idx(), 1, 1, 0, -1 });

        honk_composer.create_tag(1, 2);
        honk_composer.create_tag(2, 1);

        honk_composer.assign_tag(a_idx, 1);
        honk_composer.assign_tag(b_idx, 1);
        honk_composer.assign_tag(c_idx, 2);
        honk_composer.assign_tag(d_idx, 2);
    }
    {
        fr b = -a;

        auto a_idx = plonk_composer.add_variable(a);
        auto b_idx = plonk_composer.add_variable(b);
        auto c_idx = plonk_composer.add_variable(b);
        auto d_idx = plonk_composer.add_variable(a + 1);

        plonk_composer.create_add_gate({ a_idx, b_idx, plonk_composer.zero_idx, 1, 1, 0, 0 });
        plonk_composer.create_add_gate({ c_idx, d_idx, plonk_composer.zero_idx, 1, 1, 0, -1 });

        plonk_composer.create_tag(1, 2);
        plonk_composer.create_tag(2, 1);

        plonk_composer.assign_tag(a_idx, 1);
        plonk_composer.assign_tag(b_idx, 1);
        plonk_composer.assign_tag(c_idx, 2);
        plonk_composer.assign_tag(d_idx, 2);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, sort_widget)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        fr a = fr::one();
        fr b = fr(2);
        fr c = fr(3);
        fr d = fr(4);

        auto a_idx = honk_composer.add_variable(a);
        auto b_idx = honk_composer.add_variable(b);
        auto c_idx = honk_composer.add_variable(c);
        auto d_idx = honk_composer.add_variable(d);
        honk_composer.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
    }
    {
        fr a = fr::one();
        fr b = fr(2);
        fr c = fr(3);
        fr d = fr(4);

        auto a_idx = plonk_composer.add_variable(a);
        auto b_idx = plonk_composer.add_variable(b);
        auto c_idx = plonk_composer.add_variable(c);
        auto d_idx = plonk_composer.add_variable(d);
        plonk_composer.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, sort_with_edges_gate)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        auto idx = add_variables(honk_composer, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                                  26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });

        honk_composer.create_sort_constraint_with_edges(idx, 1, 29);
    }
    {
        auto idx = add_variables(plonk_composer, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                                   26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });

        plonk_composer.create_sort_constraint_with_edges(idx, 1, 29);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, range_constraint)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        auto indices =
            add_variables(honk_composer, { 1, 0, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            honk_composer.create_new_range_constraint(indices[i], 79);
        }
        honk_composer.create_dummy_constraints(indices);
    }
    {
        auto indices =
            add_variables(plonk_composer, { 1, 0, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            plonk_composer.create_new_range_constraint(indices[i], 79);
        }
        plonk_composer.create_dummy_constraints(indices);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, range_with_gates)
{

    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        auto idx = add_variables(honk_composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < idx.size(); i++) {
            honk_composer.create_new_range_constraint(idx[i], 8);
        }

        honk_composer.create_add_gate(
            { idx[0], idx[1], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -3 });
        honk_composer.create_add_gate(
            { idx[2], idx[3], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -7 });
        honk_composer.create_add_gate(
            { idx[4], idx[5], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -11 });
        honk_composer.create_add_gate(
            { idx[6], idx[7], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -15 });
    }
    {
        auto idx = add_variables(plonk_composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < idx.size(); i++) {
            plonk_composer.create_new_range_constraint(idx[i], 8);
        }

        plonk_composer.create_add_gate(
            { idx[0], idx[1], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -3 });
        plonk_composer.create_add_gate(
            { idx[2], idx[3], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -7 });
        plonk_composer.create_add_gate(
            { idx[4], idx[5], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -11 });
        plonk_composer.create_add_gate(
            { idx[6], idx[7], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -15 });
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, range_with_gates_where_range_is_not_a_power_of_two)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        auto idx = add_variables(honk_composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < idx.size(); i++) {
            honk_composer.create_new_range_constraint(idx[i], 12);
        }

        honk_composer.create_add_gate(
            { idx[0], idx[1], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -3 });
        honk_composer.create_add_gate(
            { idx[2], idx[3], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -7 });
        honk_composer.create_add_gate(
            { idx[4], idx[5], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -11 });
        honk_composer.create_add_gate(
            { idx[6], idx[7], honk_composer.get_zero_idx(), fr::one(), fr::one(), fr::zero(), -15 });
    }
    {
        auto idx = add_variables(plonk_composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < idx.size(); i++) {
            plonk_composer.create_new_range_constraint(idx[i], 12);
        }

        plonk_composer.create_add_gate(
            { idx[0], idx[1], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -3 });
        plonk_composer.create_add_gate(
            { idx[2], idx[3], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -7 });
        plonk_composer.create_add_gate(
            { idx[4], idx[5], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -11 });
        plonk_composer.create_add_gate(
            { idx[6], idx[7], plonk_composer.zero_idx, fr::one(), fr::one(), fr::zero(), -15 });
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, sort_widget_complex)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    {
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 16, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(honk_composer.add_variable(a[i]));
        honk_composer.create_sort_constraint(ind);
    }
    {
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 16, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(plonk_composer.add_variable(a[i]));
        plonk_composer.create_sort_constraint(ind);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, composed_range_constraint)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    auto c = fr::random_element();
    {
        auto d = uint256_t(c).slice(0, 133);
        auto e = fr(d);
        auto a_idx = honk_composer.add_variable(fr(e));
        honk_composer.create_add_gate(
            { a_idx, honk_composer.get_zero_idx(), honk_composer.get_zero_idx(), 1, 0, 0, -fr(e) });
        honk_composer.decompose_into_default_range(a_idx, 134);
    }
    {
        auto d = uint256_t(c).slice(0, 133);
        auto e = fr(d);
        auto a_idx = plonk_composer.add_variable(fr(e));
        plonk_composer.create_add_gate({ a_idx, plonk_composer.zero_idx, plonk_composer.zero_idx, 1, 0, 0, -fr(e) });
        plonk_composer.decompose_into_default_range(a_idx, 134);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, non_native_field_multiplication)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    fq a = fq::random_element();
    fq b = fq::random_element();
    {
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
            limb_indices[0] = honk_composer.add_variable(limbs[0]);
            limb_indices[1] = honk_composer.add_variable(limbs[1]);
            limb_indices[2] = honk_composer.add_variable(limbs[2]);
            limb_indices[3] = honk_composer.add_variable(limbs[3]);
            limb_indices[4] = honk_composer.add_variable(limbs[4]);
            return limb_indices;
        };
        const uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (68 * 4);
        auto modulus_limbs = split_into_limbs(BINARY_BASIS_MODULUS - uint512_t(modulus));

        const auto a_indices = get_limb_witness_indices(split_into_limbs(uint256_t(a)));
        const auto b_indices = get_limb_witness_indices(split_into_limbs(uint256_t(b)));
        const auto q_indices = get_limb_witness_indices(split_into_limbs(uint256_t(q)));
        const auto r_indices = get_limb_witness_indices(split_into_limbs(uint256_t(r)));

        proof_system::non_native_field_witnesses inputs{
            a_indices, b_indices, q_indices, r_indices, modulus_limbs, fr(uint256_t(modulus)),
        };
        const auto [lo_1_idx, hi_1_idx] = honk_composer.evaluate_non_native_field_multiplication(inputs);
        honk_composer.range_constrain_two_limbs(lo_1_idx, hi_1_idx, 70, 70);
    }
    {
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
            limb_indices[0] = plonk_composer.add_variable(limbs[0]);
            limb_indices[1] = plonk_composer.add_variable(limbs[1]);
            limb_indices[2] = plonk_composer.add_variable(limbs[2]);
            limb_indices[3] = plonk_composer.add_variable(limbs[3]);
            limb_indices[4] = plonk_composer.add_variable(limbs[4]);
            return limb_indices;
        };
        const uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (68 * 4);
        auto modulus_limbs = split_into_limbs(BINARY_BASIS_MODULUS - uint512_t(modulus));

        const auto a_indices = get_limb_witness_indices(split_into_limbs(uint256_t(a)));
        const auto b_indices = get_limb_witness_indices(split_into_limbs(uint256_t(b)));
        const auto q_indices = get_limb_witness_indices(split_into_limbs(uint256_t(q)));
        const auto r_indices = get_limb_witness_indices(split_into_limbs(uint256_t(r)));

        proof_system::plonk::UltraComposer::non_native_field_witnesses inputs{
            a_indices, b_indices, q_indices, r_indices, modulus_limbs, fr(uint256_t(modulus)),
        };
        const auto [lo_1_idx, hi_1_idx] = plonk_composer.evaluate_non_native_field_multiplication(inputs);
        plonk_composer.range_constrain_two_limbs(lo_1_idx, hi_1_idx, 70, 70);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, rom)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    auto a = fr::random_element();
    auto b = fr::random_element();
    auto c = fr::random_element();
    auto d = fr::random_element();
    auto e = fr::random_element();
    auto f = fr::random_element();
    auto g = fr::random_element();
    auto h = fr::random_element();
    {
        uint32_t rom_values[8]{
            honk_composer.add_variable(a), honk_composer.add_variable(b), honk_composer.add_variable(c),
            honk_composer.add_variable(d), honk_composer.add_variable(e), honk_composer.add_variable(f),
            honk_composer.add_variable(g), honk_composer.add_variable(h),
        };

        size_t rom_id = honk_composer.create_ROM_array(8);

        for (size_t i = 0; i < 8; ++i) {
            honk_composer.set_ROM_element(rom_id, i, rom_values[i]);
        }

        uint32_t a_idx = honk_composer.read_ROM_array(rom_id, honk_composer.add_variable(5));
        EXPECT_EQ(a_idx != rom_values[5], true);
        uint32_t b_idx = honk_composer.read_ROM_array(rom_id, honk_composer.add_variable(4));
        uint32_t c_idx = honk_composer.read_ROM_array(rom_id, honk_composer.add_variable(1));

        const auto d_value =
            honk_composer.get_variable(a_idx) + honk_composer.get_variable(b_idx) + honk_composer.get_variable(c_idx);
        uint32_t d_idx = honk_composer.add_variable(d_value);

        honk_composer.create_big_add_gate({
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
    }
    {
        uint32_t rom_values[8]{
            plonk_composer.add_variable(a), plonk_composer.add_variable(b), plonk_composer.add_variable(c),
            plonk_composer.add_variable(d), plonk_composer.add_variable(e), plonk_composer.add_variable(f),
            plonk_composer.add_variable(g), plonk_composer.add_variable(h),
        };

        size_t rom_id = plonk_composer.create_ROM_array(8);

        for (size_t i = 0; i < 8; ++i) {
            plonk_composer.set_ROM_element(rom_id, i, rom_values[i]);
        }

        uint32_t a_idx = plonk_composer.read_ROM_array(rom_id, plonk_composer.add_variable(5));
        EXPECT_EQ(a_idx != rom_values[5], true);
        uint32_t b_idx = plonk_composer.read_ROM_array(rom_id, plonk_composer.add_variable(4));
        uint32_t c_idx = plonk_composer.read_ROM_array(rom_id, plonk_composer.add_variable(1));

        const auto d_value = plonk_composer.get_variable(a_idx) + plonk_composer.get_variable(b_idx) +
                             plonk_composer.get_variable(c_idx);
        uint32_t d_idx = plonk_composer.add_variable(d_value);

        plonk_composer.create_big_add_gate({
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
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    check_consistency(honk_prover, plonk_prover);
    verify_consistency(honk_prover, plonk_prover);
}

TEST(UltraHonkComposer, ram)
{
    auto honk_composer = UltraHonkComposer();
    auto plonk_composer = proof_system::plonk::UltraComposer();

    auto a = fr::random_element();
    auto b = fr::random_element();
    auto c = fr::random_element();
    auto d = fr::random_element();
    auto e = fr::random_element();
    auto f = fr::random_element();
    auto g = fr::random_element();
    auto h = fr::random_element();
    {
        uint32_t ram_values[8]{
            honk_composer.add_variable(a), honk_composer.add_variable(b), honk_composer.add_variable(c),
            honk_composer.add_variable(d), honk_composer.add_variable(e), honk_composer.add_variable(f),
            honk_composer.add_variable(g), honk_composer.add_variable(h),
        };

        size_t ram_id = honk_composer.create_RAM_array(8);

        for (size_t i = 0; i < 8; ++i) {
            honk_composer.init_RAM_element(ram_id, i, ram_values[i]);
        }

        uint32_t a_idx = honk_composer.read_RAM_array(ram_id, honk_composer.add_variable(5));
        EXPECT_EQ(a_idx != ram_values[5], true);

        uint32_t b_idx = honk_composer.read_RAM_array(ram_id, honk_composer.add_variable(4));
        uint32_t c_idx = honk_composer.read_RAM_array(ram_id, honk_composer.add_variable(1));

        honk_composer.write_RAM_array(ram_id, honk_composer.add_variable(4), honk_composer.add_variable(500));
        uint32_t d_idx = honk_composer.read_RAM_array(ram_id, honk_composer.add_variable(4));

        EXPECT_EQ(honk_composer.get_variable(d_idx), 500);

        // ensure these vars get used in another arithmetic gate
        const auto e_value = honk_composer.get_variable(a_idx) + honk_composer.get_variable(b_idx) +
                             honk_composer.get_variable(c_idx) + honk_composer.get_variable(d_idx);
        uint32_t e_idx = honk_composer.add_variable(e_value);

        honk_composer.create_big_add_gate(
            {
                a_idx,
                b_idx,
                c_idx,
                d_idx,
                -1,
                -1,
                -1,
                -1,
                0,
            },
            true);
        honk_composer.create_big_add_gate(
            {
                honk_composer.get_zero_idx(),
                honk_composer.get_zero_idx(),
                honk_composer.get_zero_idx(),
                e_idx,
                0,
                0,
                0,
                0,
                0,
            },
            false);
    }
    {
        uint32_t ram_values[8]{
            plonk_composer.add_variable(a), plonk_composer.add_variable(b), plonk_composer.add_variable(c),
            plonk_composer.add_variable(d), plonk_composer.add_variable(e), plonk_composer.add_variable(f),
            plonk_composer.add_variable(g), plonk_composer.add_variable(h),
        };

        size_t ram_id = plonk_composer.create_RAM_array(8);

        for (size_t i = 0; i < 8; ++i) {
            plonk_composer.init_RAM_element(ram_id, i, ram_values[i]);
        }

        uint32_t a_idx = plonk_composer.read_RAM_array(ram_id, plonk_composer.add_variable(5));
        EXPECT_EQ(a_idx != ram_values[5], true);

        uint32_t b_idx = plonk_composer.read_RAM_array(ram_id, plonk_composer.add_variable(4));
        uint32_t c_idx = plonk_composer.read_RAM_array(ram_id, plonk_composer.add_variable(1));

        plonk_composer.write_RAM_array(ram_id, plonk_composer.add_variable(4), plonk_composer.add_variable(500));
        uint32_t d_idx = plonk_composer.read_RAM_array(ram_id, plonk_composer.add_variable(4));

        EXPECT_EQ(plonk_composer.get_variable(d_idx), 500);

        // ensure these vars get used in another arithmetic gate
        const auto e_value = plonk_composer.get_variable(a_idx) + plonk_composer.get_variable(b_idx) +
                             plonk_composer.get_variable(c_idx) + plonk_composer.get_variable(d_idx);
        uint32_t e_idx = plonk_composer.add_variable(e_value);

        plonk_composer.create_big_add_gate(
            {
                a_idx,
                b_idx,
                c_idx,
                d_idx,
                -1,
                -1,
                -1,
                -1,
                0,
            },
            true);
        plonk_composer.create_big_add_gate(
            {
                plonk_composer.zero_idx,
                plonk_composer.zero_idx,
                plonk_composer.zero_idx,
                e_idx,
                0,
                0,
                0,
                0,
                0,
            },
            false);
    }

    auto honk_prover = honk_composer.create_prover();
    auto plonk_prover = plonk_composer.create_prover();

    verify_consistency(honk_prover, plonk_prover);
}

} // namespace test_ultra_honk_composer
