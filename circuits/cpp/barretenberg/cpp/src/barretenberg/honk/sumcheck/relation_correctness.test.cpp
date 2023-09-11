#include "barretenberg/honk/composer/standard_composer.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/grand_product_library.hpp"
#include "barretenberg/honk/proof_system/prover_library.hpp"
#include "barretenberg/proof_system/relations/arithmetic_relation.hpp"
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
    fr a = fr::random_element();

    // Add some basic add gates; incorporate a public input for non-trivial PI-delta
    uint32_t a_idx = circuit_builder.add_public_variable(a);
    fr b = fr::random_element();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = circuit_builder.add_variable(b);
    uint32_t c_idx = circuit_builder.add_variable(c);
    uint32_t d_idx = circuit_builder.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        circuit_builder.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });
        circuit_builder.create_add_gate({ d_idx, c_idx, a_idx, 1, -1, -1, 0 });
    }

    // If Ultra arithmetization, add a big add gate with use of next row to test q_arith = 2
    if constexpr (IsUltraFlavor<Flavor>) {
        fr e = a + b + c + d;
        uint32_t e_idx = circuit_builder.add_variable(e);

        uint32_t zero_idx = circuit_builder.zero_idx;
        circuit_builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true); // use next row
        circuit_builder.create_big_add_gate({ zero_idx, zero_idx, zero_idx, e_idx, 0, 0, 0, 0, 0 }, false);
    }
}

