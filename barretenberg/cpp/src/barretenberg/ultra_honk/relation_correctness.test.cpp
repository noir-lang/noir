#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
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
template <typename Flavor, typename Relation> void check_relation(auto circuit_size, auto& polynomials, auto params)
{
    using AllValues = typename Flavor::AllValues;
    for (size_t i = 0; i < circuit_size; i++) {

        // Extract an array containing all the polynomial evaluations at a given row i
        AllValues evaluations_at_index_i;
        for (auto [eval, poly] : zip_view(evaluations_at_index_i.get_all(), polynomials.get_all())) {
            eval = poly[i];
        }

        // Define the appropriate SumcheckArrayOfValuesOverSubrelations type for this relation and initialize to zero
        using SumcheckArrayOfValuesOverSubrelations = typename Relation::SumcheckArrayOfValuesOverSubrelations;
        SumcheckArrayOfValuesOverSubrelations result;
        for (auto& element : result) {
            element = 0;
        }

        // Evaluate each constraint in the relation and check that each is satisfied
        Relation::accumulate(result, evaluations_at_index_i, params, 1);
        for (auto& element : result) {
            ASSERT_EQ(element, 0);
        }
    }
}

/**
 * @brief Check that a given linearly dependent relation is satisfied for a set of polynomials
 * @details We refer to a relation as linearly dependent if it defines a constraint on the sum across the full execution
 * trace rather than at each individual row. For example, a subrelation of this type arises in the log derivative lookup
 * argument.
 *
 * @tparam relation_idx Index into a tuple of provided relations
 * @tparam Flavor
 */
template <typename Flavor, typename Relation>
void check_linearly_dependent_relation(auto circuit_size, auto& polynomials, auto params)
{
    using AllValues = typename Flavor::AllValues;
    // Define the appropriate SumcheckArrayOfValuesOverSubrelations type for this relation and initialize to zero
    using SumcheckArrayOfValuesOverSubrelations = typename Relation::SumcheckArrayOfValuesOverSubrelations;
    SumcheckArrayOfValuesOverSubrelations result;
    for (auto& element : result) {
        element = 0;
    }

    for (size_t i = 0; i < circuit_size; i++) {

        // Extract an array containing all the polynomial evaluations at a given row i
        AllValues evaluations_at_index_i;
        for (auto [eval, poly] : zip_view(evaluations_at_index_i.get_all(), polynomials.get_all())) {
            eval = poly[i];
        }

        // Evaluate each constraint in the relation and check that each is satisfied
        Relation::accumulate(result, evaluations_at_index_i, params, 1);
    }

    // Result accumulated across entire execution trace should be zero
    for (auto& element : result) {
        ASSERT_EQ(element, 0);
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
    const auto input_hi =
        uint256_t(pedersen_input_value)
            .slice(plookup::fixed_base::table::BITS_PER_LO_SCALAR,
                   plookup::fixed_base::table::BITS_PER_LO_SCALAR + plookup::fixed_base::table::BITS_PER_HI_SCALAR);
    const auto input_lo = uint256_t(pedersen_input_value).slice(0, plookup::fixed_base::table::BITS_PER_LO_SCALAR);
    const auto input_hi_index = circuit_builder.add_variable(input_hi);
    const auto input_lo_index = circuit_builder.add_variable(input_lo);

    const auto sequence_data_hi = plookup::get_lookup_accumulators(plookup::MultiTableId::FIXED_BASE_LEFT_HI, input_hi);
    const auto sequence_data_lo = plookup::get_lookup_accumulators(plookup::MultiTableId::FIXED_BASE_LEFT_LO, input_lo);

    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::FIXED_BASE_LEFT_HI, sequence_data_hi, input_hi_index);
    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::FIXED_BASE_LEFT_LO, sequence_data_lo, input_lo_index);
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
    grumpkin::g1::affine_element p1 = grumpkin::g1::affine_element::random_element();
    grumpkin::g1::affine_element p2 = grumpkin::g1::affine_element::random_element();

    grumpkin::g1::affine_element p3(grumpkin::g1::element(p1) - grumpkin::g1::element(p2));

    uint32_t x1 = circuit_builder.add_variable(p1.x);
    uint32_t y1 = circuit_builder.add_variable(p1.y);
    uint32_t x2 = circuit_builder.add_variable(p2.x);
    uint32_t y2 = circuit_builder.add_variable(p2.y);
    uint32_t x3 = circuit_builder.add_variable(p3.x);
    uint32_t y3 = circuit_builder.add_variable(p3.y);

    circuit_builder.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, -1 });
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
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

