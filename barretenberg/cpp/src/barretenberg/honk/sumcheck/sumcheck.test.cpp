#include "sumcheck.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/grand_product_library.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/proof_system/relations/auxiliary_relation.hpp"
#include "barretenberg/proof_system/relations/elliptic_relation.hpp"
#include "barretenberg/proof_system/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/proof_system/relations/lookup_relation.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include "barretenberg/proof_system/relations/ultra_arithmetic_relation.hpp"
#include <gtest/gtest.h>

using namespace proof_system::honk;
using namespace proof_system::honk::sumcheck;
using Flavor = proof_system::honk::flavor::Ultra; // TODO(Cody): Generalize this test.
using FF = typename Flavor::FF;
using ProverPolynomials = typename Flavor::ProverPolynomials;
const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

namespace test_sumcheck_round {

barretenberg::Polynomial<FF> random_poly(size_t size)
{
    auto poly = barretenberg::Polynomial<FF>(size);
    for (auto& coeff : poly) {
        coeff = FF::random_element();
    }
    return poly;
}

ProverPolynomials construct_ultra_full_polynomials(auto& input_polynomials)
{
    ProverPolynomials full_polynomials;
    full_polynomials.q_c = input_polynomials[0];
    full_polynomials.q_l = input_polynomials[1];
    full_polynomials.q_r = input_polynomials[2];
    full_polynomials.q_o = input_polynomials[3];
    full_polynomials.q_4 = input_polynomials[4];
    full_polynomials.q_m = input_polynomials[5];
    full_polynomials.q_arith = input_polynomials[6];
    full_polynomials.q_sort = input_polynomials[7];
    full_polynomials.q_elliptic = input_polynomials[8];
    full_polynomials.q_aux = input_polynomials[9];
    full_polynomials.q_lookup = input_polynomials[10];
    full_polynomials.sigma_1 = input_polynomials[11];
    full_polynomials.sigma_2 = input_polynomials[12];
    full_polynomials.sigma_3 = input_polynomials[13];
    full_polynomials.sigma_4 = input_polynomials[14];
    full_polynomials.id_1 = input_polynomials[15];
    full_polynomials.id_2 = input_polynomials[16];
    full_polynomials.id_3 = input_polynomials[17];
    full_polynomials.id_4 = input_polynomials[18];
    full_polynomials.table_1 = input_polynomials[19];
    full_polynomials.table_2 = input_polynomials[20];
    full_polynomials.table_3 = input_polynomials[21];
    full_polynomials.table_4 = input_polynomials[22];
    full_polynomials.lagrange_first = input_polynomials[23];
    full_polynomials.lagrange_last = input_polynomials[24];
    full_polynomials.w_l = input_polynomials[25];
    full_polynomials.w_r = input_polynomials[26];
    full_polynomials.w_o = input_polynomials[27];
    full_polynomials.w_4 = input_polynomials[28];
    full_polynomials.sorted_accum = input_polynomials[29];
    full_polynomials.z_perm = input_polynomials[30];
    full_polynomials.z_lookup = input_polynomials[31];
    full_polynomials.table_1_shift = input_polynomials[32];
    full_polynomials.table_2_shift = input_polynomials[33];
    full_polynomials.table_3_shift = input_polynomials[34];
    full_polynomials.table_4_shift = input_polynomials[35];
    full_polynomials.w_l_shift = input_polynomials[36];
    full_polynomials.w_r_shift = input_polynomials[37];
    full_polynomials.w_o_shift = input_polynomials[38];
    full_polynomials.w_4_shift = input_polynomials[39];
    full_polynomials.sorted_accum_shift = input_polynomials[40];
    full_polynomials.z_perm_shift = input_polynomials[41];
    full_polynomials.z_lookup_shift = input_polynomials[42];

    return full_polynomials;
}

class SumcheckTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(SumcheckTests, PolynomialNormalization)
{
    // TODO(#225)(Cody): We should not use real constants like this in the tests, at least not in so many of them.
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);

    // Randomly construct the prover polynomials that are input to Sumcheck.
    // Note: ProverPolynomials are defined as spans so the polynomials they point to need to exist in memory.
    std::array<barretenberg::Polynomial<FF>, NUM_POLYNOMIALS> random_polynomials;
    for (auto& poly : random_polynomials) {
        poly = random_poly(multivariate_n);
    }
    auto full_polynomials = construct_ultra_full_polynomials(random_polynomials);

    info(full_polynomials.w_l[0]);
    info(full_polynomials.w_l[1]);
    info(full_polynomials.w_l[2]);
    info(full_polynomials.w_l[3]);

