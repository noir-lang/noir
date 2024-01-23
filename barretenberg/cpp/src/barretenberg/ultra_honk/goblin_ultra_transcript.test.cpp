#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::honk;

class GoblinUltraTranscriptTests : public ::testing::Test {
  public:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    using Flavor = honk::flavor::GoblinUltra;
    using FF = Flavor::FF;

    /**
     * @brief Construct a manifest for a GoblinUltra Honk proof
     *
     * @details This is where we define the "Manifest" for a GoblinUltra Honk proof. The tests in this suite are
     * intented to warn the developer if the Prover/Verifier has deviated from this manifest, however, the
     * Transcript class is not otherwise contrained to follow the manifest.
     *
     * @note Entries in the manifest consist of a name string and a size (bytes), NOT actual data.
     *
     * @return TranscriptManifest
     */
    static TranscriptManifest construct_goblin_ultra_honk_manifest(size_t circuit_size)
    {
        TranscriptManifest manifest_expected;

        auto log_n = numeric::get_msb(circuit_size);

        size_t MAX_PARTIAL_RELATION_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;
        size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;

        size_t size_FF = sizeof(FF);
        size_t size_G = 2 * size_FF;
        size_t size_uni = MAX_PARTIAL_RELATION_LENGTH * size_FF;
        size_t size_evals = (Flavor::NUM_ALL_ENTITIES)*size_FF;
        size_t size_uint32 = 4;

        size_t round = 0;
        manifest_expected.add_entry(round, "circuit_size", size_uint32);
        manifest_expected.add_entry(round, "public_input_size", size_uint32);
        manifest_expected.add_entry(round, "pub_inputs_offset", size_uint32);
        manifest_expected.add_entry(round, "public_input_0", size_FF);
        manifest_expected.add_entry(round, "W_L", size_G);
        manifest_expected.add_entry(round, "W_R", size_G);
        manifest_expected.add_entry(round, "W_O", size_G);
        manifest_expected.add_entry(round, "ECC_OP_WIRE_1", size_G);
        manifest_expected.add_entry(round, "ECC_OP_WIRE_2", size_G);
        manifest_expected.add_entry(round, "ECC_OP_WIRE_3", size_G);
        manifest_expected.add_entry(round, "ECC_OP_WIRE_4", size_G);
        manifest_expected.add_entry(round, "CALLDATA", size_G);
        manifest_expected.add_entry(round, "CALLDATA_READ_COUNTS", size_G);
        manifest_expected.add_challenge(round, "eta");

        round++;
        manifest_expected.add_entry(round, "SORTED_ACCUM", size_G);
        manifest_expected.add_entry(round, "W_4", size_G);
        manifest_expected.add_challenge(round, "beta", "gamma");

        round++;
        manifest_expected.add_entry(round, "LOOKUP_INVERSES", size_G);
        manifest_expected.add_entry(round, "Z_PERM", size_G);
        manifest_expected.add_entry(round, "Z_LOOKUP", size_G);

        for (size_t i = 0; i < NUM_SUBRELATIONS - 1; i++) {
            std::string label = "Sumcheck:alpha_" + std::to_string(i);
            manifest_expected.add_challenge(round, label);
            round++;
        }

        for (size_t i = 0; i < log_n; i++) {
            std::string label = "Sumcheck:gate_challenge_" + std::to_string(i);
            manifest_expected.add_challenge(round, label);
            round++;
        }

        for (size_t i = 0; i < log_n; ++i) {
            std::string idx = std::to_string(i);
            manifest_expected.add_entry(round, "Sumcheck:univariate_" + idx, size_uni);
            std::string label = "Sumcheck:u_" + idx;
            manifest_expected.add_challenge(round, label);
            round++;
        }

        manifest_expected.add_entry(round, "Sumcheck:evaluations", size_evals);
        manifest_expected.add_challenge(round, "rho");

        round++;
        for (size_t i = 0; i < log_n; ++i) {
            std::string idx = std::to_string(i);
            manifest_expected.add_entry(round, "ZM:C_q_" + idx, size_G);
        }
        manifest_expected.add_challenge(round, "ZM:y");

        round++;
        manifest_expected.add_entry(round, "ZM:C_q", size_G);
        manifest_expected.add_challenge(round, "ZM:x", "ZM:z");

        round++;
        // TODO(Mara): Make testing more flavor agnostic so we can test this with all flavors
        manifest_expected.add_entry(round, "ZM:PI", size_G);
        manifest_expected.add_challenge(round); // no challenge

        return manifest_expected;
    }

    void generate_test_circuit(auto& builder)
    {
        // Add some ecc op gates
        for (size_t i = 0; i < 3; ++i) {
            auto point = Flavor::Curve::AffineElement::one() * FF::random_element();
            auto scalar = FF::random_element();
            builder.queue_ecc_mul_accum(point, scalar);
        }
        builder.queue_ecc_eq();

        // Add one conventional gates that utilize public inputs
        FF a = FF::random_element();
        FF b = FF::random_element();
        FF c = FF::random_element();
        FF d = a + b + c;
        uint32_t a_idx = builder.add_public_variable(a);
        uint32_t b_idx = builder.add_variable(b);
        uint32_t c_idx = builder.add_variable(c);
        uint32_t d_idx = builder.add_variable(d);

        builder.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, FF(1), FF(1), FF(1), FF(-1), FF(0) });
    }
};