/**
 * @brief Test the correctness of the Ultra Honk relations
 *
 * @details Check that the constraints encoded by the relations are satisfied by the polynomials produced by the
 * Ultra Honk Composer for a real circuit.
 *
 * TODO(Kesha): We'll have to update this function once we add zk, since the relation will be incorrect for the first
 * few indices
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

    instance->initialize_prover_polynomials();
    instance->compute_sorted_accumulator_polynomials(eta);
    instance->compute_grand_product_polynomials(beta, gamma);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(proving_key->q_arith);
    ensure_non_zero(proving_key->q_sort);
    ensure_non_zero(proving_key->q_lookup);
    ensure_non_zero(proving_key->q_elliptic);
    ensure_non_zero(proving_key->q_aux);

    // Construct the round for applying sumcheck relations and results for storing computed results
    using Relations = typename Flavor::Relations;

    auto& prover_polynomials = instance->prover_polynomials;
    auto params = instance->relation_parameters;
    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<0, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<1, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<2, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<3, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<4, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<5, Relations>>(circuit_size, prover_polynomials, params);
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

    instance->initialize_prover_polynomials();
    instance->compute_sorted_accumulator_polynomials(eta);
    instance->compute_logderivative_inverse(beta, gamma);
    instance->compute_grand_product_polynomials(beta, gamma);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(proving_key->q_arith);
    ensure_non_zero(proving_key->q_sort);
    ensure_non_zero(proving_key->q_lookup);
    ensure_non_zero(proving_key->q_elliptic);
    ensure_non_zero(proving_key->q_aux);
    ensure_non_zero(proving_key->q_busread);
    ensure_non_zero(proving_key->q_poseidon2_external);
    ensure_non_zero(proving_key->q_poseidon2_internal);

    ensure_non_zero(proving_key->calldata);
    ensure_non_zero(proving_key->calldata_read_counts);
    ensure_non_zero(proving_key->lookup_inverses);

    // Construct the round for applying sumcheck relations and results for storing computed results
    using Relations = typename Flavor::Relations;
    auto& prover_polynomials = instance->prover_polynomials;
    auto params = instance->relation_parameters;

    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<0, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<1, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<2, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<3, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<4, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<5, Relations>>(circuit_size, prover_polynomials, params);
    check_relation<Flavor, std::tuple_element_t<6, Relations>>(circuit_size, prover_polynomials, params);
    check_linearly_dependent_relation<Flavor, std::tuple_element_t<7, Relations>>(
        circuit_size, prover_polynomials, params);
}

/**
 * @brief Test the correctness of GolbinTranslator's Permutation Relation
 *
 */
TEST_F(RelationCorrectnessTests, GoblinTranslatorPermutationRelationCorrectness)
{
    using Flavor = flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = bb::Polynomial<FF>;
    using namespace proof_system::honk::permutation_library;
    auto& engine = numeric::random::get_debug_engine();
    auto circuit_size = Flavor::MINI_CIRCUIT_SIZE * Flavor::CONCATENATION_INDEX;

    // We only need gamma, because permutationr elation only uses gamma
    FF gamma = FF::random_element();

    // Fill relation parameters
    proof_system::RelationParameters<FF> params;
    params.gamma = gamma;

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    for (Polynomial& prover_poly : prover_polynomials.get_all()) {
        prover_poly = Polynomial{ circuit_size };
    }

    // Fill in lagrange polynomials used in the permutation relation
    prover_polynomials.lagrange_first[0] = 1;
    prover_polynomials.lagrange_last[circuit_size - 1] = 1;

    // Put random values in all the non-concatenated constraint polynomials used to range constrain the values
    auto fill_polynomial_with_random_14_bit_values = [&](auto& polynomial) {
        for (size_t i = 0; i < Flavor::MINI_CIRCUIT_SIZE; i++) {
            polynomial[i] = engine.get_random_uint16() & ((1 << Flavor::MICRO_LIMB_BITS) - 1);
        }
    };
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_x_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.p_y_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.z_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.accumulator_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_low_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_3);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_4);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.quotient_high_limbs_range_constraint_tail);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_0);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_1);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_2);
    fill_polynomial_with_random_14_bit_values(prover_polynomials.relation_wide_limbs_range_constraint_3);

    // Compute ordered range constraint polynomials that go in the denominator of the grand product polynomial
    compute_goblin_translator_range_constraint_ordered_polynomials<Flavor>(&prover_polynomials);

    // Compute the fixed numerator (part of verification key)
    compute_extra_range_constraint_numerator<Flavor>(&prover_polynomials);

    // Compute concatenated polynomials (4 polynomials produced from other constraint polynomials by concatenation)
    compute_concatenated_polynomials<Flavor>(&prover_polynomials);

    // Compute the grand product polynomial
    grand_product_library::compute_grand_product<Flavor, proof_system::GoblinTranslatorPermutationRelation<FF>>(
        circuit_size, prover_polynomials, params);
    prover_polynomials.z_perm_shift = prover_polynomials.z_perm.shifted();

    using Relations = typename Flavor::Relations;

    // Check that permutation relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<0, Relations>>(circuit_size, prover_polynomials, params);
}

