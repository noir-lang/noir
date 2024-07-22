#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/relations/auxiliary_relation.hpp"
#include "barretenberg/relations/delta_range_constraint_relation.hpp"
#include "barretenberg/relations/ecc_op_queue_relation.hpp"
#include "barretenberg/relations/elliptic_relation.hpp"
#include "barretenberg/relations/logderiv_lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/fixed_base/fixed_base.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"

#include <gtest/gtest.h>
using namespace bb;

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
 */
template <typename Relation> void check_relation(auto circuit_size, auto& polynomials, auto params)
{
    for (size_t i = 0; i < circuit_size; i++) {
        // Define the appropriate SumcheckArrayOfValuesOverSubrelations type for this relation and initialize to zero
        using SumcheckArrayOfValuesOverSubrelations = typename Relation::SumcheckArrayOfValuesOverSubrelations;
        SumcheckArrayOfValuesOverSubrelations result;
        for (auto& element : result) {
            element = 0;
        }

        // Evaluate each constraint in the relation and check that each is satisfied
        Relation::accumulate(result, polynomials.get_row(i), params, 1);
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
    const auto input_lo = uint256_t(pedersen_input_value).slice(0, bb::plookup::fixed_base::table::BITS_PER_LO_SCALAR);
    const auto input_hi_index = circuit_builder.add_variable(input_hi);
    const auto input_lo_index = circuit_builder.add_variable(input_lo);

    const auto sequence_data_hi =
        plookup::get_lookup_accumulators(bb::plookup::MultiTableId::FIXED_BASE_LEFT_HI, input_hi);
    const auto sequence_data_lo =
        plookup::get_lookup_accumulators(bb::plookup::MultiTableId::FIXED_BASE_LEFT_LO, input_lo);

    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::FIXED_BASE_LEFT_HI, sequence_data_hi, input_hi_index);
    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::FIXED_BASE_LEFT_LO, sequence_data_lo, input_lo_index);
}

template <typename Flavor> void create_some_delta_range_constraint_gates(auto& circuit_builder)
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
    static_assert(IsGoblinFlavor<Flavor>);
    const size_t num_ecc_operations = 10; // arbitrary
    for (size_t i = 0; i < num_ecc_operations; ++i) {
        auto point = G1::affine_one * FF::random_element();
        auto scalar = FF::random_element();
        circuit_builder.queue_ecc_mul_accum(point, scalar);
    }
}

