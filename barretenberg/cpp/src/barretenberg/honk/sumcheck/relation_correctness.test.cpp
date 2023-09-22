#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/grand_product_library.hpp"
#include "barretenberg/proof_system/relations/auxiliary_relation.hpp"
#include "barretenberg/proof_system/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/proof_system/relations/elliptic_relation.hpp"
#include "barretenberg/proof_system/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/proof_system/relations/lookup_relation.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"
#include "barretenberg/proof_system/relations/ultra_arithmetic_relation.hpp"
#include <gtest/gtest.h>

using namespace proof_system::honk;

namespace test_honk_relations {

void ensure_non_zero(auto& polynomial)
{
    bool has_non_zero_coefficient = false;
    for (auto& coeff : polynomial) {
        has_non_zero_coefficient |= !coeff.is_zero();
    }
    ASSERT_TRUE(has_non_zero_coefficient);
}

/**
 * @brief Check that a given relation is satified for a set of polynomials
 *
 * @tparam relation_idx Index into a tuple of provided relations
 * @tparam Flavor
 */
template <typename Flavor> void check_relation(auto relation, auto circuit_size, auto polynomials, auto params)
{
    using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;
    for (size_t i = 0; i < circuit_size; i++) {

        // Extract an array containing all the polynomial evaluations at a given row i
        ClaimedEvaluations evaluations_at_index_i;
        size_t poly_idx = 0;
        for (auto& poly : polynomials) {
            evaluations_at_index_i[poly_idx] = poly[i];
            ++poly_idx;
        }

        // Define the appropriate RelationValues type for this relation and initialize to zero
        using RelationValues = typename decltype(relation)::RelationValues;
        RelationValues result;
        for (auto& element : result) {
            element = 0;
        }

        // Evaluate each constraint in the relation and check that each is satisfied
        relation.add_full_relation_value_contribution(result, evaluations_at_index_i, params);
        for (auto& element : result) {
            ASSERT_EQ(element, 0);
        }
    }
}

template <typename Flavor> void create_some_add_gates(auto& circuit_builder)
{
    using FF = typename Flavor::FF;
    auto a = FF::random_element();

    // Add some basic add gates; incorporate a public input for non-trivial PI-delta
    uint32_t a_idx = circuit_builder.add_public_variable(a);
    FF b = FF::random_element();
    FF c = a + b;
    FF d = a + c;
    uint32_t b_idx = circuit_builder.add_variable(b);
    uint32_t c_idx = circuit_builder.add_variable(c);
    uint32_t d_idx = circuit_builder.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        circuit_builder.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });
        circuit_builder.create_add_gate({ d_idx, c_idx, a_idx, 1, -1, -1, 0 });
    }

    // Add an Ultra-style big add gate with use of next row to test q_arith = 2
    FF e = a + b + c + d;
    uint32_t e_idx = circuit_builder.add_variable(e);

    uint32_t zero_idx = circuit_builder.zero_idx;
    circuit_builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true); // use next row
    circuit_builder.create_big_add_gate({ zero_idx, zero_idx, zero_idx, e_idx, 0, 0, 0, 0, 0 }, false);
}

template <typename Flavor> void create_some_lookup_gates(auto& circuit_builder)
{
    using FF = typename Flavor::FF;
    // Add some lookup gates (related to pedersen hashing)
    auto pedersen_input_value = FF::random_element();
    const auto input_hi = uint256_t(pedersen_input_value).slice(126, 256);
    const auto input_lo = uint256_t(pedersen_input_value).slice(0, 126);
    const auto input_hi_index = circuit_builder.add_variable(input_hi);
    const auto input_lo_index = circuit_builder.add_variable(input_lo);

    const auto sequence_data_hi = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_HI, input_hi);
    const auto sequence_data_lo = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_LO, input_lo);

    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);
}

template <typename Flavor> void create_some_genperm_sort_gates(auto& circuit_builder)
{
    // Add a sort gate (simply checks that consecutive inputs have a difference of < 4)
    using FF = typename Flavor::FF;
    auto a_idx = circuit_builder.add_variable(FF(0));
    auto b_idx = circuit_builder.add_variable(FF(1));
    auto c_idx = circuit_builder.add_variable(FF(2));
    auto d_idx = circuit_builder.add_variable(FF(3));
    circuit_builder.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
}

