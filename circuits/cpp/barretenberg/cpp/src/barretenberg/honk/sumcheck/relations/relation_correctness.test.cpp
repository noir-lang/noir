#include <gtest/gtest.h>

#include "barretenberg/honk/composer/ultra_honk_composer.hpp"
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/honk/proof_system/prover_library.hpp"
#include "barretenberg/honk/sumcheck/relations/relation.hpp"
#include "barretenberg/honk/sumcheck/relations/arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_initialization_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_computation_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation_secondary.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_grand_product_relation.hpp"

using namespace proof_system::honk;

namespace test_honk_relations {

/**
 * @brief Test the correctness of the Standard Honk relations
 *
 * @details Check that the constraints encoded by the relations are satisfied by the polynomials produced by the
 * Standard Honk Composer for a real circuit.
 *
 * TODO(Kesha): We'll have to update this function once we add zk, since the relation will be incorrect for he first few
 * indices
 *
 */
TEST(RelationCorrectness, StandardRelationCorrectness)
{
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using PurportedEvaluations = typename Flavor::PurportedEvaluations;

    // Create a composer and a dummy circuit with a few gates
    auto composer = StandardHonkComposer();
    fr a = fr::one();
    // Using the public variable to check that public_input_delta is computed and added to the relation correctly
    uint32_t a_idx = composer.add_public_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = composer.add_variable(b);
    uint32_t c_idx = composer.add_variable(c);
    uint32_t d_idx = composer.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
        composer.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });
    }
    // Create a prover (it will compute proving key and witness)
    auto prover = composer.create_prover();

    // Generate beta and gamma
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = composer.circuit_constructor.get_public_inputs();
    auto public_input_delta =
        honk::compute_public_input_delta<FF>(public_inputs, beta, gamma, prover.key->circuit_size);

    sumcheck::RelationParameters<FF> params{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    // Compute grand product polynomial
    polynomial z_permutation =
        prover_library::compute_permutation_grand_product<honk::flavor::Standard>(prover.key, beta, gamma);

    // Create an array of spans to the underlying polynomials to more easily
    // get the transposition.
    // Ex: polynomial_spans[3][i] returns the i-th coefficient of the third polynomial
    // in the list below
    ProverPolynomials prover_polynomials;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
    prover_polynomials.z_perm = z_permutation;
    prover_polynomials.z_perm_shift = z_permutation.shifted();
    prover_polynomials.q_m = prover.key->q_m;
    prover_polynomials.q_l = prover.key->q_l;
    prover_polynomials.q_r = prover.key->q_r;
    prover_polynomials.q_o = prover.key->q_o;
    prover_polynomials.q_c = prover.key->q_c;
    prover_polynomials.sigma_1 = prover.key->sigma_1;
    prover_polynomials.sigma_2 = prover.key->sigma_2;
    prover_polynomials.sigma_3 = prover.key->sigma_3;
    prover_polynomials.id_1 = prover.key->id_1;
    prover_polynomials.id_2 = prover.key->id_2;
    prover_polynomials.id_3 = prover.key->id_3;
    prover_polynomials.lagrange_first = prover.key->lagrange_first;
    prover_polynomials.lagrange_last = prover.key->lagrange_last;

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(honk::sumcheck::ArithmeticRelation<FF>(),
                                honk::sumcheck::GrandProductComputationRelation<FF>(),
                                honk::sumcheck::GrandProductInitializationRelation<FF>());

    fr result = 0;
    for (size_t i = 0; i < prover.key->circuit_size; i++) {
        // Compute an array containing all the evaluations at a given row i

        PurportedEvaluations evaluations_at_index_i;
        size_t poly_idx = 0;
        for (auto& polynomial : prover_polynomials) {
            evaluations_at_index_i[poly_idx] = polynomial[i];
            ++poly_idx;
        }

        // For each relation, call the `accumulate_relation_evaluation` over all witness/selector values at the
        // i-th row/vertex of the hypercube.
        // We use ASSERT_EQ instead of EXPECT_EQ so that the tests stops at the first index at which the result is not
        // 0, since result = 0 + C(transposed), which we expect will equal 0.
        std::get<0>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<1>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<2>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);
    }
}

