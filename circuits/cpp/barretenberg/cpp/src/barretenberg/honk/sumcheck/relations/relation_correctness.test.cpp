#include "barretenberg/honk/composer/ultra_honk_composer.hpp"
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/honk/proof_system/prover_library.hpp"
#include "barretenberg/honk/sumcheck/relations/relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation_secondary.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include <cstddef>
#include <cstdint>
#include "barretenberg/honk/proof_system/prover.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_round.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_computation_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_initialization_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_grand_product_relation.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

#include <gtest/gtest.h>
#include <string>
#include <vector>

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
    // Create a composer and a dummy circuit with a few gates
    auto composer = StandardHonkComposer();
    static const size_t num_wires = StandardHonkComposer::num_wires;
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
        honk::compute_public_input_delta<fr>(public_inputs, beta, gamma, prover.key->circuit_size);

    sumcheck::RelationParameters<fr> params{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    constexpr size_t num_polynomials = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;
    // Compute grand product polynomial
    polynomial z_permutation =
        prover_library::compute_permutation_grand_product<num_wires>(prover.key, prover.wire_polynomials, beta, gamma);

    // Create an array of spans to the underlying polynomials to more easily
    // get the transposition.
    // Ex: polynomial_spans[3][i] returns the i-th coefficient of the third polynomial
    // in the list below
    std::array<std::span<const fr>, num_polynomials> evaluations_array;

    using POLYNOMIAL = proof_system::honk::StandardArithmetization::POLYNOMIAL;
    evaluations_array[POLYNOMIAL::W_L] = prover.wire_polynomials[0];
    evaluations_array[POLYNOMIAL::W_R] = prover.wire_polynomials[1];
    evaluations_array[POLYNOMIAL::W_O] = prover.wire_polynomials[2];
    evaluations_array[POLYNOMIAL::Z_PERM] = z_permutation;
    evaluations_array[POLYNOMIAL::Z_PERM_SHIFT] = z_permutation.shifted();
    evaluations_array[POLYNOMIAL::Q_M] = prover.key->polynomial_store.get("q_m_lagrange");
    evaluations_array[POLYNOMIAL::Q_L] = prover.key->polynomial_store.get("q_1_lagrange");
    evaluations_array[POLYNOMIAL::Q_R] = prover.key->polynomial_store.get("q_2_lagrange");
    evaluations_array[POLYNOMIAL::Q_O] = prover.key->polynomial_store.get("q_3_lagrange");
    evaluations_array[POLYNOMIAL::Q_C] = prover.key->polynomial_store.get("q_c_lagrange");
    evaluations_array[POLYNOMIAL::SIGMA_1] = prover.key->polynomial_store.get("sigma_1_lagrange");
    evaluations_array[POLYNOMIAL::SIGMA_2] = prover.key->polynomial_store.get("sigma_2_lagrange");
    evaluations_array[POLYNOMIAL::SIGMA_3] = prover.key->polynomial_store.get("sigma_3_lagrange");
    evaluations_array[POLYNOMIAL::ID_1] = prover.key->polynomial_store.get("id_1_lagrange");
    evaluations_array[POLYNOMIAL::ID_2] = prover.key->polynomial_store.get("id_2_lagrange");
    evaluations_array[POLYNOMIAL::ID_3] = prover.key->polynomial_store.get("id_3_lagrange");
    evaluations_array[POLYNOMIAL::LAGRANGE_FIRST] = prover.key->polynomial_store.get("L_first_lagrange");
    evaluations_array[POLYNOMIAL::LAGRANGE_LAST] = prover.key->polynomial_store.get("L_last_lagrange");

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(honk::sumcheck::ArithmeticRelation<fr>(),
                                honk::sumcheck::GrandProductComputationRelation<fr>(),
                                honk::sumcheck::GrandProductInitializationRelation<fr>());

    fr result = 0;
    for (size_t i = 0; i < prover.key->circuit_size; i++) {
        // Compute an array containing all the evaluations at a given row i
        std::array<fr, num_polynomials> evaluations_at_index_i;
        for (size_t j = 0; j < num_polynomials; ++j) {
            evaluations_at_index_i[j] = evaluations_array[j][i];
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
    // Create a composer and a dummy circuit with a few gates
    auto composer = UltraHonkComposer();

    static const size_t num_wires = 4;

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
        honk::compute_public_input_delta<fr>(public_inputs, beta, gamma, prover.key->circuit_size);
    auto lookup_grand_product_delta =
        honk::compute_lookup_grand_product_delta<fr>(beta, gamma, prover.key->circuit_size);

    sumcheck::RelationParameters<fr> params{
        .eta = eta,
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
        .lookup_grand_product_delta = lookup_grand_product_delta,
    };

    constexpr size_t num_polynomials = proof_system::honk::UltraArithmetization::COUNT;

    // Compute permutation grand product polynomial
    auto z_permutation =
        prover_library::compute_permutation_grand_product<num_wires>(prover.key, prover.wire_polynomials, beta, gamma);

    // Construct local sorted_list_polynomials to pass to compute_sorted_list_accumulator()
    std::vector<polynomial> sorted_list_polynomials;
    for (size_t i = 0; i < 4; ++i) {
        std::string label = "s_" + std::to_string(i + 1) + "_lagrange";
        sorted_list_polynomials.emplace_back(prover.key->polynomial_store.get(label));
    }
    // Compute sorted witness-table accumulator
    auto sorted_list_accumulator =
        prover_library::compute_sorted_list_accumulator(prover.key, sorted_list_polynomials, eta);

    // Compute lookup grand product polynomial
    auto z_lookup = prover_library::compute_lookup_grand_product(
        prover.key, prover.wire_polynomials, sorted_list_accumulator, eta, beta, gamma);

    // Create an array of spans to the underlying polynomials to more easily
    // get the transposition.
    // Ex: polynomial_spans[3][i] returns the i-th coefficient of the third polynomial
    // in the list below
    std::array<std::span<const fr>, num_polynomials> evaluations_array;

    using POLYNOMIAL = proof_system::honk::UltraArithmetization::POLYNOMIAL;
    evaluations_array[POLYNOMIAL::W_L] = prover.wire_polynomials[0];
    evaluations_array[POLYNOMIAL::W_R] = prover.wire_polynomials[1];
    evaluations_array[POLYNOMIAL::W_O] = prover.wire_polynomials[2];
    evaluations_array[POLYNOMIAL::W_4] = prover.wire_polynomials[3];
    evaluations_array[POLYNOMIAL::W_1_SHIFT] = prover.wire_polynomials[0].shifted();
    evaluations_array[POLYNOMIAL::W_2_SHIFT] = prover.wire_polynomials[1].shifted();
    evaluations_array[POLYNOMIAL::W_3_SHIFT] = prover.wire_polynomials[2].shifted();
    evaluations_array[POLYNOMIAL::W_4_SHIFT] = prover.wire_polynomials[3].shifted();

    evaluations_array[POLYNOMIAL::S_1] = prover.key->polynomial_store.get("s_1_lagrange");
    evaluations_array[POLYNOMIAL::S_2] = prover.key->polynomial_store.get("s_2_lagrange");
    evaluations_array[POLYNOMIAL::S_3] = prover.key->polynomial_store.get("s_3_lagrange");
    evaluations_array[POLYNOMIAL::S_4] = prover.key->polynomial_store.get("s_4_lagrange");

    evaluations_array[POLYNOMIAL::S_ACCUM] = sorted_list_accumulator;
    evaluations_array[POLYNOMIAL::S_ACCUM_SHIFT] = sorted_list_accumulator.shifted();

    evaluations_array[POLYNOMIAL::Z_PERM] = z_permutation;
    evaluations_array[POLYNOMIAL::Z_PERM_SHIFT] = z_permutation.shifted();

    evaluations_array[POLYNOMIAL::Z_LOOKUP] = z_lookup;
    evaluations_array[POLYNOMIAL::Z_LOOKUP_SHIFT] = z_lookup.shifted();

    evaluations_array[POLYNOMIAL::Q_M] = prover.key->polynomial_store.get("q_m_lagrange");
    evaluations_array[POLYNOMIAL::Q_L] = prover.key->polynomial_store.get("q_1_lagrange");
    evaluations_array[POLYNOMIAL::Q_R] = prover.key->polynomial_store.get("q_2_lagrange");
    evaluations_array[POLYNOMIAL::Q_O] = prover.key->polynomial_store.get("q_3_lagrange");
    evaluations_array[POLYNOMIAL::Q_4] = prover.key->polynomial_store.get("q_4_lagrange");
    evaluations_array[POLYNOMIAL::Q_C] = prover.key->polynomial_store.get("q_c_lagrange");
    evaluations_array[POLYNOMIAL::QARITH] = prover.key->polynomial_store.get("q_arith_lagrange");
    evaluations_array[POLYNOMIAL::QSORT] = prover.key->polynomial_store.get("q_sort_lagrange");
    evaluations_array[POLYNOMIAL::QELLIPTIC] = prover.key->polynomial_store.get("q_elliptic_lagrange");
    evaluations_array[POLYNOMIAL::QAUX] = prover.key->polynomial_store.get("q_aux_lagrange");
    evaluations_array[POLYNOMIAL::QLOOKUPTYPE] = prover.key->polynomial_store.get("table_type_lagrange");

    evaluations_array[POLYNOMIAL::SIGMA_1] = prover.key->polynomial_store.get("sigma_1_lagrange");
    evaluations_array[POLYNOMIAL::SIGMA_2] = prover.key->polynomial_store.get("sigma_2_lagrange");
    evaluations_array[POLYNOMIAL::SIGMA_3] = prover.key->polynomial_store.get("sigma_3_lagrange");
    evaluations_array[POLYNOMIAL::SIGMA_4] = prover.key->polynomial_store.get("sigma_4_lagrange");

    evaluations_array[POLYNOMIAL::ID_1] = prover.key->polynomial_store.get("id_1_lagrange");
    evaluations_array[POLYNOMIAL::ID_2] = prover.key->polynomial_store.get("id_2_lagrange");
    evaluations_array[POLYNOMIAL::ID_3] = prover.key->polynomial_store.get("id_3_lagrange");
    evaluations_array[POLYNOMIAL::ID_4] = prover.key->polynomial_store.get("id_4_lagrange");

    evaluations_array[POLYNOMIAL::TABLE_1] = prover.key->polynomial_store.get("table_value_1_lagrange");
    evaluations_array[POLYNOMIAL::TABLE_2] = prover.key->polynomial_store.get("table_value_2_lagrange");
    evaluations_array[POLYNOMIAL::TABLE_3] = prover.key->polynomial_store.get("table_value_3_lagrange");
    evaluations_array[POLYNOMIAL::TABLE_4] = prover.key->polynomial_store.get("table_value_4_lagrange");

    evaluations_array[POLYNOMIAL::TABLE_1_SHIFT] = prover.key->polynomial_store.get("table_value_1_lagrange").shifted();
    evaluations_array[POLYNOMIAL::TABLE_2_SHIFT] = prover.key->polynomial_store.get("table_value_2_lagrange").shifted();
    evaluations_array[POLYNOMIAL::TABLE_3_SHIFT] = prover.key->polynomial_store.get("table_value_3_lagrange").shifted();
    evaluations_array[POLYNOMIAL::TABLE_4_SHIFT] = prover.key->polynomial_store.get("table_value_4_lagrange").shifted();

    evaluations_array[POLYNOMIAL::LAGRANGE_FIRST] = prover.key->polynomial_store.get("L_first_lagrange");
    evaluations_array[POLYNOMIAL::LAGRANGE_LAST] = prover.key->polynomial_store.get("L_last_lagrange");

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(honk::sumcheck::UltraArithmeticRelation<fr>(),
                                honk::sumcheck::UltraArithmeticRelationSecondary<fr>(),
                                honk::sumcheck::UltraGrandProductInitializationRelation<fr>(),
                                honk::sumcheck::UltraGrandProductComputationRelation<fr>(),
                                honk::sumcheck::LookupGrandProductComputationRelation<fr>(),
                                honk::sumcheck::LookupGrandProductInitializationRelation<fr>());

    fr result = 0;
    for (size_t i = 0; i < prover.key->circuit_size; i++) {
        // Compute an array containing all the evaluations at a given row i
        std::array<fr, num_polynomials> evaluations_at_index_i;
        for (size_t j = 0; j < num_polynomials; ++j) {
            evaluations_at_index_i[j] = evaluations_array[j][i];
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