template <typename Flavor> void create_some_RAM_gates(auto& circuit_builder)
{
    using FF = typename Flavor::FF;
    // Add some RAM gates
    uint32_t ram_values[8]{
        circuit_builder.add_variable(FF::random_element()), circuit_builder.add_variable(FF::random_element()),
        circuit_builder.add_variable(FF::random_element()), circuit_builder.add_variable(FF::random_element()),
        circuit_builder.add_variable(FF::random_element()), circuit_builder.add_variable(FF::random_element()),
        circuit_builder.add_variable(FF::random_element()), circuit_builder.add_variable(FF::random_element()),
    };

    size_t ram_id = circuit_builder.create_RAM_array(8);

    for (size_t i = 0; i < 8; ++i) {
        circuit_builder.init_RAM_element(ram_id, i, ram_values[i]);
    }

    auto a_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(5));
    EXPECT_EQ(a_idx != ram_values[5], true);

    auto b_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(4));
    auto c_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(1));

    circuit_builder.write_RAM_array(ram_id, circuit_builder.add_variable(4), circuit_builder.add_variable(500));
    auto d_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(4));

    EXPECT_EQ(circuit_builder.get_variable(d_idx), 500);

    // ensure these vars get used in another arithmetic gate
    const auto e_value = circuit_builder.get_variable(a_idx) + circuit_builder.get_variable(b_idx) +
                         circuit_builder.get_variable(c_idx) + circuit_builder.get_variable(d_idx);
    auto e_idx = circuit_builder.add_variable(e_value);

    circuit_builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true);
    circuit_builder.create_big_add_gate(
        {
            circuit_builder.zero_idx,
            circuit_builder.zero_idx,
            circuit_builder.zero_idx,
            e_idx,
            0,
            0,
            0,
            0,
            0,
        },
        false);
}

template <typename Flavor> void create_some_elliptic_curve_addition_gates(auto& circuit_builder)
{
    // Add an elliptic curve addition gate
    grumpkin::g1::affine_element p1 = crypto::generators::get_generator_data({ 0, 0 }).generator;
    grumpkin::g1::affine_element p2 = crypto::generators::get_generator_data({ 0, 1 }).generator;

    grumpkin::fq beta_scalar = grumpkin::fq::cube_root_of_unity();
    grumpkin::g1::affine_element p2_endo = p2;
    p2_endo.x *= beta_scalar;

    grumpkin::g1::affine_element p3(grumpkin::g1::element(p1) - grumpkin::g1::element(p2_endo));

    uint32_t x1 = circuit_builder.add_variable(p1.x);
    uint32_t y1 = circuit_builder.add_variable(p1.y);
    uint32_t x2 = circuit_builder.add_variable(p2.x);
    uint32_t y2 = circuit_builder.add_variable(p2.y);
    uint32_t x3 = circuit_builder.add_variable(p3.x);
    uint32_t y3 = circuit_builder.add_variable(p3.y);

    circuit_builder.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, beta_scalar, -1 });
}

template <typename Flavor> void create_some_ecc_op_queue_gates(auto& circuit_builder)
{
    using G1 = typename Flavor::Curve::Group;
    using FF = typename Flavor::FF;
    static_assert(proof_system::IsGoblinFlavor<Flavor>);
    const size_t num_ecc_operations = 10; // arbitrary
    for (size_t i = 0; i < num_ecc_operations; ++i) {
        auto point = G1::affine_one * FF::random_element();
        auto scalar = FF::random_element();
        circuit_builder.queue_ecc_mul_accum(point, scalar);
    }
}

class RelationCorrectnessTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }
};

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
// TODO(luke): Add a gate that sets q_arith = 3 to check secondary arithmetic relation
TEST_F(RelationCorrectnessTests, UltraRelationCorrectness)
{
    using Flavor = flavor::Ultra;
    using FF = typename Flavor::FF;

    // Create a composer and then add an assortment of gates designed to ensure that the constraint(s) represented
    // by each relation are non-trivially exercised.
    auto builder = proof_system::UltraCircuitBuilder();

    // Create an assortment of representative gates
    create_some_add_gates<Flavor>(builder);
    create_some_lookup_gates<Flavor>(builder);
    create_some_genperm_sort_gates<Flavor>(builder);
    create_some_elliptic_curve_addition_gates<Flavor>(builder);
    create_some_RAM_gates<Flavor>(builder);

    // Create a prover (it will compute proving key and witness)
    auto composer = UltraComposer();
    auto instance = composer.create_instance(builder);
    auto proving_key = instance->proving_key;
    auto circuit_size = proving_key->circuit_size;

    // Generate eta, beta and gamma
    FF eta = FF::random_element();
    FF beta = FF::random_element();
    FF gamma = FF::random_element();

    instance->initialise_prover_polynomials();
    instance->compute_sorted_accumulator_polynomials(eta);
    instance->compute_grand_product_polynomials(beta, gamma);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(proving_key->q_arith);
    ensure_non_zero(proving_key->q_sort);
    ensure_non_zero(proving_key->q_lookup);
    ensure_non_zero(proving_key->q_elliptic);
    ensure_non_zero(proving_key->q_aux);

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(proof_system::UltraArithmeticRelation<FF>(),
                                proof_system::UltraPermutationRelation<FF>(),
                                proof_system::LookupRelation<FF>(),
                                proof_system::GenPermSortRelation<FF>(),
                                proof_system::EllipticRelation<FF>(),
                                proof_system::AuxiliaryRelation<FF>());

    auto prover_polynomials = instance->prover_polynomials;
    auto params = instance->relation_parameters;
    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<Flavor>(std::get<0>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<1>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<2>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<3>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<4>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<5>(relations), circuit_size, prover_polynomials, params);
}

TEST_F(RelationCorrectnessTests, GoblinUltraRelationCorrectness)
{
    using Flavor = flavor::GoblinUltra;
    using FF = typename Flavor::FF;

    // Create a composer and then add an assortment of gates designed to ensure that the constraint(s) represented
    // by each relation are non-trivially exercised.
    auto builder = proof_system::GoblinUltraCircuitBuilder();

    // Create an assortment of representative gates
    create_some_add_gates<Flavor>(builder);
    create_some_lookup_gates<Flavor>(builder);
    create_some_genperm_sort_gates<Flavor>(builder);
    create_some_elliptic_curve_addition_gates<Flavor>(builder);
    create_some_RAM_gates<Flavor>(builder);
    create_some_ecc_op_queue_gates<Flavor>(builder); // Goblin!

    // Create a prover (it will compute proving key and witness)
    auto composer = GoblinUltraComposer();
    auto instance = composer.create_instance(builder);
    auto proving_key = instance->proving_key;
    auto circuit_size = proving_key->circuit_size;

    // Generate eta, beta and gamma
    FF eta = FF::random_element();
    FF beta = FF::random_element();
    FF gamma = FF::random_element();

    instance->initialise_prover_polynomials();
    instance->compute_sorted_accumulator_polynomials(eta);
    instance->compute_grand_product_polynomials(beta, gamma);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(proving_key->q_arith);
    ensure_non_zero(proving_key->q_sort);
    ensure_non_zero(proving_key->q_lookup);
    ensure_non_zero(proving_key->q_elliptic);
    ensure_non_zero(proving_key->q_aux);

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(proof_system::UltraArithmeticRelation<FF>(),
                                proof_system::UltraPermutationRelation<FF>(),
                                proof_system::LookupRelation<FF>(),
                                proof_system::GenPermSortRelation<FF>(),
                                proof_system::EllipticRelation<FF>(),
                                proof_system::AuxiliaryRelation<FF>(),
                                proof_system::EccOpQueueRelation<FF>());

    auto prover_polynomials = instance->prover_polynomials;
    auto params = instance->relation_parameters;

    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<Flavor>(std::get<0>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<1>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<2>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<3>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<4>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<5>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<6>(relations), circuit_size, prover_polynomials, params);
}

} // namespace test_honk_relations