/**
 * @brief Test the correctness of the Ultra Honk relations
 *
 * @details Check that the constraints encoded by the relations are satisfied by the polynomials produced by the
 * Ultra Honk Composer for a real circuit.
 *
 * TODO(Kesha): We'll have to update this function once we add zk, since the relation will be incorrect for he first few
 * indices
 *
 */
// TODO(luke): Ensure all relations are added as they are implemented for Ultra Honk
TEST(RelationCorrectness, UltraRelationCorrectness)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using PurportedEvaluations = typename Flavor::PurportedEvaluations;

    // Create a composer and a dummy circuit with a few gates
    auto composer = UltraHonkComposer();

    barretenberg::fr pedersen_input_value = fr::random_element();
    fr a = fr::one();
    // Using the public variable to check that public_input_delta is computed and added to the relation correctly
    // TODO(luke): add method "add_public_variable" to UH composer
    // uint32_t a_idx = composer.add_public_variable(a);
    uint32_t a_idx = composer.add_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = composer.add_variable(b);
    uint32_t c_idx = composer.add_variable(c);
    uint32_t d_idx = composer.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        composer.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });
        composer.create_add_gate({ d_idx, c_idx, a_idx, 1, -1, -1, 0 });
    }

    // Add a big add gate with use of next row to test q_arith = 2
    fr e = a + b + c + d;
    uint32_t e_idx = composer.add_variable(e);

    uint32_t zero_idx = composer.get_zero_idx();
    composer.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true); // use next row
    composer.create_big_add_gate({ zero_idx, zero_idx, zero_idx, e_idx, 0, 0, 0, 0, 0 }, false);

    // Add some lookup gates (related to pedersen hashing)
    const fr input_hi = uint256_t(pedersen_input_value).slice(126, 256);
    const fr input_lo = uint256_t(pedersen_input_value).slice(0, 126);
    const auto input_hi_index = composer.add_variable(input_hi);
    const auto input_lo_index = composer.add_variable(input_lo);

    const auto sequence_data_hi = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_HI, input_hi);
    const auto sequence_data_lo = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_LO, input_lo);

    composer.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
    composer.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);

    // Create a prover (it will compute proving key and witness)
    auto prover = composer.create_prover();

    // Generate eta, beta and gamma
    fr eta = fr::random_element();
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = composer.circuit_constructor.get_public_inputs();
    auto public_input_delta =
        honk::compute_public_input_delta<FF>(public_inputs, beta, gamma, prover.key->circuit_size);
    auto lookup_grand_product_delta =
        honk::compute_lookup_grand_product_delta<FF>(beta, gamma, prover.key->circuit_size);

    sumcheck::RelationParameters<FF> params{
        .eta = eta,
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
        .lookup_grand_product_delta = lookup_grand_product_delta,
    };

    // Compute grand product polynomial
    auto z_permutation = prover_library::compute_permutation_grand_product<Flavor>(prover.key, beta, gamma);

    // Construct local sorted_list_polynomials to pass to compute_sorted_list_accumulator()
    // TODO(luke): this clunkiness can be cleaned up once we decide where the sorted polynomials will be stored: proving
    // key or prover
    std::vector<polynomial> sorted_list_polynomials;
    auto sorted_polynomials = prover.key->get_sorted_polynomials();
    for (size_t i = 0; i < 4; ++i) {
        sorted_list_polynomials.emplace_back(prover.key->circuit_size);
        for (size_t j = 0; j < prover.key->circuit_size; ++j) {
            sorted_list_polynomials[i][j] = sorted_polynomials[i][j];
        }
    }

    // Compute sorted witness-table accumulator
    auto sorted_list_accumulator =
        prover_library::compute_sorted_list_accumulator<Flavor>(prover.key, sorted_list_polynomials, eta);

    // Compute lookup grand product polynomial
    auto z_lookup =
        prover_library::compute_lookup_grand_product<Flavor>(prover.key, sorted_list_accumulator, eta, beta, gamma);

    // Create an array of spans to the underlying polynomials to more easily
    // get the transposition.
    // Ex: polynomial_spans[3][i] returns the i-th coefficient of the third polynomial
    // in the list below
    ProverPolynomials prover_polynomials;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
    prover_polynomials.w_4 = prover.key->w_4;
    prover_polynomials.w_l_shift = prover.key->w_l.shifted();
    prover_polynomials.w_r_shift = prover.key->w_r.shifted();
    prover_polynomials.w_o_shift = prover.key->w_o.shifted();
    prover_polynomials.w_4_shift = prover.key->w_4.shifted();
    prover_polynomials.sorted_1 = prover.key->sorted_1;
    prover_polynomials.sorted_2 = prover.key->sorted_2;
    prover_polynomials.sorted_3 = prover.key->sorted_3;
    prover_polynomials.sorted_4 = prover.key->sorted_4;
    prover_polynomials.sorted_accum = sorted_list_accumulator;
    prover_polynomials.sorted_accum_shift = sorted_list_accumulator.shifted();
    prover_polynomials.table_1 = prover.key->table_1;
    prover_polynomials.table_2 = prover.key->table_2;
    prover_polynomials.table_3 = prover.key->table_3;
    prover_polynomials.table_4 = prover.key->table_4;
    prover_polynomials.table_1_shift = prover.key->table_1.shifted();
    prover_polynomials.table_2_shift = prover.key->table_2.shifted();
    prover_polynomials.table_3_shift = prover.key->table_3.shifted();
    prover_polynomials.table_4_shift = prover.key->table_4.shifted();
    prover_polynomials.z_perm = z_permutation;
    prover_polynomials.z_perm_shift = z_permutation.shifted();
    prover_polynomials.z_lookup = z_lookup;
    prover_polynomials.z_lookup_shift = z_lookup.shifted();
    prover_polynomials.q_m = prover.key->q_m;
    prover_polynomials.q_l = prover.key->q_l;
    prover_polynomials.q_r = prover.key->q_r;
    prover_polynomials.q_o = prover.key->q_o;
    prover_polynomials.q_c = prover.key->q_c;
    prover_polynomials.q_4 = prover.key->q_4;
    prover_polynomials.q_arith = prover.key->q_arith;
    prover_polynomials.q_sort = prover.key->q_sort;
    prover_polynomials.q_elliptic = prover.key->q_elliptic;
    prover_polynomials.q_aux = prover.key->q_aux;
    prover_polynomials.q_lookup = prover.key->q_lookup;
    prover_polynomials.sigma_1 = prover.key->sigma_1;
    prover_polynomials.sigma_2 = prover.key->sigma_2;
    prover_polynomials.sigma_3 = prover.key->sigma_3;
    prover_polynomials.sigma_4 = prover.key->sigma_4;
    prover_polynomials.id_1 = prover.key->id_1;
    prover_polynomials.id_2 = prover.key->id_2;
    prover_polynomials.id_3 = prover.key->id_3;
    prover_polynomials.id_4 = prover.key->id_4;
    prover_polynomials.lagrange_first = prover.key->lagrange_first;
    prover_polynomials.lagrange_last = prover.key->lagrange_last;

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(honk::sumcheck::UltraArithmeticRelation<FF>(),
                                honk::sumcheck::UltraArithmeticRelationSecondary<FF>(),
                                honk::sumcheck::UltraGrandProductInitializationRelation<FF>(),
                                honk::sumcheck::UltraGrandProductComputationRelation<FF>(),
                                honk::sumcheck::LookupGrandProductComputationRelation<FF>(),
                                honk::sumcheck::LookupGrandProductInitializationRelation<FF>());

    fr result = 0;
    for (size_t i = 0; i < prover.key->circuit_size; i++) {
        // Compute an array containing all the evaluations at a given row i
        PurportedEvaluations evaluations_at_index_i;
        size_t poly_idx = 0;
        for (auto& polynomial : prover_polynomials) {
            evaluations_at_index_i[poly_idx] = polynomial[i];
            ++poly_idx;
        }

        // For each relation, call the `accumulate_relation_evaluation` over all witness/selector values at the
        // i-th row/vertex of the hypercube. We use ASSERT_EQ instead of EXPECT_EQ so that the tests stops at
        // the first index at which the result is not 0, since result = 0 + C(transposed), which we expect will
        // equal 0.
        std::get<0>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<1>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<2>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<3>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<4>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);

        std::get<5>(relations).add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        ASSERT_EQ(result, 0);
    }
}

} // namespace test_honk_relations
