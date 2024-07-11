#include "barretenberg/common/serialize.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/fixed_base/fixed_base.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/types.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

#include <gtest/gtest.h>

using namespace bb;

using ProverInstance = ProverInstance_<UltraFlavor>;
using VerificationKey = UltraFlavor::VerificationKey;

std::vector<uint32_t> add_variables(auto& circuit_builder, std::vector<bb::fr> variables)
{
    std::vector<uint32_t> res;
    for (size_t i = 0; i < variables.size(); i++) {
        res.emplace_back(circuit_builder.add_variable(variables[i]));
    }
    return res;
}

void prove_and_verify(auto& circuit_builder, bool expected_result)
{
    auto instance = std::make_shared<ProverInstance>(circuit_builder);
    UltraProver prover(instance);
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    UltraVerifier verifier(verification_key);
    auto proof = prover.construct_proof();
    bool verified = verifier.verify_proof(proof);
    EXPECT_EQ(verified, expected_result);
};

void ensure_non_zero(auto& polynomial)
{
    bool has_non_zero_coefficient = false;
    for (auto& coeff : polynomial) {
        has_non_zero_coefficient |= !coeff.is_zero();
    }
    ASSERT_TRUE(has_non_zero_coefficient);
}

class UltraHonkTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

/**
 * @brief A quick test to ensure that none of our polynomials are identically zero
 *
 * @note This test assumes that gates have been added by default in the composer
 * to achieve non-zero polynomials
 *
 */
TEST_F(UltraHonkTests, ANonZeroPolynomialIsAGoodPolynomial)
{
    auto circuit_builder = UltraCircuitBuilder();

    auto instance = std::make_shared<ProverInstance>(circuit_builder);
    UltraProver prover(instance);
    auto proof = prover.construct_proof();
    auto& polynomials = instance->proving_key.polynomials;

    for (auto& poly : polynomials.get_selectors()) {
        ensure_non_zero(poly);
    }

    for (auto& poly : polynomials.get_tables()) {
        ensure_non_zero(poly);
    }

    for (auto& poly : polynomials.get_wires()) {
        ensure_non_zero(poly);
    }
}

/**
 * @brief Test proof construction/verification for a structured execution trace
 *
 */