class UltraRelationCorrectnessTests : public ::testing::Test {
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
TEST_F(UltraRelationCorrectnessTests, Ultra)
{
    using Flavor = UltraFlavor;
    using FF = typename Flavor::FF;

    // Create a composer and then add an assortment of gates designed to ensure that the constraint(s) represented
    // by each relation are non-trivially exercised.
    auto builder = UltraCircuitBuilder();

    // Create an assortment of representative gates
    create_some_add_gates<Flavor>(builder);
    create_some_lookup_gates<Flavor>(builder);
    create_some_delta_range_constraint_gates<Flavor>(builder);
    create_some_elliptic_curve_addition_gates<Flavor>(builder);
    create_some_RAM_gates<Flavor>(builder);

    // Create a prover (it will compute proving key and witness)
    auto instance = std::make_shared<ProverInstance_<Flavor>>(builder);
    auto& proving_key = instance->proving_key;
    auto circuit_size = proving_key.circuit_size;

    // Generate eta, beta and gamma
    instance->relation_parameters.eta = FF::random_element();
    instance->relation_parameters.eta_two = FF::random_element();
    instance->relation_parameters.eta_three = FF::random_element();
    instance->relation_parameters.beta = FF::random_element();
    instance->relation_parameters.gamma = FF::random_element();

    instance->proving_key.add_ram_rom_memory_records_to_wire_4(instance->relation_parameters.eta,
                                                               instance->relation_parameters.eta_two,
                                                               instance->relation_parameters.eta_three);
    instance->proving_key.compute_logderivative_inverses(instance->relation_parameters);
    instance->proving_key.compute_grand_product_polynomials(instance->relation_parameters);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(proving_key.polynomials.q_arith);
    ensure_non_zero(proving_key.polynomials.q_delta_range);
    ensure_non_zero(proving_key.polynomials.q_lookup);
    ensure_non_zero(proving_key.polynomials.q_elliptic);
    ensure_non_zero(proving_key.polynomials.q_aux);

    auto& prover_polynomials = instance->proving_key.polynomials;
    auto params = instance->relation_parameters;
    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<UltraArithmeticRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<UltraPermutationRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<DeltaRangeConstraintRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<EllipticRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<AuxiliaryRelation<FF>>(circuit_size, prover_polynomials, params);
    check_linearly_dependent_relation<Flavor, LogDerivLookupRelation<FF>>(circuit_size, prover_polynomials, params);
}

TEST_F(UltraRelationCorrectnessTests, Mega)
{
    using Flavor = MegaFlavor;
    using FF = typename Flavor::FF;

    // Create a composer and then add an assortment of gates designed to ensure that the constraint(s) represented
    // by each relation are non-trivially exercised.
    auto builder = MegaCircuitBuilder();

    // Create an assortment of representative gates
    create_some_add_gates<Flavor>(builder);
    create_some_lookup_gates<Flavor>(builder);
    create_some_delta_range_constraint_gates<Flavor>(builder);
    create_some_elliptic_curve_addition_gates<Flavor>(builder);
    create_some_RAM_gates<Flavor>(builder);
    create_some_ecc_op_queue_gates<Flavor>(builder); // Goblin!

    // Create a prover (it will compute proving key and witness)
    auto instance = std::make_shared<ProverInstance_<Flavor>>(builder);
    auto& proving_key = instance->proving_key;
    auto circuit_size = proving_key.circuit_size;

    // Generate eta, beta and gamma
    instance->relation_parameters.eta = FF::random_element();
    instance->relation_parameters.eta_two = FF::random_element();
    instance->relation_parameters.eta_three = FF::random_element();
    instance->relation_parameters.beta = FF::random_element();
    instance->relation_parameters.gamma = FF::random_element();

    instance->proving_key.add_ram_rom_memory_records_to_wire_4(instance->relation_parameters.eta,
                                                               instance->relation_parameters.eta_two,
                                                               instance->relation_parameters.eta_three);
    instance->proving_key.compute_logderivative_inverses(instance->relation_parameters);
    instance->proving_key.compute_grand_product_polynomials(instance->relation_parameters);

    // Check that selectors are nonzero to ensure corresponding relation has nontrivial contribution
    ensure_non_zero(proving_key.polynomials.q_arith);
    ensure_non_zero(proving_key.polynomials.q_delta_range);
    ensure_non_zero(proving_key.polynomials.q_lookup);
    ensure_non_zero(proving_key.polynomials.q_elliptic);
    ensure_non_zero(proving_key.polynomials.q_aux);
    ensure_non_zero(proving_key.polynomials.q_busread);
    ensure_non_zero(proving_key.polynomials.q_poseidon2_external);
    ensure_non_zero(proving_key.polynomials.q_poseidon2_internal);

    ensure_non_zero(proving_key.polynomials.calldata);
    ensure_non_zero(proving_key.polynomials.calldata_read_counts);
    ensure_non_zero(proving_key.polynomials.calldata_inverses);
    ensure_non_zero(proving_key.polynomials.secondary_calldata);
    ensure_non_zero(proving_key.polynomials.secondary_calldata_read_counts);
    ensure_non_zero(proving_key.polynomials.secondary_calldata_inverses);
    ensure_non_zero(proving_key.polynomials.return_data);
    ensure_non_zero(proving_key.polynomials.return_data_read_counts);
    ensure_non_zero(proving_key.polynomials.return_data_inverses);

    auto& prover_polynomials = instance->proving_key.polynomials;
    auto params = instance->relation_parameters;

    // Check that each relation is satisfied across each row of the prover polynomials
    check_relation<UltraArithmeticRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<UltraPermutationRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<DeltaRangeConstraintRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<EllipticRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<AuxiliaryRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<EccOpQueueRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<Poseidon2ExternalRelation<FF>>(circuit_size, prover_polynomials, params);
    check_relation<Poseidon2InternalRelation<FF>>(circuit_size, prover_polynomials, params);
    check_linearly_dependent_relation<Flavor, DatabusLookupRelation<FF>>(circuit_size, prover_polynomials, params);
    check_linearly_dependent_relation<Flavor, LogDerivLookupRelation<FF>>(circuit_size, prover_polynomials, params);
}