TEST_F(RelationCorrectnessTests, GoblinTranslatorGenPermSortRelationCorrectness)
{
    using Flavor = flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = bb::Polynomial<FF>;
    auto& engine = numeric::random::get_debug_engine();

    const auto circuit_size = Flavor::FULL_CIRCUIT_SIZE;
    const auto sort_step = Flavor::SORT_STEP;
    const auto max_value = (1 << Flavor::MICRO_LIMB_BITS) - 1;

    // No relation parameters are used in this relation
    proof_system::RelationParameters<FF> params;

    ProverPolynomials prover_polynomials;
    // Allocate polynomials
    for (Polynomial& polynomial : prover_polynomials.get_all()) {
        polynomial = Polynomial{ circuit_size };
    }

    // Construct lagrange polynomials that are needed for Goblin Translator's GenPermSort Relation
    prover_polynomials.lagrange_first[0] = 1;
    prover_polynomials.lagrange_last[circuit_size - 1] = 1;

    // Create a vector and fill with necessary steps for the GenPermSort relation
    auto sorted_elements_count = (max_value / sort_step) + 1;
    std::vector<uint64_t> vector_for_sorting(circuit_size);
    for (size_t i = 0; i < sorted_elements_count - 1; i++) {
        vector_for_sorting[i] = i * sort_step;
    }
    vector_for_sorting[sorted_elements_count - 1] = max_value;

    // Add random values to fill the leftover space
    for (size_t i = sorted_elements_count; i < circuit_size; i++) {
        vector_for_sorting[i] = engine.get_random_uint16() & ((1 << Flavor::MICRO_LIMB_BITS) - 1);
    }

    // Get ordered polynomials
    auto polynomial_pointers = std::vector{ &prover_polynomials.ordered_range_constraints_0,
                                            &prover_polynomials.ordered_range_constraints_1,
                                            &prover_polynomials.ordered_range_constraints_2,
                                            &prover_polynomials.ordered_range_constraints_3,
                                            &prover_polynomials.ordered_range_constraints_4 };

    // Sort the vector
    std::sort(vector_for_sorting.begin(), vector_for_sorting.end());

    // Copy values, transforming them into Finite Field elements
    std::transform(vector_for_sorting.cbegin(),
                   vector_for_sorting.cend(),
                   prover_polynomials.ordered_range_constraints_0.begin(),
                   [](uint64_t in) { return FF(in); });

    // Copy the same polynomial into the 4 other ordered polynomials (they are not the same in an actual proof, but we
    // only need to check the correctness of the relation and it acts independently on each polynomial)
    parallel_for(4, [&](size_t i) {
        std::copy(prover_polynomials.ordered_range_constraints_0.begin(),
                  prover_polynomials.ordered_range_constraints_0.end(),
                  polynomial_pointers[i + 1]->begin());
    });

    // Get shifted polynomials
    prover_polynomials.ordered_range_constraints_0_shift = prover_polynomials.ordered_range_constraints_0.shifted();
    prover_polynomials.ordered_range_constraints_1_shift = prover_polynomials.ordered_range_constraints_1.shifted();
    prover_polynomials.ordered_range_constraints_2_shift = prover_polynomials.ordered_range_constraints_2.shifted();
    prover_polynomials.ordered_range_constraints_3_shift = prover_polynomials.ordered_range_constraints_3.shifted();
    prover_polynomials.ordered_range_constraints_4_shift = prover_polynomials.ordered_range_constraints_4.shifted();

    using Relations = typename Flavor::Relations;

    // Check that GenPermSort relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<1, Relations>>(circuit_size, prover_polynomials, params);
}

/**
 * @brief Test the correctness of GoblinTranslator's  extra relations (GoblinTranslatorOpcodeConstraintRelation and
 * GoblinTranslatorAccumulatorTransferRelation)
 *
 */