TEST_F(UltraHonkTests, StructuredTrace)
{
    auto builder = UltraCircuitBuilder();
    size_t num_gates = 3;

    // Add some arbitrary arithmetic gates that utilize public inputs
    MockCircuits::add_arithmetic_gates_with_public_inputs(builder, num_gates);

    // Construct an instance with a structured execution trace
    TraceStructure trace_structure = TraceStructure::SMALL_TEST;
    auto instance = std::make_shared<ProverInstance>(builder, trace_structure);
    info(instance->proving_key.circuit_size);
    UltraProver prover(instance);
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    UltraVerifier verifier(verification_key);
    auto proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

/**
 * @brief Test simple circuit with public inputs
 *
 */
TEST_F(UltraHonkTests, PublicInputs)
{
    auto builder = UltraCircuitBuilder();
    size_t num_gates = 10;

    // Add some arbitrary arithmetic gates that utilize public inputs
    MockCircuits::add_arithmetic_gates_with_public_inputs(builder, num_gates);

    prove_and_verify(builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, XorConstraint)
{
    auto circuit_builder = UltraCircuitBuilder();

    uint32_t left_value = engine.get_random_uint32();
    uint32_t right_value = engine.get_random_uint32();

    fr left_witness_value = fr{ left_value, 0, 0, 0 }.to_montgomery_form();
    fr right_witness_value = fr{ right_value, 0, 0, 0 }.to_montgomery_form();

    uint32_t left_witness_index = circuit_builder.add_variable(left_witness_value);
    uint32_t right_witness_index = circuit_builder.add_variable(right_witness_value);

    uint32_t xor_result_expected = left_value ^ right_value;

    const auto lookup_accumulators = plookup::get_lookup_accumulators(
        plookup::MultiTableId::UINT32_XOR, left_witness_value, right_witness_value, true);
    auto xor_result = lookup_accumulators[plookup::ColumnIdx::C3]
                                         [0]; // The zeroth index in the 3rd column is the fully accumulated xor

    EXPECT_EQ(xor_result, xor_result_expected);
    circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::UINT32_XOR, lookup_accumulators, left_witness_index, right_witness_index);

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, create_gates_from_plookup_accumulators)
{
    auto circuit_builder = UltraCircuitBuilder();

    fr input_value = fr::random_element();
    const fr input_lo = static_cast<uint256_t>(input_value).slice(0, plookup::fixed_base::table::BITS_PER_LO_SCALAR);
    const auto input_lo_index = circuit_builder.add_variable(input_lo);

    const auto sequence_data_lo = plookup::get_lookup_accumulators(plookup::MultiTableId::FIXED_BASE_LEFT_LO, input_lo);

    const auto lookup_witnesses = circuit_builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::FIXED_BASE_LEFT_LO, sequence_data_lo, input_lo_index);

    const size_t num_lookups = plookup::fixed_base::table::NUM_TABLES_PER_LO_MULTITABLE;

    EXPECT_EQ(num_lookups, lookup_witnesses[plookup::ColumnIdx::C1].size());

    {
        const auto mask = plookup::fixed_base::table::MAX_TABLE_SIZE - 1;

        grumpkin::g1::affine_element base_point = plookup::fixed_base::table::LHS_GENERATOR_POINT;
        std::vector<uint8_t> input_buf;
        write(input_buf, base_point);
        const auto offset_generators =
            grumpkin::g1::derive_generators(input_buf, plookup::fixed_base::table::NUM_TABLES_PER_LO_MULTITABLE);

        grumpkin::g1::element accumulator = base_point;
        uint256_t expected_scalar(input_lo);
        const auto table_bits = plookup::fixed_base::table::BITS_PER_TABLE;
        const auto num_tables = plookup::fixed_base::table::NUM_TABLES_PER_LO_MULTITABLE;
        for (size_t i = 0; i < num_tables; ++i) {

            auto round_scalar = circuit_builder.get_variable(lookup_witnesses[plookup::ColumnIdx::C1][i]);
            auto round_x = circuit_builder.get_variable(lookup_witnesses[plookup::ColumnIdx::C2][i]);
            auto round_y = circuit_builder.get_variable(lookup_witnesses[plookup::ColumnIdx::C3][i]);

            EXPECT_EQ(uint256_t(round_scalar), expected_scalar);

            auto next_scalar = static_cast<uint256_t>(
                (i == num_tables - 1) ? fr(0)
                                      : circuit_builder.get_variable(lookup_witnesses[plookup::ColumnIdx::C1][i + 1]));

            uint256_t slice = static_cast<uint256_t>(round_scalar) - (next_scalar << table_bits);
            EXPECT_EQ(slice, (uint256_t(input_lo) >> (i * table_bits)) & mask);

            grumpkin::g1::affine_element expected_point(accumulator * static_cast<uint256_t>(slice) +
                                                        offset_generators[i]);

            EXPECT_EQ(round_x, expected_point.x);
            EXPECT_EQ(round_y, expected_point.y);
            for (size_t j = 0; j < table_bits; ++j) {
                accumulator = accumulator.dbl();
            }
            expected_scalar >>= table_bits;
        }
    }

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

/**
 * @brief Test various failure modes for the lookup relation via bad input polynomials
 *
 */
TEST_F(UltraHonkTests, LookupFailure)
{
    // Construct a circuit with lookup and arithmetic gates
    auto construct_circuit_with_lookups = []() {
        UltraCircuitBuilder builder;

        MockCircuits::add_lookup_gates(builder);
        MockCircuits::add_arithmetic_gates(builder);

        return builder;
    };

    auto prove_and_verify = [](auto& instance) {
        UltraProver prover(instance);
        auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
        UltraVerifier verifier(verification_key);
        auto proof = prover.construct_proof();
        return verifier.verify_proof(proof);
    };

    // Ensure the unaltered test circuit is valid
    {
        auto builder = construct_circuit_with_lookups();

        auto instance = std::make_shared<ProverInstance>(builder);

        EXPECT_TRUE(prove_and_verify(instance));
    }

    // Failure mode 1: bad read counts/tags
    {
        auto builder = construct_circuit_with_lookups();

        auto instance = std::make_shared<ProverInstance>(builder);
        auto& polynomials = instance->proving_key.polynomials;

        // Erroneously update the read counts/tags at an arbitrary index
        // Note: updating only one or the other may not cause failure due to the design of the relation algebra. For
        // example, the inverse is only computed if read tags is non-zero, otherwise the inverse at the row in question
        // will be zero. So if read counts is incremented at some arbitrary index but read tags is not, the inverse will
        // be 0 and the erroneous read_counts value will get multiplied by 0 in the relation. This is expected behavior.
        polynomials.lookup_read_counts[25] = 1;
        polynomials.lookup_read_tags[25] = 1;

        EXPECT_FALSE(prove_and_verify(instance));
    }

    // Failure mode 2: bad lookup gate wire value
    {
        auto builder = construct_circuit_with_lookups();

        auto instance = std::make_shared<ProverInstance>(builder);
        auto& polynomials = instance->proving_key.polynomials;

        // Find a lookup gate and alter one of the wire values
        for (auto [q_lookup, wire_3] : zip_view(polynomials.q_lookup, polynomials.w_o)) {
            if (!q_lookup.is_zero()) {
                wire_3 += 1;
                break;
            }
        }

        EXPECT_FALSE(prove_and_verify(instance));
    }

    // Failure mode 3: erroneous lookup gate
    {
        auto builder = construct_circuit_with_lookups();

        auto instance = std::make_shared<ProverInstance>(builder);
        auto& polynomials = instance->proving_key.polynomials;

        // Turn the lookup selector on for an arbitrary row where it is not already active
        EXPECT_TRUE(polynomials.q_lookup[25] != 1);
        polynomials.q_lookup[25] = 1;

        EXPECT_FALSE(prove_and_verify(instance));
    }
}

TEST_F(UltraHonkTests, test_no_lookup_proof)
{
    auto circuit_builder = UltraCircuitBuilder();

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = circuit_builder.add_variable(fr(left));
            uint32_t right_idx = circuit_builder.add_variable(fr(right));
            uint32_t result_idx = circuit_builder.add_variable(fr(left ^ right));

            uint32_t add_idx =
                circuit_builder.add_variable(fr(left) + fr(right) + circuit_builder.get_variable(result_idx));
            circuit_builder.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, test_elliptic_gate)
{
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;
    auto circuit_builder = UltraCircuitBuilder();

    affine_element p1 = affine_element::random_element();
    affine_element p2 = affine_element::random_element();

    affine_element p3(element(p1) + element(p2));

    uint32_t x1 = circuit_builder.add_variable(p1.x);
    uint32_t y1 = circuit_builder.add_variable(p1.y);
    uint32_t x2 = circuit_builder.add_variable(p2.x);
    uint32_t y2 = circuit_builder.add_variable(p2.y);
    uint32_t x3 = circuit_builder.add_variable(p3.x);
    uint32_t y3 = circuit_builder.add_variable(p3.y);

    circuit_builder.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, 1 });

    p3 = affine_element(element(p1) + element(p2));
    x3 = circuit_builder.add_variable(p3.x);
    y3 = circuit_builder.add_variable(p3.y);
    circuit_builder.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, 1 });

    p3 = affine_element(element(p1) - element(p2));
    x3 = circuit_builder.add_variable(p3.x);
    y3 = circuit_builder.add_variable(p3.y);
    circuit_builder.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, -1 });

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, non_trivial_tag_permutation)
{
    auto circuit_builder = UltraCircuitBuilder();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = circuit_builder.add_variable(a);
    auto b_idx = circuit_builder.add_variable(b);
    auto c_idx = circuit_builder.add_variable(b);
    auto d_idx = circuit_builder.add_variable(a);

    circuit_builder.create_add_gate(
        { a_idx, b_idx, circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });
    circuit_builder.create_add_gate(
        { c_idx, d_idx, circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });

    circuit_builder.create_tag(1, 2);
    circuit_builder.create_tag(2, 1);

    circuit_builder.assign_tag(a_idx, 1);
    circuit_builder.assign_tag(b_idx, 1);
    circuit_builder.assign_tag(c_idx, 2);
    circuit_builder.assign_tag(d_idx, 2);

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, non_trivial_tag_permutation_and_cycles)
{
    auto circuit_builder = UltraCircuitBuilder();
    fr a = fr::random_element();
    fr c = -a;

    auto a_idx = circuit_builder.add_variable(a);
    auto b_idx = circuit_builder.add_variable(a);
    circuit_builder.assert_equal(a_idx, b_idx);
    auto c_idx = circuit_builder.add_variable(c);
    auto d_idx = circuit_builder.add_variable(c);
    circuit_builder.assert_equal(c_idx, d_idx);
    auto e_idx = circuit_builder.add_variable(a);
    auto f_idx = circuit_builder.add_variable(a);
    circuit_builder.assert_equal(e_idx, f_idx);
    auto g_idx = circuit_builder.add_variable(c);
    auto h_idx = circuit_builder.add_variable(c);
    circuit_builder.assert_equal(g_idx, h_idx);

    circuit_builder.create_tag(1, 2);
    circuit_builder.create_tag(2, 1);

    circuit_builder.assign_tag(a_idx, 1);
    circuit_builder.assign_tag(c_idx, 1);
    circuit_builder.assign_tag(e_idx, 2);
    circuit_builder.assign_tag(g_idx, 2);

    circuit_builder.create_add_gate(
        { b_idx, a_idx, circuit_builder.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    circuit_builder.create_add_gate(
        { c_idx, g_idx, circuit_builder.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });
    circuit_builder.create_add_gate(
        { e_idx, f_idx, circuit_builder.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, bad_tag_permutation)
{
    {
        auto circuit_builder = UltraCircuitBuilder();
        fr a = fr::random_element();
        fr b = -a;

        auto a_idx = circuit_builder.add_variable(a);
        auto b_idx = circuit_builder.add_variable(b);
        auto c_idx = circuit_builder.add_variable(b);
        auto d_idx = circuit_builder.add_variable(a + 1);

        circuit_builder.create_add_gate({ a_idx, b_idx, circuit_builder.zero_idx, 1, 1, 0, 0 });
        circuit_builder.create_add_gate({ c_idx, d_idx, circuit_builder.zero_idx, 1, 1, 0, -1 });

        circuit_builder.create_tag(1, 2);
        circuit_builder.create_tag(2, 1);

        circuit_builder.assign_tag(a_idx, 1);
        circuit_builder.assign_tag(b_idx, 1);
        circuit_builder.assign_tag(c_idx, 2);
        circuit_builder.assign_tag(d_idx, 2);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
    // Same as above but without tag creation to check reason of failure is really tag mismatch
    {
        auto circuit_builder = UltraCircuitBuilder();
        fr a = fr::random_element();
        fr b = -a;

        auto a_idx = circuit_builder.add_variable(a);
        auto b_idx = circuit_builder.add_variable(b);
        auto c_idx = circuit_builder.add_variable(b);
        auto d_idx = circuit_builder.add_variable(a + 1);

        circuit_builder.create_add_gate({ a_idx, b_idx, circuit_builder.zero_idx, 1, 1, 0, 0 });
        circuit_builder.create_add_gate({ c_idx, d_idx, circuit_builder.zero_idx, 1, 1, 0, -1 });

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }
}

TEST_F(UltraHonkTests, sort_widget)
{
    auto circuit_builder = UltraCircuitBuilder();
    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(4);

    auto a_idx = circuit_builder.add_variable(a);
    auto b_idx = circuit_builder.add_variable(b);
    auto c_idx = circuit_builder.add_variable(c);
    auto d_idx = circuit_builder.add_variable(d);
    circuit_builder.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, sort_with_edges_gate)
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
        auto circuit_builder = UltraCircuitBuilder();
        auto a_idx = circuit_builder.add_variable(a);
        auto b_idx = circuit_builder.add_variable(b);
        auto c_idx = circuit_builder.add_variable(c);
        auto d_idx = circuit_builder.add_variable(d);
        auto e_idx = circuit_builder.add_variable(e);
        auto f_idx = circuit_builder.add_variable(f);
        auto g_idx = circuit_builder.add_variable(g);
        auto h_idx = circuit_builder.add_variable(h);
        circuit_builder.create_sort_constraint_with_edges(
            { a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, a, h);

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }

    {
        auto circuit_builder = UltraCircuitBuilder();
        auto a_idx = circuit_builder.add_variable(a);
        auto b_idx = circuit_builder.add_variable(b);
        auto c_idx = circuit_builder.add_variable(c);
        auto d_idx = circuit_builder.add_variable(d);
        auto e_idx = circuit_builder.add_variable(e);
        auto f_idx = circuit_builder.add_variable(f);
        auto g_idx = circuit_builder.add_variable(g);
        auto h_idx = circuit_builder.add_variable(h);
        circuit_builder.create_sort_constraint_with_edges(
            { a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, a, g);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto a_idx = circuit_builder.add_variable(a);
        auto b_idx = circuit_builder.add_variable(b);
        auto c_idx = circuit_builder.add_variable(c);
        auto d_idx = circuit_builder.add_variable(d);
        auto e_idx = circuit_builder.add_variable(e);
        auto f_idx = circuit_builder.add_variable(f);
        auto g_idx = circuit_builder.add_variable(g);
        auto h_idx = circuit_builder.add_variable(h);
        circuit_builder.create_sort_constraint_with_edges(
            { a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, b, h);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto a_idx = circuit_builder.add_variable(a);
        auto c_idx = circuit_builder.add_variable(c);
        auto d_idx = circuit_builder.add_variable(d);
        auto e_idx = circuit_builder.add_variable(e);
        auto f_idx = circuit_builder.add_variable(f);
        auto g_idx = circuit_builder.add_variable(g);
        auto h_idx = circuit_builder.add_variable(h);
        auto b2_idx = circuit_builder.add_variable(fr(15));
        circuit_builder.create_sort_constraint_with_edges(
            { a_idx, b2_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, b, h);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto idx = add_variables(circuit_builder, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                                    26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });
        circuit_builder.create_sort_constraint_with_edges(idx, 1, 45);

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto idx = add_variables(circuit_builder, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                                    26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });
        circuit_builder.create_sort_constraint_with_edges(idx, 1, 29);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
}

TEST_F(UltraHonkTests, range_constraint)
{
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto indices = add_variables(circuit_builder, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < indices.size(); i++) {
            circuit_builder.create_new_range_constraint(indices[i], 8);
        }
        // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
        circuit_builder.create_sort_constraint(indices);

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto indices = add_variables(circuit_builder, { 3 });
        for (size_t i = 0; i < indices.size(); i++) {
            circuit_builder.create_new_range_constraint(indices[i], 3);
        }
        // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
        circuit_builder.create_dummy_constraints(indices);

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto indices = add_variables(circuit_builder, { 1, 2, 3, 4, 5, 6, 8, 25 });
        for (size_t i = 0; i < indices.size(); i++) {
            circuit_builder.create_new_range_constraint(indices[i], 8);
        }
        circuit_builder.create_sort_constraint(indices);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto indices =
            add_variables(circuit_builder, { 1, 2, 3, 4, 5, 6, 10, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 19, 51 });
        for (size_t i = 0; i < indices.size(); i++) {
            circuit_builder.create_new_range_constraint(indices[i], 128);
        }
        circuit_builder.create_dummy_constraints(indices);

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto indices =
            add_variables(circuit_builder, { 1, 2, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            circuit_builder.create_new_range_constraint(indices[i], 79);
        }
        circuit_builder.create_dummy_constraints(indices);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
    {
        auto circuit_builder = UltraCircuitBuilder();
        auto indices =
            add_variables(circuit_builder, { 1, 0, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            circuit_builder.create_new_range_constraint(indices[i], 79);
        }
        circuit_builder.create_dummy_constraints(indices);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
}

TEST_F(UltraHonkTests, range_with_gates)
{
    auto circuit_builder = UltraCircuitBuilder();
    auto idx = add_variables(circuit_builder, { 1, 2, 3, 4, 5, 6, 7, 8 });
    for (size_t i = 0; i < idx.size(); i++) {
        circuit_builder.create_new_range_constraint(idx[i], 8);
    }

    circuit_builder.create_add_gate({ idx[0], idx[1], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -3 });
    circuit_builder.create_add_gate({ idx[2], idx[3], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -7 });
    circuit_builder.create_add_gate(
        { idx[4], idx[5], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -11 });
    circuit_builder.create_add_gate(
        { idx[6], idx[7], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -15 });

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, range_with_gates_where_range_is_not_a_power_of_two)
{
    auto circuit_builder = UltraCircuitBuilder();
    auto idx = add_variables(circuit_builder, { 1, 2, 3, 4, 5, 6, 7, 8 });
    for (size_t i = 0; i < idx.size(); i++) {
        circuit_builder.create_new_range_constraint(idx[i], 12);
    }

    circuit_builder.create_add_gate({ idx[0], idx[1], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -3 });
    circuit_builder.create_add_gate({ idx[2], idx[3], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -7 });
    circuit_builder.create_add_gate(
        { idx[4], idx[5], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -11 });
    circuit_builder.create_add_gate(
        { idx[6], idx[7], circuit_builder.zero_idx, fr::one(), fr::one(), fr::zero(), -15 });

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, sort_widget_complex)
{
    {

        auto circuit_builder = UltraCircuitBuilder();
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 11, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(circuit_builder.add_variable(a[i]));
        circuit_builder.create_sort_constraint(ind);

        prove_and_verify(circuit_builder, /*expected_result=*/true);
    }
    {

        auto circuit_builder = UltraCircuitBuilder();
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 16, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(circuit_builder.add_variable(a[i]));
        circuit_builder.create_sort_constraint(ind);

        prove_and_verify(circuit_builder, /*expected_result=*/false);
    }
}

TEST_F(UltraHonkTests, sort_widget_neg)
{
    auto circuit_builder = UltraCircuitBuilder();
    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(8);

    auto a_idx = circuit_builder.add_variable(a);
    auto b_idx = circuit_builder.add_variable(b);
    auto c_idx = circuit_builder.add_variable(c);
    auto d_idx = circuit_builder.add_variable(d);
    circuit_builder.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });

    prove_and_verify(circuit_builder, /*expected_result=*/false);
}

TEST_F(UltraHonkTests, composed_range_constraint)
{
    auto circuit_builder = UltraCircuitBuilder();
    auto c = fr::random_element();
    auto d = uint256_t(c).slice(0, 133);
    auto e = fr(d);
    auto a_idx = circuit_builder.add_variable(fr(e));
    circuit_builder.create_add_gate({ a_idx, circuit_builder.zero_idx, circuit_builder.zero_idx, 1, 0, 0, -fr(e) });
    circuit_builder.decompose_into_default_range(a_idx, 134);

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, non_native_field_multiplication)
{
    using fq = fq;
    auto circuit_builder = UltraCircuitBuilder();

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
        limb_indices[0] = circuit_builder.add_variable(limbs[0]);
        limb_indices[1] = circuit_builder.add_variable(limbs[1]);
        limb_indices[2] = circuit_builder.add_variable(limbs[2]);
        limb_indices[3] = circuit_builder.add_variable(limbs[3]);
        limb_indices[4] = circuit_builder.add_variable(limbs[4]);
        return limb_indices;
    };
    const uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (68 * 4);
    auto modulus_limbs = split_into_limbs(BINARY_BASIS_MODULUS - uint512_t(modulus));

    const auto a_indices = get_limb_witness_indices(split_into_limbs(uint256_t(a)));
    const auto b_indices = get_limb_witness_indices(split_into_limbs(uint256_t(b)));
    const auto q_indices = get_limb_witness_indices(split_into_limbs(uint256_t(q)));
    const auto r_indices = get_limb_witness_indices(split_into_limbs(uint256_t(r)));

    non_native_field_witnesses<fr> inputs{
        a_indices, b_indices, q_indices, r_indices, modulus_limbs, fr(uint256_t(modulus)),
    };
    const auto [lo_1_idx, hi_1_idx] = circuit_builder.evaluate_non_native_field_multiplication(inputs);
    circuit_builder.range_constrain_two_limbs(lo_1_idx, hi_1_idx, 70, 70);

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, rom)
{
    auto circuit_builder = UltraCircuitBuilder();

    uint32_t rom_values[8]{
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
        circuit_builder.add_variable(fr::random_element()), circuit_builder.add_variable(fr::random_element()),
    };

    size_t rom_id = circuit_builder.create_ROM_array(8);

    for (size_t i = 0; i < 8; ++i) {
        circuit_builder.set_ROM_element(rom_id, i, rom_values[i]);
    }

    uint32_t a_idx = circuit_builder.read_ROM_array(rom_id, circuit_builder.add_variable(5));
    EXPECT_EQ(a_idx != rom_values[5], true);
    uint32_t b_idx = circuit_builder.read_ROM_array(rom_id, circuit_builder.add_variable(4));
    uint32_t c_idx = circuit_builder.read_ROM_array(rom_id, circuit_builder.add_variable(1));

    const auto d_value =
        circuit_builder.get_variable(a_idx) + circuit_builder.get_variable(b_idx) + circuit_builder.get_variable(c_idx);
    uint32_t d_idx = circuit_builder.add_variable(d_value);

    circuit_builder.create_big_add_gate({
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

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, ram)
{
    auto circuit_builder = UltraCircuitBuilder();

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

    uint32_t a_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(5));
    EXPECT_EQ(a_idx != ram_values[5], true);

    uint32_t b_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(4));
    uint32_t c_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(1));

    circuit_builder.write_RAM_array(ram_id, circuit_builder.add_variable(4), circuit_builder.add_variable(500));
    uint32_t d_idx = circuit_builder.read_RAM_array(ram_id, circuit_builder.add_variable(4));

    EXPECT_EQ(circuit_builder.get_variable(d_idx), 500);

    // ensure these vars get used in another arithmetic gate
    const auto e_value = circuit_builder.get_variable(a_idx) + circuit_builder.get_variable(b_idx) +
                         circuit_builder.get_variable(c_idx) + circuit_builder.get_variable(d_idx);
    uint32_t e_idx = circuit_builder.add_variable(e_value);

    circuit_builder.create_big_add_gate(
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

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

TEST_F(UltraHonkTests, range_checks_on_duplicates)
{
    auto circuit_builder = UltraCircuitBuilder();

    uint32_t a = circuit_builder.add_variable(100);
    uint32_t b = circuit_builder.add_variable(100);
    uint32_t c = circuit_builder.add_variable(100);
    uint32_t d = circuit_builder.add_variable(100);

    circuit_builder.assert_equal(a, b);
    circuit_builder.assert_equal(a, c);
    circuit_builder.assert_equal(a, d);

    circuit_builder.create_new_range_constraint(a, 1000);
    circuit_builder.create_new_range_constraint(b, 1001);
    circuit_builder.create_new_range_constraint(c, 999);
    circuit_builder.create_new_range_constraint(d, 1000);

    circuit_builder.create_big_add_gate(
        {
            a,
            b,
            c,
            d,
            0,
            0,
            0,
            0,
            0,
        },
        false);

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}

// Ensure copy constraints added on variables smaller than 2^14, which have been previously
// range constrained, do not break the set equivalence checks because of indices mismatch.
// 2^14 is DEFAULT_PLOOKUP_RANGE_BITNUM i.e. the maximum size before a variable gets sliced
// before range constraints are applied to it.
TEST_F(UltraHonkTests, range_constraint_small_variable)
{
    auto circuit_builder = UltraCircuitBuilder();

    uint16_t mask = (1 << 8) - 1;
    int a = engine.get_random_uint16() & mask;
    uint32_t a_idx = circuit_builder.add_variable(fr(a));
    uint32_t b_idx = circuit_builder.add_variable(fr(a));
    ASSERT_NE(a_idx, b_idx);
    uint32_t c_idx = circuit_builder.add_variable(fr(a));
    ASSERT_NE(c_idx, b_idx);
    circuit_builder.create_range_constraint(b_idx, 8, "bad range");
    circuit_builder.assert_equal(a_idx, b_idx);
    circuit_builder.create_range_constraint(c_idx, 8, "bad range");
    circuit_builder.assert_equal(a_idx, c_idx);

    prove_and_verify(circuit_builder, /*expected_result=*/true);
}