/**
 * @brief Ensure consistency between the manifest hard coded in this testing suite and the one generated by the
 * standard honk prover over the course of proof construction.
 */
TEST_F(GoblinUltraTranscriptTests, ProverManifestConsistency)
{
    // Construct a simple circuit of size n = 8 (i.e. the minimum circuit size)
    auto builder = typename Flavor::CircuitBuilder();
    generate_test_circuit(builder);

    // Automatically generate a transcript manifest by constructing a proof
    auto composer = GoblinUltraComposer();
    auto instance = composer.create_instance(builder);
    auto prover = composer.create_prover(instance);
    auto proof = prover.construct_proof();

    // Check that the prover generated manifest agrees with the manifest hard coded in this suite
    auto manifest_expected = construct_goblin_ultra_honk_manifest(instance->proving_key->circuit_size);
    auto prover_manifest = prover.transcript->get_manifest();
    // Note: a manifest can be printed using manifest.print()
    for (size_t round = 0; round < manifest_expected.size(); ++round) {
        ASSERT_EQ(prover_manifest[round], manifest_expected[round]) << "Prover manifest discrepency in round " << round;
    }
}

/**
 * @brief Ensure consistency between the manifest generated by the goblin ultra honk prover over the course of proof
 * construction and the one generated by the verifier over the course of proof verification.
 *
 */
TEST_F(GoblinUltraTranscriptTests, VerifierManifestConsistency)
{

    // Construct a simple circuit of size n = 8 (i.e. the minimum circuit size)
    auto builder = Flavor::CircuitBuilder();
    generate_test_circuit(builder);

    // Automatically generate a transcript manifest in the prover by constructing a proof
    auto composer = GoblinUltraComposer();
    auto instance = composer.create_instance(builder);
    auto prover = composer.create_prover(instance);
    auto proof = prover.construct_proof();

    // Automatically generate a transcript manifest in the verifier by verifying a proof
    auto verifier = composer.create_verifier(instance);
    verifier.verify_proof(proof);

    // Check consistency between the manifests generated by the prover and verifier
    auto prover_manifest = prover.transcript->get_manifest();
    auto verifier_manifest = verifier.transcript->get_manifest();

    // Note: a manifest can be printed using manifest.print()
    for (size_t round = 0; round < prover_manifest.size(); ++round) {
        ASSERT_EQ(prover_manifest[round], verifier_manifest[round])
            << "Prover/Verifier manifest discrepency in round " << round;
    }
}

/**
 * @brief Check that multiple challenges can be generated and sanity check
 * @details We generate 6 challenges that are each 128 bits, and check that they are not 0.
 *
 */
TEST_F(GoblinUltraTranscriptTests, ChallengeGenerationTest)
{
    // initialized with random value sent to verifier
    auto transcript = Flavor::Transcript::prover_init_empty();
    // test a bunch of challenges
    auto challenges = transcript->get_challenges("a", "b", "c", "d", "e", "f");
    // check they are not 0
    for (size_t i = 0; i < challenges.size(); ++i) {
        ASSERT_NE(challenges[i], 0) << "Challenge " << i << " is 0";
    }
    constexpr uint32_t random_val{ 17 }; // arbitrary
    transcript->send_to_verifier("random val", random_val);
    // test more challenges
    auto [a, b, c] = challenges_to_field_elements<FF>(transcript->get_challenges("a", "b", "c"));
    ASSERT_NE(a, 0) << "Challenge a is 0";
    ASSERT_NE(b, 0) << "Challenge b is 0";
    ASSERT_NE(c, 0) << "Challenge c is 0";
}

TEST_F(GoblinUltraTranscriptTests, StructureTest)
{
    // Construct a simple circuit of size n = 8 (i.e. the minimum circuit size)
    auto builder = typename Flavor::CircuitBuilder();
    generate_test_circuit(builder);

    // Automatically generate a transcript manifest by constructing a proof
    auto composer = GoblinUltraComposer();
    auto instance = composer.create_instance(builder);
    auto prover = composer.create_prover(instance);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(instance);
    EXPECT_TRUE(verifier.verify_proof(proof));

    // try deserializing and serializing with no changes and check proof is still valid
    prover.transcript->deserialize_full_transcript();
    prover.transcript->serialize_full_transcript();
    EXPECT_TRUE(verifier.verify_proof(prover.export_proof())); // we have changed nothing so proof is still valid

    Flavor::Commitment one_group_val = Flavor::Commitment::one();
    FF rand_val = FF::random_element();
    prover.transcript->sorted_accum_comm = one_group_val * rand_val; // choose random object to modify
    EXPECT_TRUE(verifier.verify_proof(
        prover.export_proof())); // we have not serialized it back to the proof so it should still be fine

    prover.transcript->serialize_full_transcript();
    EXPECT_FALSE(verifier.verify_proof(prover.export_proof())); // the proof is now wrong after serializing it

    prover.transcript->deserialize_full_transcript();
    EXPECT_EQ(static_cast<Flavor::Commitment>(prover.transcript->sorted_accum_comm), one_group_val * rand_val);
}