    auto transcript = ProverTranscript<FF>::init_empty();

    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    auto output = sumcheck.prove(full_polynomials, {});

    FF u_0 = output.challenge[0];
    FF u_1 = output.challenge[1];
    FF u_2 = output.challenge[2];

    /* sumcheck.prove() terminates with sumcheck.multivariates.folded_polynoimals as an array such that
     * sumcheck.multivariates.folded_polynoimals[i][0] is the evaluatioin of the i'th multivariate at the vector of
     challenges u_i. What does this mean?

     Here we show that if the multivariate is F(X0, X1, X2) defined as above, then what we get is F(u0, u1, u2) and
     not, say F(u2, u1, u0). This is in accordance with Adrian's thesis (cf page 9).
      */

    // Get the values of the Lagrange basis polys L_i defined
    // by: L_i(v) = 1 if i = v, 0 otherwise, for v from 0 to 7.
    FF one{ 1 };
    // clang-format off
    FF l_0 = (one - u_0) * (one - u_1) * (one - u_2);
    FF l_1 = (      u_0) * (one - u_1) * (one - u_2);
    FF l_2 = (one - u_0) * (      u_1) * (one - u_2);
    FF l_3 = (      u_0) * (      u_1) * (one - u_2);
    FF l_4 = (one - u_0) * (one - u_1) * (      u_2);
    FF l_5 = (      u_0) * (one - u_1) * (      u_2);
    FF l_6 = (one - u_0) * (      u_1) * (      u_2);
    FF l_7 = (      u_0) * (      u_1) * (      u_2);
    // clang-format on
    FF hand_computed_value;
    for (size_t i = 0; i < NUM_POLYNOMIALS; i++) {
        // full_polynomials[0][0] = w_l[0], full_polynomials[1][1] = w_r[1], and so on.
        hand_computed_value = l_0 * full_polynomials[i][0] + l_1 * full_polynomials[i][1] +
                              l_2 * full_polynomials[i][2] + l_3 * full_polynomials[i][3] +
                              l_4 * full_polynomials[i][4] + l_5 * full_polynomials[i][5] +
                              l_6 * full_polynomials[i][6] + l_7 * full_polynomials[i][7];
        EXPECT_EQ(hand_computed_value, sumcheck.partially_evaluated_polynomials[i][0]);
    }

    // We can also check the correctness of the multilinear evaluations produced by Sumcheck by directly evaluating the
    // full polynomials at challenge u via the evaluate_mle() function
    std::vector<FF> u_challenge = { u_0, u_1, u_2 };
    for (size_t i = 0; i < NUM_POLYNOMIALS; i++) {
        barretenberg::Polynomial<FF> poly(full_polynomials[i]);
        auto v_expected = poly.evaluate_mle(u_challenge);
        auto v_result = output.claimed_evaluations[i];
        EXPECT_EQ(v_expected, v_result);
    }
}

TEST_F(SumcheckTests, Prover)
{
    const size_t multivariate_d(2);
    const size_t multivariate_n(1 << multivariate_d);

    // Randomly construct the prover polynomials that are input to Sumcheck.
    // Note: ProverPolynomials are defined as spans so the polynomials they point to need to exist in memory.
    std::array<barretenberg::Polynomial<FF>, NUM_POLYNOMIALS> random_polynomials;
    for (auto& poly : random_polynomials) {
        poly = random_poly(multivariate_n);
    }
    auto full_polynomials = construct_ultra_full_polynomials(random_polynomials);

    auto transcript = ProverTranscript<FF>::init_empty();

    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    auto output = sumcheck.prove(full_polynomials, {});
    FF u_0 = output.challenge[0];
    FF u_1 = output.challenge[1];
    std::vector<FF> expected_values;
    for (auto& polynomial : full_polynomials) {
        // using knowledge of inputs here to derive the evaluation
        FF expected_lo = polynomial[0] * (FF(1) - u_0) + polynomial[1] * u_0;
        expected_lo *= (FF(1) - u_1);
        FF expected_hi = polynomial[2] * (FF(1) - u_0) + polynomial[3] * u_0;
        expected_hi *= u_1;
        expected_values.emplace_back(expected_lo + expected_hi);
    }

    for (size_t poly_idx = 0; poly_idx < NUM_POLYNOMIALS; poly_idx++) {
        EXPECT_EQ(output.claimed_evaluations[poly_idx], expected_values[poly_idx]);
    }
}