TEST_F(RelationCorrectnessTests, GoblinTranslatorExtraRelationsCorrectness)
{
    using Flavor = flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using ProverPolynomialIds = typename Flavor::ProverPolynomialIds;
    using Polynomial = bb::Polynomial<FF>;

    auto& engine = numeric::random::get_debug_engine();

    auto circuit_size = Flavor::FULL_CIRCUIT_SIZE;
    auto mini_circuit_size = Flavor::MINI_CIRCUIT_SIZE;

    // We only use accumulated_result from relation parameters in this relation
    proof_system::RelationParameters<FF> params;
    params.accumulated_result = {
        FF::random_element(), FF::random_element(), FF::random_element(), FF::random_element()
    };

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    // We use polynomial ids to make shifting the polynomials easier
    ProverPolynomialIds prover_polynomial_ids;
    auto polynomial_id_get_all = prover_polynomial_ids.get_all();
    std::vector<Polynomial> polynomial_container;
    std::vector<size_t> polynomial_ids;
    for (size_t i = 0; i < polynomial_id_get_all.size(); i++) {
        Polynomial temporary_polynomial(circuit_size);
        // Allocate polynomials
        polynomial_container.push_back(temporary_polynomial);
        // Push sequential ids to polynomial ids
        polynomial_ids.push_back(i);
        polynomial_id_get_all[i] = polynomial_ids[i];
    }
    // Get ids of shifted polynomials and put them in a set
    auto shifted_ids = prover_polynomial_ids.get_shifted();
    std::unordered_set<size_t> shifted_id_set;
    for (auto& id : shifted_ids) {
        shifted_id_set.emplace(id);
    }
    // Assign to non-shifted prover polynomials
    auto polynomial_get_all = prover_polynomials.get_all();
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        if (!shifted_id_set.contains(i)) {
            polynomial_get_all[i] = polynomial_container[i].share();
        }
    }

    // Assign to shifted prover polynomials using ids
    for (size_t i = 0; i < shifted_ids.size(); i++) {
        auto shifted_id = shifted_ids[i];
        auto to_be_shifted_id = prover_polynomial_ids.get_to_be_shifted()[i];
        polynomial_get_all[shifted_id] = polynomial_container[to_be_shifted_id].shifted();
    }

    // Fill in lagrange even polynomial
    for (size_t i = 2; i < mini_circuit_size; i += 2) {
        prover_polynomials.lagrange_even_in_minicircuit[i] = 1;
    }
    constexpr size_t NUMBER_OF_POSSIBLE_OPCODES = 6;
    constexpr std::array<uint64_t, NUMBER_OF_POSSIBLE_OPCODES> possible_opcode_values = { 0, 1, 2, 3, 4, 8 };

    // Assign random opcode values
    for (size_t i = 1; i < mini_circuit_size - 1; i += 2) {
        prover_polynomials.op[i] =
            possible_opcode_values[static_cast<size_t>(engine.get_random_uint8() % NUMBER_OF_POSSIBLE_OPCODES)];
    }

    // Initialize used lagrange polynomials
    prover_polynomials.lagrange_second[1] = 1;
    prover_polynomials.lagrange_second_to_last_in_minicircuit[mini_circuit_size - 2] = 1;

    // Put random values in accumulator binary limbs (values should be preserved across even->next odd shift)
    for (size_t i = 2; i < mini_circuit_size - 2; i += 2) {
        prover_polynomials.accumulators_binary_limbs_0[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_1[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_2[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_3[i] = FF ::random_element();
        prover_polynomials.accumulators_binary_limbs_0[i + 1] = prover_polynomials.accumulators_binary_limbs_0[i];
        prover_polynomials.accumulators_binary_limbs_1[i + 1] = prover_polynomials.accumulators_binary_limbs_1[i];
        prover_polynomials.accumulators_binary_limbs_2[i + 1] = prover_polynomials.accumulators_binary_limbs_2[i];
        prover_polynomials.accumulators_binary_limbs_3[i + 1] = prover_polynomials.accumulators_binary_limbs_3[i];
    }

    // The values of accumulator binary limbs at index 1 should equal the accumulated result from relation parameters
    prover_polynomials.accumulators_binary_limbs_0[1] = params.accumulated_result[0];
    prover_polynomials.accumulators_binary_limbs_1[1] = params.accumulated_result[1];
    prover_polynomials.accumulators_binary_limbs_2[1] = params.accumulated_result[2];
    prover_polynomials.accumulators_binary_limbs_3[1] = params.accumulated_result[3];

    using Relations = typename Flavor::Relations;

    // Check that Opcode Constraint relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<2, Relations>>(circuit_size, prover_polynomials, params);

    // Check that Accumulator Transfer relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<3, Relations>>(circuit_size, prover_polynomials, params);
}
/**
 * @brief Test the correctness of GoblinTranslator's Decomposition Relation
 *
 */
TEST_F(RelationCorrectnessTests, GoblinTranslatorDecompositionRelationCorrectness)
{
    using Flavor = flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using ProverPolynomialIds = typename Flavor::ProverPolynomialIds;
    using Polynomial = bb::Polynomial<FF>;
    auto& engine = numeric::random::get_debug_engine();

    auto circuit_size = Flavor::FULL_CIRCUIT_SIZE;

    // Decomposition relation doesn't use any relation parameters
    proof_system::RelationParameters<FF> params;

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    // We use polynomial ids to make shifting the polynomials easier
    ProverPolynomialIds prover_polynomial_ids;
    std::vector<Polynomial> polynomial_container;
    std::vector<size_t> polynomial_ids;
    auto polynomial_id_get_all = prover_polynomial_ids.get_all();
    auto polynomial_get_all = prover_polynomials.get_all();
    for (size_t i = 0; i < polynomial_id_get_all.size(); i++) {
        Polynomial temporary_polynomial(circuit_size);
        // Allocate polynomials
        polynomial_container.push_back(temporary_polynomial);
        // Push sequential ids to polynomial ids
        polynomial_ids.push_back(i);
        polynomial_id_get_all[i] = polynomial_ids[i];
    }
    // Get ids of shifted polynomials and put them in a set
    auto shifted_ids = prover_polynomial_ids.get_shifted();
    std::unordered_set<size_t> shifted_id_set;
    for (auto& id : shifted_ids) {
        shifted_id_set.emplace(id);
    }
    // Assign spans to non-shifted prover polynomials
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        if (!shifted_id_set.contains(i)) {
            polynomial_get_all[i] = polynomial_container[i].share();
        }
    }

    // Assign shifted spans to shifted prover polynomials using ids
    for (size_t i = 0; i < shifted_ids.size(); i++) {
        auto shifted_id = shifted_ids[i];
        auto to_be_shifted_id = prover_polynomial_ids.get_to_be_shifted()[i];
        polynomial_get_all[shifted_id] = polynomial_container[to_be_shifted_id].shifted();
    }

    // Fill in lagrange odd polynomial (the only non-witness one we are using)
    for (size_t i = 1; i < Flavor::MINI_CIRCUIT_SIZE - 1; i += 2) {
        prover_polynomials.lagrange_odd_in_minicircuit[i] = 1;
    }

    constexpr size_t NUM_LIMB_BITS = Flavor::CircuitBuilder::NUM_LIMB_BITS;
    constexpr size_t HIGH_WIDE_LIMB_WIDTH =
        Flavor::CircuitBuilder::NUM_LIMB_BITS + Flavor::CircuitBuilder::NUM_LAST_LIMB_BITS;
    constexpr size_t LOW_WIDE_LIMB_WIDTH = Flavor::CircuitBuilder::NUM_LIMB_BITS * 2;
    constexpr size_t Z_LIMB_WIDTH = 128;
    constexpr size_t MICRO_LIMB_WIDTH = Flavor::MICRO_LIMB_BITS;
    constexpr size_t SHIFT_12_TO_14 = 4;
    constexpr size_t SHIFT_10_TO_14 = 16;
    constexpr size_t SHIFT_8_TO_14 = 64;
    constexpr size_t SHIFT_4_TO_14 = 1024;

    /**
     * @brief Decompose a standard 68-bit limb of binary into 5 14-bit limbs and the 6th limb that is the same as the
     * 5th but shifted by 2 bits
     *
     */
    auto decompose_standard_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& limb_4, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            limb_4 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 4, MICRO_LIMB_WIDTH * 5);
            shifted_limb = limb_4 * SHIFT_12_TO_14;
        };

    /**
     * @brief Decompose a standard 50-bit top limb into 4 14-bit limbs and the 5th limb that is the same as 5th, but
     * shifted by 6 bits
     *
     */
    auto decompose_standard_top_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            shifted_limb = limb_3 * SHIFT_8_TO_14;
        };

    /**
     * @brief Decompose the 60-bit top limb of z1 or z2 into 5 14-bit limbs and a 6th limb which is equal to the 5th,
     * but shifted by 10 bits.
     *
     */
    auto decompose_standard_top_z_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& limb_4, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            limb_4 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 4, MICRO_LIMB_WIDTH * 5);
            shifted_limb = limb_4 * SHIFT_4_TO_14;
        };

    /**
     * @brief Decompose the 52-bit top limb of quotient into 4 14-bit limbs and the 5th limb that is the same as 5th,
     * but shifted by 4 bits
     *
     */
    auto decompose_top_quotient_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& shifted_limb) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            shifted_limb = limb_3 * SHIFT_10_TO_14;
        };

    /**
     * @brief Decompose relation wide limb into 6 14-bit limbs
     *
     */
    auto decompose_relation_limb =
        [](auto& input, auto& limb_0, auto& limb_1, auto& limb_2, auto& limb_3, auto& limb_4, auto& limb_5) {
            limb_0 = uint256_t(input).slice(0, MICRO_LIMB_WIDTH);
            limb_1 = uint256_t(input).slice(MICRO_LIMB_WIDTH, MICRO_LIMB_WIDTH * 2);
            limb_2 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 2, MICRO_LIMB_WIDTH * 3);
            limb_3 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 3, MICRO_LIMB_WIDTH * 4);
            limb_4 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 4, MICRO_LIMB_WIDTH * 5);
            limb_5 = uint256_t(input).slice(MICRO_LIMB_WIDTH * 5, MICRO_LIMB_WIDTH * 6);
        };

    // Put random values in all the non-concatenated constraint polynomials used to range constrain the values
    for (size_t i = 1; i < Flavor::MINI_CIRCUIT_SIZE - 1; i += 2) {
        // P.x
        prover_polynomials.x_lo_y_hi[i] = FF(engine.get_random_uint256() & ((uint256_t(1) << LOW_WIDE_LIMB_WIDTH) - 1));
        prover_polynomials.x_hi_z_1[i] = FF(engine.get_random_uint256() & ((uint256_t(1) << HIGH_WIDE_LIMB_WIDTH) - 1));

        // P.y
        prover_polynomials.y_lo_z_2[i] = FF(engine.get_random_uint256() & ((uint256_t(1) << LOW_WIDE_LIMB_WIDTH) - 1));
        prover_polynomials.x_lo_y_hi[i + 1] =
            FF(engine.get_random_uint256() & ((uint256_t(1) << HIGH_WIDE_LIMB_WIDTH) - 1));

        // z1 and z2
        prover_polynomials.x_hi_z_1[i + 1] = FF(engine.get_random_uint256() & ((uint256_t(1) << Z_LIMB_WIDTH) - 1));
        prover_polynomials.y_lo_z_2[i + 1] = FF(engine.get_random_uint256() & ((uint256_t(1) << Z_LIMB_WIDTH) - 1));

        // Slice P.x into chunks
        prover_polynomials.p_x_low_limbs[i] = uint256_t(prover_polynomials.x_lo_y_hi[i]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_x_low_limbs[i + 1] =
            uint256_t(prover_polynomials.x_lo_y_hi[i]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        prover_polynomials.p_x_high_limbs[i] = uint256_t(prover_polynomials.x_hi_z_1[i]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_x_high_limbs[i + 1] =
            uint256_t(prover_polynomials.x_hi_z_1[i]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);

        // Slice P.y into chunks
        prover_polynomials.p_y_low_limbs[i] = uint256_t(prover_polynomials.y_lo_z_2[i]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_y_low_limbs[i + 1] =
            uint256_t(prover_polynomials.y_lo_z_2[i]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        prover_polynomials.p_y_high_limbs[i] = uint256_t(prover_polynomials.x_lo_y_hi[i + 1]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.p_y_high_limbs[i + 1] =
            uint256_t(prover_polynomials.x_lo_y_hi[i + 1]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);

        // Slice z1 and z2 into chunks
        prover_polynomials.z_low_limbs[i] = uint256_t(prover_polynomials.x_hi_z_1[i + 1]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.z_low_limbs[i + 1] = uint256_t(prover_polynomials.y_lo_z_2[i + 1]).slice(0, NUM_LIMB_BITS);
        prover_polynomials.z_high_limbs[i] =
            uint256_t(prover_polynomials.x_hi_z_1[i + 1]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);
        prover_polynomials.z_high_limbs[i + 1] =
            uint256_t(prover_polynomials.y_lo_z_2[i + 1]).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS);

        // Slice accumulator
        auto tmp = uint256_t(BF::random_element(&engine));
        prover_polynomials.accumulators_binary_limbs_0[i] = tmp.slice(0, NUM_LIMB_BITS);
        prover_polynomials.accumulators_binary_limbs_1[i] = tmp.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2);
        prover_polynomials.accumulators_binary_limbs_2[i] = tmp.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3);
        prover_polynomials.accumulators_binary_limbs_3[i] = tmp.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4);

        // Slice low limbs of P.x into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_x_low_limbs[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_0[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_1[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_2[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_3[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_4[i],
                                prover_polynomials.p_x_low_limbs_range_constraint_tail[i]);

        decompose_standard_limb(prover_polynomials.p_x_low_limbs[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.p_x_low_limbs_range_constraint_tail[i + 1]);

        // Slice high limbs of P.x into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_x_high_limbs[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_0[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_1[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_2[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_3[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_4[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_tail[i]);

        decompose_standard_top_limb(prover_polynomials.p_x_high_limbs[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.p_x_high_limbs_range_constraint_4[i + 1]);

        // Slice low limbs of P.y into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_y_low_limbs[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_0[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_1[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_2[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_3[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_4[i],
                                prover_polynomials.p_y_low_limbs_range_constraint_tail[i]);

        decompose_standard_limb(prover_polynomials.p_y_low_limbs[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.p_y_low_limbs_range_constraint_tail[i + 1]);

        // Slice high limbs of P.y into range constraint microlimbs
        decompose_standard_limb(prover_polynomials.p_y_high_limbs[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_0[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_1[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_2[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_3[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_4[i],
                                prover_polynomials.p_y_high_limbs_range_constraint_tail[i]);

        decompose_standard_top_limb(prover_polynomials.p_y_high_limbs[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.p_y_high_limbs_range_constraint_4[i + 1]);

        // Slice low limb of of z1 and z2 into range constraints
        decompose_standard_limb(prover_polynomials.z_low_limbs[i],
                                prover_polynomials.z_low_limbs_range_constraint_0[i],
                                prover_polynomials.z_low_limbs_range_constraint_1[i],
                                prover_polynomials.z_low_limbs_range_constraint_2[i],
                                prover_polynomials.z_low_limbs_range_constraint_3[i],
                                prover_polynomials.z_low_limbs_range_constraint_4[i],
                                prover_polynomials.z_low_limbs_range_constraint_tail[i]);

        decompose_standard_limb(prover_polynomials.z_low_limbs[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.z_low_limbs_range_constraint_tail[i + 1]);

        // Slice high limb of of z1 and z2 into range constraints
        decompose_standard_top_z_limb(prover_polynomials.z_high_limbs[i],
                                      prover_polynomials.z_high_limbs_range_constraint_0[i],
                                      prover_polynomials.z_high_limbs_range_constraint_1[i],
                                      prover_polynomials.z_high_limbs_range_constraint_2[i],
                                      prover_polynomials.z_high_limbs_range_constraint_3[i],
                                      prover_polynomials.z_high_limbs_range_constraint_4[i],
                                      prover_polynomials.z_high_limbs_range_constraint_tail[i]);

        decompose_standard_top_z_limb(prover_polynomials.z_high_limbs[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_0[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_1[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_2[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_3[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_4[i + 1],
                                      prover_polynomials.z_high_limbs_range_constraint_tail[i + 1]);

        // Slice accumulator limbs into range constraints
        decompose_standard_limb(prover_polynomials.accumulators_binary_limbs_0[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_0[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_1[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_2[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_3[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_4[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_tail[i]);
        decompose_standard_limb(prover_polynomials.accumulators_binary_limbs_1[i],
                                prover_polynomials.accumulator_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.accumulator_low_limbs_range_constraint_tail[i + 1]);

        decompose_standard_limb(prover_polynomials.accumulators_binary_limbs_2[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_0[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_1[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_2[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_3[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_4[i],
                                prover_polynomials.accumulator_high_limbs_range_constraint_tail[i]);
        decompose_standard_top_limb(prover_polynomials.accumulators_binary_limbs_3[i],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.accumulator_high_limbs_range_constraint_4[i + 1]);

        // Slice quotient limbs into range constraints
        decompose_standard_limb(prover_polynomials.quotient_low_binary_limbs[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_0[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_1[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_2[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_3[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_4[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_tail[i]);
        decompose_standard_limb(prover_polynomials.quotient_low_binary_limbs_shift[i],
                                prover_polynomials.quotient_low_limbs_range_constraint_0[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_1[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_2[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_3[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_4[i + 1],
                                prover_polynomials.quotient_low_limbs_range_constraint_tail[i + 1]);

        decompose_standard_limb(prover_polynomials.quotient_high_binary_limbs[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_0[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_1[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_2[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_3[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_4[i],
                                prover_polynomials.quotient_high_limbs_range_constraint_tail[i]);

        decompose_top_quotient_limb(prover_polynomials.quotient_high_binary_limbs_shift[i],
                                    prover_polynomials.quotient_high_limbs_range_constraint_0[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_1[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_2[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_3[i + 1],
                                    prover_polynomials.quotient_high_limbs_range_constraint_4[i + 1]);

        // Decompose wide relation limbs into range constraints
        decompose_relation_limb(prover_polynomials.relation_wide_limbs[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_0[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_1[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_2[i],
                                prover_polynomials.relation_wide_limbs_range_constraint_3[i],
                                prover_polynomials.p_x_high_limbs_range_constraint_tail[i + 1],
                                prover_polynomials.accumulator_high_limbs_range_constraint_tail[i + 1]);

        decompose_relation_limb(prover_polynomials.relation_wide_limbs[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_0[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_1[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_2[i + 1],
                                prover_polynomials.relation_wide_limbs_range_constraint_3[i + 1],
                                prover_polynomials.p_y_high_limbs_range_constraint_tail[i + 1],
                                prover_polynomials.quotient_high_limbs_range_constraint_tail[i + 1]);
    }

    using Relations = Flavor::Relations;
    // Check that Decomposition relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<4, Relations>>(circuit_size, prover_polynomials, params);
}

/**
 * @brief Test the correctness of GoblinTranslator's  NonNativeField Relation
 *
 */
TEST_F(RelationCorrectnessTests, GoblinTranslatorNonNativeRelationCorrectness)
{
    using Flavor = flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using ProverPolynomialIds = typename Flavor::ProverPolynomialIds;
    using GroupElement = typename Flavor::GroupElement;
    using Polynomial = bb::Polynomial<FF>;

    constexpr size_t NUM_LIMB_BITS = Flavor::NUM_LIMB_BITS;
    constexpr auto circuit_size = Flavor::FULL_CIRCUIT_SIZE;
    constexpr auto mini_circuit_size = Flavor::MINI_CIRCUIT_SIZE;

    auto& engine = numeric::random::get_debug_engine();

    auto op_queue = std::make_shared<proof_system::ECCOpQueue>();

    // Generate random EccOpQueue actions
    for (size_t i = 0; i < ((Flavor::MINI_CIRCUIT_SIZE >> 1) - 1); i++) {
        switch (engine.get_random_uint8() & 3) {
        case 0:
            op_queue->empty_row();
            break;
        case 1:
            op_queue->eq();
            break;
        case 2:
            op_queue->add_accumulate(GroupElement::random_element(&engine));
            break;
        case 3:
            op_queue->mul_accumulate(GroupElement::random_element(&engine), FF::random_element(&engine));
            break;
        }
    }
    const auto batching_challenge_v = BF::random_element(&engine);
    const auto evaluation_input_x = BF::random_element(&engine);

    // Generating all the values is pretty tedious, so just use CircuitBuilder
    auto circuit_builder =
        proof_system::GoblinTranslatorCircuitBuilder(batching_challenge_v, evaluation_input_x, op_queue);

    // The non-native field relation uses limbs of evaluation_input_x and powers of batching_challenge_v as inputs
    proof_system::RelationParameters<FF> params;
    auto v_power = BF::one();
    for (size_t i = 0; i < 4 /*Number of powers of v that we need {1,2,3,4}*/; i++) {
        v_power *= batching_challenge_v;
        auto uint_v_power = uint256_t(v_power);
        params.batching_challenge_v[i] = { uint_v_power.slice(0, NUM_LIMB_BITS),
                                           uint_v_power.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                           uint_v_power.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                           uint_v_power.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                           uint_v_power };
    }
    auto uint_input_x = uint256_t(evaluation_input_x);
    params.evaluation_input_x = { uint_input_x.slice(0, NUM_LIMB_BITS),
                                  uint_input_x.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                  uint_input_x.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                  uint_input_x.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                  uint_input_x };

    // Create storage for polynomials
    ProverPolynomials prover_polynomials;
    // We use polynomial ids to make shifting the polynomials easier
    ProverPolynomialIds prover_polynomial_ids;
    std::vector<Polynomial> polynomial_container;
    std::vector<size_t> polynomial_ids;
    auto polynomial_get_all = prover_polynomials.get_all();
    auto polynomial_id_get_all = prover_polynomial_ids.get_all();
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        Polynomial temporary_polynomial(circuit_size);
        // Allocate polynomials
        polynomial_container.push_back(temporary_polynomial);
        // Push sequential ids to polynomial ids
        polynomial_ids.push_back(i);
        polynomial_id_get_all[i] = polynomial_ids[i];
    }
    // Get ids of shifted polynomials and put them in a set
    auto shifted_ids = prover_polynomial_ids.get_shifted();
    std::unordered_set<size_t> shifted_id_set;
    for (auto& id : shifted_ids) {
        shifted_id_set.emplace(id);
    }
    // Assign to non-shifted prover polynomials
    for (size_t i = 0; i < polynomial_get_all.size(); i++) {
        if (!shifted_id_set.contains(i)) {
            polynomial_get_all[i] = polynomial_container[i].share();
        }
    }

    // Assign to shifted prover polynomials using ids
    for (size_t i = 0; i < shifted_ids.size(); i++) {
        auto shifted_id = shifted_ids[i];
        auto to_be_shifted_id = prover_polynomial_ids.get_to_be_shifted()[i];
        polynomial_get_all[shifted_id] = polynomial_container[to_be_shifted_id].shifted();
    }

    // Copy values of wires used in the non-native field relation from the circuit builder
    for (size_t i = 1; i < circuit_builder.get_num_gates(); i++) {
        prover_polynomials.op[i] = circuit_builder.get_variable(circuit_builder.wires[circuit_builder.OP][i]);
        prover_polynomials.p_x_low_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_X_LOW_LIMBS][i]);
        prover_polynomials.p_x_high_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_X_HIGH_LIMBS][i]);
        prover_polynomials.p_y_low_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_Y_LOW_LIMBS][i]);
        prover_polynomials.p_y_high_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.P_Y_HIGH_LIMBS][i]);
        prover_polynomials.z_low_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.Z_LOW_LIMBS][i]);
        prover_polynomials.z_high_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.Z_HIGH_LIMBS][i]);
        prover_polynomials.accumulators_binary_limbs_0[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_0][i]);
        prover_polynomials.accumulators_binary_limbs_1[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_1][i]);
        prover_polynomials.accumulators_binary_limbs_2[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_2][i]);
        prover_polynomials.accumulators_binary_limbs_3[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.ACCUMULATORS_BINARY_LIMBS_3][i]);
        prover_polynomials.quotient_low_binary_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.QUOTIENT_LOW_BINARY_LIMBS][i]);
        prover_polynomials.quotient_high_binary_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.QUOTIENT_HIGH_BINARY_LIMBS][i]);
        prover_polynomials.relation_wide_limbs[i] =
            circuit_builder.get_variable(circuit_builder.wires[circuit_builder.RELATION_WIDE_LIMBS][i]);
    }

    // Fill in lagrange odd polynomial
    for (size_t i = 1; i < mini_circuit_size - 1; i += 2) {
        prover_polynomials.lagrange_odd_in_minicircuit[i] = 1;
    }

    using Relations = Flavor::Relations;
    // Check that Non-Native Field relation is satisfied across each row of the prover polynomials
    check_relation<Flavor, std::tuple_element_t<5, Relations>>(circuit_size, prover_polynomials, params);
}

} // namespace test_honk_relations