template <typename Flavor> void create_some_lookup_gates(auto& circuit_builder)
{
    // Add some lookup gates (related to pedersen hashing)
    barretenberg::fr pedersen_input_value = fr::random_element();
    const fr input_hi = uint256_t(pedersen_input_value).slice(126, 256);
    const fr input_lo = uint256_t(pedersen_input_value).slice(0, 126);
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
    // Add some RAM gates
    uint32_t ram_values[8]{
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
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
    static_assert(IsGoblinFlavor<Flavor>);
    const size_t num_ecc_operations = 10; // arbitrary
    for (size_t i = 0; i < num_ecc_operations; ++i) {
        auto point = g1::affine_one * fr::random_element();
        auto scalar = fr::random_element();
        circuit_builder.queue_ecc_mul_accum(point, scalar);
    }
}

class RelationCorrectnessTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }
};

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
TEST_F(RelationCorrectnessTests, StandardRelationCorrectness)
{
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    // Create a composer and a dummy circuit with a few gates
    auto circuit_constructor = StandardCircuitBuilder();

    create_some_add_gates<Flavor>(circuit_constructor);

    // Create a prover (it will compute proving key and witness)
    auto composer = StandardComposer();
    auto prover = composer.create_prover(circuit_constructor);
    auto circuit_size = prover.key->circuit_size;

    // Generate beta and gamma
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = circuit_constructor.get_public_inputs();
    auto public_input_delta =
        honk::compute_public_input_delta<Flavor>(public_inputs, beta, gamma, prover.key->circuit_size);

    proof_system::RelationParameters<FF> params{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    // Create an array of spans to the underlying polynomials to more easily
    // get the transposition.
    // Ex: polynomial_spans[3][i] returns the i-th coefficient of the third polynomial
    // in the list below
    ProverPolynomials prover_polynomials;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
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

    // Compute grand product polynomial
    grand_product_library::compute_grand_products<honk::flavor::Standard>(prover.key, prover_polynomials, params);

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(proof_system::ArithmeticRelation<FF>(), proof_system::PermutationRelation<FF>());

    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<Flavor>(std::get<0>(relations), circuit_size, prover_polynomials, params);
    check_relation<Flavor>(std::get<1>(relations), circuit_size, prover_polynomials, params);
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
// TODO(luke): Add a gate that sets q_arith = 3 to check secondary arithmetic relation
TEST_F(RelationCorrectnessTests, UltraRelationCorrectness)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    // Create a composer and then add an assortment of gates designed to ensure that the constraint(s) represented
    // by each relation are non-trivially exercised.
    auto circuit_constructor = UltraCircuitBuilder();

    // Create an assortment of representative gates
    create_some_add_gates<Flavor>(circuit_constructor);
    create_some_lookup_gates<Flavor>(circuit_constructor);
    create_some_genperm_sort_gates<Flavor>(circuit_constructor);
    create_some_elliptic_curve_addition_gates<Flavor>(circuit_constructor);
    create_some_RAM_gates<Flavor>(circuit_constructor);

    // Create a prover (it will compute proving key and witness)
    auto composer = UltraComposer();
    auto prover = composer.create_prover(circuit_constructor);
    auto circuit_size = prover.key->circuit_size;

    // Generate eta, beta and gamma
    fr eta = fr::random_element();
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = circuit_constructor.get_public_inputs();
    const size_t pub_inputs_offset = Flavor::has_zero_row ? 1 : 0;
    auto public_input_delta = honk::compute_public_input_delta<Flavor>(
        public_inputs, beta, gamma, prover.key->circuit_size, pub_inputs_offset);
    auto lookup_grand_product_delta =
        honk::compute_lookup_grand_product_delta<FF>(beta, gamma, prover.key->circuit_size);

    proof_system::RelationParameters<FF> params{
        .eta = eta,
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
        .lookup_grand_product_delta = lookup_grand_product_delta,
    };

    // Compute sorted witness-table accumulator
    prover.key->sorted_accum = prover_library::compute_sorted_list_accumulator<Flavor>(prover.key, eta);

    // Add RAM/ROM memory records to wire four
    prover_library::add_plookup_memory_records_to_wire_4<Flavor>(prover.key, eta);

    ProverPolynomials prover_polynomials;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
    prover_polynomials.w_4 = prover.key->w_4;
    prover_polynomials.w_l_shift = prover.key->w_l.shifted();
    prover_polynomials.w_r_shift = prover.key->w_r.shifted();
    prover_polynomials.w_o_shift = prover.key->w_o.shifted();
    prover_polynomials.w_4_shift = prover.key->w_4.shifted();
    prover_polynomials.sorted_accum = prover.key->sorted_accum;
    prover_polynomials.sorted_accum_shift = prover.key->sorted_accum.shifted();
    prover_polynomials.table_1 = prover.key->table_1;
    prover_polynomials.table_2 = prover.key->table_2;
    prover_polynomials.table_3 = prover.key->table_3;
    prover_polynomials.table_4 = prover.key->table_4;
    prover_polynomials.table_1_shift = prover.key->table_1.shifted();
    prover_polynomials.table_2_shift = prover.key->table_2.shifted();
    prover_polynomials.table_3_shift = prover.key->table_3.shifted();
    prover_polynomials.table_4_shift = prover.key->table_4.shifted();
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

    // Compute grand product polynomials for permutation + lookup
    grand_product_library::compute_grand_products<Flavor>(prover.key, prover_polynomials, params);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(prover.key->q_arith);
    ensure_non_zero(prover.key->q_sort);
    ensure_non_zero(prover.key->q_lookup);
    ensure_non_zero(prover.key->q_elliptic);
    ensure_non_zero(prover.key->q_aux);

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(proof_system::UltraArithmeticRelation<FF>(),
                                proof_system::UltraPermutationRelation<FF>(),
                                proof_system::LookupRelation<FF>(),
                                proof_system::GenPermSortRelation<FF>(),
                                proof_system::EllipticRelation<FF>(),
                                proof_system::AuxiliaryRelation<FF>());

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
    using Flavor = honk::flavor::GoblinUltra;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    // Create a composer and then add an assortment of gates designed to ensure that the constraint(s) represented
    // by each relation are non-trivially exercised.
    auto builder = UltraCircuitBuilder();

    // Create an assortment of representative gates
    create_some_add_gates<Flavor>(builder);
    create_some_lookup_gates<Flavor>(builder);
    create_some_genperm_sort_gates<Flavor>(builder);
    create_some_elliptic_curve_addition_gates<Flavor>(builder);
    create_some_RAM_gates<Flavor>(builder);
    create_some_ecc_op_queue_gates<Flavor>(builder); // Goblin!

    // Create a prover (it will compute proving key and witness)
    auto composer = GoblinUltraComposer();
    auto prover = composer.create_prover(builder);
    auto circuit_size = prover.key->circuit_size;

    // Generate eta, beta and gamma
    fr eta = fr::random_element();
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = builder.get_public_inputs();

    // If Goblin, must account for the fact that PI are offset in the wire polynomials by the number of ecc op gates
    size_t pub_inputs_offset = Flavor::has_zero_row ? 1 : 0;
    if constexpr (IsGoblinFlavor<Flavor>) {
        pub_inputs_offset += builder.num_ecc_op_gates;
    }
    auto public_input_delta = honk::compute_public_input_delta<Flavor>(
        public_inputs, beta, gamma, prover.key->circuit_size, pub_inputs_offset);
    auto lookup_grand_product_delta =
        honk::compute_lookup_grand_product_delta<FF>(beta, gamma, prover.key->circuit_size);

    proof_system::RelationParameters<FF> params{
        .eta = eta,
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
        .lookup_grand_product_delta = lookup_grand_product_delta,
    };

    // Compute sorted witness-table accumulator
    prover.key->sorted_accum = prover_library::compute_sorted_list_accumulator<Flavor>(prover.key, eta);

    // Add RAM/ROM memory records to wire four
    prover_library::add_plookup_memory_records_to_wire_4<Flavor>(prover.key, eta);

    ProverPolynomials prover_polynomials;

    prover_polynomials.lagrange_ecc_op = prover.key->lagrange_ecc_op;
    prover_polynomials.ecc_op_wire_1 = prover.key->ecc_op_wire_1;
    prover_polynomials.ecc_op_wire_2 = prover.key->ecc_op_wire_2;
    prover_polynomials.ecc_op_wire_3 = prover.key->ecc_op_wire_3;
    prover_polynomials.ecc_op_wire_4 = prover.key->ecc_op_wire_4;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
    prover_polynomials.w_4 = prover.key->w_4;
    prover_polynomials.w_l_shift = prover.key->w_l.shifted();
    prover_polynomials.w_r_shift = prover.key->w_r.shifted();
    prover_polynomials.w_o_shift = prover.key->w_o.shifted();
    prover_polynomials.w_4_shift = prover.key->w_4.shifted();
    prover_polynomials.sorted_accum = prover.key->sorted_accum;
    prover_polynomials.sorted_accum_shift = prover.key->sorted_accum.shifted();
    prover_polynomials.table_1 = prover.key->table_1;
    prover_polynomials.table_2 = prover.key->table_2;
    prover_polynomials.table_3 = prover.key->table_3;
    prover_polynomials.table_4 = prover.key->table_4;
    prover_polynomials.table_1_shift = prover.key->table_1.shifted();
    prover_polynomials.table_2_shift = prover.key->table_2.shifted();
    prover_polynomials.table_3_shift = prover.key->table_3.shifted();
    prover_polynomials.table_4_shift = prover.key->table_4.shifted();
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

    // Compute grand product polynomials for permutation + lookup
    grand_product_library::compute_grand_products<Flavor>(prover.key, prover_polynomials, params);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(prover.key->q_arith);
    ensure_non_zero(prover.key->q_sort);
    ensure_non_zero(prover.key->q_lookup);
    ensure_non_zero(prover.key->q_elliptic);
    ensure_non_zero(prover.key->q_aux);

    // Construct the round for applying sumcheck relations and results for storing computed results
    auto relations = std::tuple(proof_system::UltraArithmeticRelation<FF>(),
                                proof_system::UltraPermutationRelation<FF>(),
                                proof_system::LookupRelation<FF>(),
                                proof_system::GenPermSortRelation<FF>(),
                                proof_system::EllipticRelation<FF>(),
                                proof_system::AuxiliaryRelation<FF>(),
                                proof_system::EccOpQueueRelation<FF>());

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