// TODO(#225): make the inputs to this test more interesting, e.g. non-trivial permutations
TEST_F(SumcheckTests, ProverAndVerifierSimple)
{
    auto run_test = [](bool expect_verified) {
        const size_t multivariate_d(2);
        const size_t multivariate_n(1 << multivariate_d);

        // Construct prover polynomials where each is the zero polynomial.
        // Note: ProverPolynomials are defined as spans so the polynomials they point to need to exist in memory.
        std::array<barretenberg::Polynomial<FF>, NUM_POLYNOMIALS> zero_polynomials;
        for (auto& poly : zero_polynomials) {
            poly = barretenberg::Polynomial<FF>(multivariate_n);
        }
        auto full_polynomials = construct_ultra_full_polynomials(zero_polynomials);

        // Add some non-trivial values to certain polynomials so that the arithmetic relation will have non-trivial
        // contribution. Note: since all other polynomials are set to 0, all other relations are trivially satisfied.
        std::array<FF, multivariate_n> w_l;
        if (expect_verified) {
            w_l = { 0, 1, 2, 0 };
        } else {
            w_l = { 0, 0, 2, 0 };
        }
        std::array<FF, multivariate_n> w_r = { 0, 1, 2, 0 };
        std::array<FF, multivariate_n> w_o = { 0, 2, 4, 0 };
        std::array<FF, multivariate_n> w_4 = { 0, 0, 0, 0 };
        std::array<FF, multivariate_n> q_m = { 0, 0, 1, 0 };
        std::array<FF, multivariate_n> q_l = { 0, 1, 0, 0 };
        std::array<FF, multivariate_n> q_r = { 0, 1, 0, 0 };
        std::array<FF, multivariate_n> q_o = { 0, -1, -1, 0 };
        std::array<FF, multivariate_n> q_c = { 0, 0, 0, 0 };
        std::array<FF, multivariate_n> q_arith = { 0, 1, 1, 0 };
        // Setting all of these to 0 ensures the GrandProductRelation is satisfied

        full_polynomials.w_l = w_l;
        full_polynomials.w_r = w_r;
        full_polynomials.w_o = w_o;
        full_polynomials.w_4 = w_4;
        full_polynomials.q_m = q_m;
        full_polynomials.q_l = q_l;
        full_polynomials.q_r = q_r;
        full_polynomials.q_o = q_o;
        full_polynomials.q_c = q_c;
        full_polynomials.q_arith = q_arith;

        // Set aribitrary random relation parameters
        proof_system::RelationParameters<FF> relation_parameters{
            .beta = FF::random_element(),
            .gamma = FF::random_element(),
            .public_input_delta = FF::one(),
        };

        auto prover_transcript = ProverTranscript<FF>::init_empty();

        auto sumcheck_prover = SumcheckProver<Flavor>(multivariate_n, prover_transcript);

        auto prover_output = sumcheck_prover.prove(full_polynomials, relation_parameters);

        auto verifier_transcript = VerifierTranscript<FF>::init_empty(prover_transcript);

        auto sumcheck_verifier = SumcheckVerifier<Flavor>(multivariate_n);

        auto verifier_output = sumcheck_verifier.verify(relation_parameters, verifier_transcript);

        auto verified = verifier_output.verified.value();

        EXPECT_EQ(verified, expect_verified);
    };

    run_test(/* expect_verified=*/true);
    run_test(/* expect_verified=*/false);
}

/**
 * @brief Test the Ultra Sumcheck Prover and Verifier for a real circuit
 *
 */
TEST_F(SumcheckTests, RealCircuitUltra)
{
    using Flavor = flavor::Ultra;
    using FF = typename Flavor::FF;

    // Create a composer and a dummy circuit with a few gates
    auto builder = proof_system::UltraCircuitBuilder();
    FF a = FF::one();

    // Add some basic add gates, with a public input for good measure
    uint32_t a_idx = builder.add_public_variable(a);
    FF b = FF::one();
    FF c = a + b;
    FF d = a + c;
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        builder.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });
        builder.create_add_gate({ d_idx, c_idx, a_idx, 1, -1, -1, 0 });
    }

    // Add a big add gate with use of next row to test q_arith = 2
    FF e = a + b + c + d;
    uint32_t e_idx = builder.add_variable(e);

    uint32_t zero_idx = builder.zero_idx;
    builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true); // use next row
    builder.create_big_add_gate({ zero_idx, zero_idx, zero_idx, e_idx, 0, 0, 0, 0, 0 }, false);

    // Add some lookup gates (related to pedersen hashing)
    auto pedersen_input_value = FF::random_element();
    const FF input_hi = uint256_t(pedersen_input_value).slice(126, 256);
    const FF input_lo = uint256_t(pedersen_input_value).slice(0, 126);
    const auto input_hi_index = builder.add_variable(input_hi);
    const auto input_lo_index = builder.add_variable(input_lo);

    const auto sequence_data_hi = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_HI, input_hi);
    const auto sequence_data_lo = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_LO, input_lo);

    builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
    builder.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);

    // Add a sort gate (simply checks that consecutive inputs have a difference of < 4)
    a_idx = builder.add_variable(FF(0));
    b_idx = builder.add_variable(FF(1));
    c_idx = builder.add_variable(FF(2));
    d_idx = builder.add_variable(FF(3));
    builder.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });

    // Add an elliptic curve addition gate
    grumpkin::g1::affine_element p1 = crypto::generators::get_generator_data({ 0, 0 }).generator;
    grumpkin::g1::affine_element p2 = crypto::generators::get_generator_data({ 0, 1 }).generator;

    grumpkin::fq beta_scalar = grumpkin::fq::cube_root_of_unity();
    grumpkin::g1::affine_element p2_endo = p2;
    p2_endo.x *= beta_scalar;

    grumpkin::g1::affine_element p3(grumpkin::g1::element(p1) - grumpkin::g1::element(p2_endo));

    uint32_t x1 = builder.add_variable(p1.x);
    uint32_t y1 = builder.add_variable(p1.y);
    uint32_t x2 = builder.add_variable(p2.x);
    uint32_t y2 = builder.add_variable(p2.y);
    uint32_t x3 = builder.add_variable(p3.x);
    uint32_t y3 = builder.add_variable(p3.y);

    builder.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, beta_scalar, -1 });

    // Add some RAM gates
    uint32_t ram_values[8]{
        builder.add_variable(FF::random_element()), builder.add_variable(FF::random_element()),
        builder.add_variable(FF::random_element()), builder.add_variable(FF::random_element()),
        builder.add_variable(FF::random_element()), builder.add_variable(FF::random_element()),
        builder.add_variable(FF::random_element()), builder.add_variable(FF::random_element()),
    };

    size_t ram_id = builder.create_RAM_array(8);

    for (size_t i = 0; i < 8; ++i) {
        builder.init_RAM_element(ram_id, i, ram_values[i]);
    }

    a_idx = builder.read_RAM_array(ram_id, builder.add_variable(5));
    EXPECT_EQ(a_idx != ram_values[5], true);

    b_idx = builder.read_RAM_array(ram_id, builder.add_variable(4));
    c_idx = builder.read_RAM_array(ram_id, builder.add_variable(1));

    builder.write_RAM_array(ram_id, builder.add_variable(4), builder.add_variable(500));
    d_idx = builder.read_RAM_array(ram_id, builder.add_variable(4));

    EXPECT_EQ(builder.get_variable(d_idx), 500);

    // ensure these vars get used in another arithmetic gate
    const auto e_value = builder.get_variable(a_idx) + builder.get_variable(b_idx) + builder.get_variable(c_idx) +
                         builder.get_variable(d_idx);
    e_idx = builder.add_variable(e_value);

    builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true);
    builder.create_big_add_gate(
        {
            builder.zero_idx,
            builder.zero_idx,
            builder.zero_idx,
            e_idx,
            0,
            0,
            0,
            0,
            0,
        },
        false);

    // Create a prover (it will compute proving key and witness)
    auto composer = UltraComposer();
    auto instance = composer.create_instance(builder);

    // Generate eta, beta and gamma
    FF eta = FF::random_element();
    FF beta = FF::random_element();
    FF gamma = FF::random_element();

    instance->initialise_prover_polynomials();
    instance->compute_sorted_accumulator_polynomials(eta);
    instance->compute_grand_product_polynomials(beta, gamma);

    auto prover_transcript = ProverTranscript<FF>::init_empty();
    auto circuit_size = instance->proving_key->circuit_size;

    auto sumcheck_prover = SumcheckProver<Flavor>(circuit_size, prover_transcript);

    auto prover_output = sumcheck_prover.prove(instance->prover_polynomials, instance->relation_parameters);

    auto verifier_transcript = VerifierTranscript<FF>::init_empty(prover_transcript);

    auto sumcheck_verifier = SumcheckVerifier<Flavor>(circuit_size);

    auto verifier_output = sumcheck_verifier.verify(instance->relation_parameters, verifier_transcript);

    auto verified = verifier_output.verified.value();

    ASSERT_TRUE(verified);
}

} // namespace test_sumcheck_round
