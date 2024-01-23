#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::honk;

template <typename Flavor> class ECCVMTranscriptTests : public ::testing::Test {
  public:
    void SetUp() override
    {
        if constexpr (std::is_same<Flavor, flavor::ECCVM>::value) {
            srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
        } else {
            srs::init_crs_factory("../srs_db/ignition");
        }
    };
    using FF = typename Flavor::FF;

    /**
     * @brief Construct a manifest for a ECCVM Honk proof
     *
     * @details This is where we define the "Manifest" for a ECCVM Honk proof. The tests in this suite are
     * intented to warn the developer if the Prover/Verifier has deviated from this manifest, however, the
     * Transcript class is not otherwise contrained to follow the manifest.
     *
     * @note Entries in the manifest consist of a name string and a size (bytes), NOT actual data.
     *
     * @return TranscriptManifest
     */
    TranscriptManifest construct_eccvm_honk_manifest(size_t circuit_size, size_t ipa_poly_degree)
    {
        TranscriptManifest manifest_expected;

        auto log_n = numeric::get_msb(circuit_size);

        size_t MAX_PARTIAL_RELATION_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;
        size_t size_FF = sizeof(FF);
        size_t size_G = 2 * size_FF;
        size_t size_uni = MAX_PARTIAL_RELATION_LENGTH * size_FF;
        size_t size_evals = (Flavor::NUM_ALL_ENTITIES)*size_FF;
        size_t size_uint32 = 4;
        size_t size_uint64 = 8;

        size_t round = 0;
        manifest_expected.add_entry(round, "circuit_size", size_uint32);
        manifest_expected.add_entry(round, "TRANSCRIPT_ADD", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_MUL", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_EQ", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_COLLISION_CHECK", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_MSM_TRANSITION", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_PC", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_MSM_COUNT", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_PX", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_PY", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_Z1", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_Z2", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_Z1ZERO", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_Z2ZERO", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_OP", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_ACCUMULATOR_X", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_ACCUMULATOR_Y", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_MSM_X", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_MSM_Y", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_PC", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_POINT_TRANSITION", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_ROUND", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_SCALAR_SUM", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S1HI", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S1LO", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S2HI", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S2LO", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S3HI", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S3LO", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S4HI", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_S4LO", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_SKEW", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_DX", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_DY", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_TX", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_TY", size_G);
        manifest_expected.add_entry(round, "MSM_TRANSITION", size_G);
        manifest_expected.add_entry(round, "MSM_ADD", size_G);
        manifest_expected.add_entry(round, "MSM_DOUBLE", size_G);
        manifest_expected.add_entry(round, "MSM_SKEW", size_G);
        manifest_expected.add_entry(round, "MSM_ACCUMULATOR_X", size_G);
        manifest_expected.add_entry(round, "MSM_ACCUMULATOR_Y", size_G);
        manifest_expected.add_entry(round, "MSM_PC", size_G);
        manifest_expected.add_entry(round, "MSM_SIZE_OF_MSM", size_G);
        manifest_expected.add_entry(round, "MSM_COUNT", size_G);
        manifest_expected.add_entry(round, "MSM_ROUND", size_G);
        manifest_expected.add_entry(round, "MSM_ADD1", size_G);
        manifest_expected.add_entry(round, "MSM_ADD2", size_G);
        manifest_expected.add_entry(round, "MSM_ADD3", size_G);
        manifest_expected.add_entry(round, "MSM_ADD4", size_G);
        manifest_expected.add_entry(round, "MSM_X1", size_G);
        manifest_expected.add_entry(round, "MSM_Y1", size_G);
        manifest_expected.add_entry(round, "MSM_X2", size_G);
        manifest_expected.add_entry(round, "MSM_Y2", size_G);
        manifest_expected.add_entry(round, "MSM_X3", size_G);
        manifest_expected.add_entry(round, "MSM_Y3", size_G);
        manifest_expected.add_entry(round, "MSM_X4", size_G);
        manifest_expected.add_entry(round, "MSM_Y4", size_G);
        manifest_expected.add_entry(round, "MSM_COLLISION_X1", size_G);
        manifest_expected.add_entry(round, "MSM_COLLISION_X2", size_G);
        manifest_expected.add_entry(round, "MSM_COLLISION_X3", size_G);
        manifest_expected.add_entry(round, "MSM_COLLISION_X4", size_G);
        manifest_expected.add_entry(round, "MSM_LAMBDA1", size_G);
        manifest_expected.add_entry(round, "MSM_LAMBDA2", size_G);
        manifest_expected.add_entry(round, "MSM_LAMBDA3", size_G);
        manifest_expected.add_entry(round, "MSM_LAMBDA4", size_G);
        manifest_expected.add_entry(round, "MSM_SLICE1", size_G);
        manifest_expected.add_entry(round, "MSM_SLICE2", size_G);
        manifest_expected.add_entry(round, "MSM_SLICE3", size_G);
        manifest_expected.add_entry(round, "MSM_SLICE4", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_ACCUMULATOR_EMPTY", size_G);
        manifest_expected.add_entry(round, "TRANSCRIPT_RESET_ACCUMULATOR", size_G);
        manifest_expected.add_entry(round, "PRECOMPUTE_SELECT", size_G);
        manifest_expected.add_entry(round, "LOOKUP_READ_COUNTS_0", size_G);
        manifest_expected.add_entry(round, "LOOKUP_READ_COUNTS_1", size_G);
        manifest_expected.add_challenge(round, "beta", "gamma");

        round++;
        manifest_expected.add_entry(round, "LOOKUP_INVERSES", size_G);
        manifest_expected.add_entry(round, "Z_PERM", size_G);
        manifest_expected.add_challenge(round, "Sumcheck:alpha");

        for (size_t i = 0; i < log_n; i++) {
            round++;
            std::string label = "Sumcheck:gate_challenge_" + std::to_string(i);
            manifest_expected.add_challenge(round, label);
        }

        for (size_t i = 0; i < log_n; ++i) {
            round++;
            std::string idx = std::to_string(i);
            manifest_expected.add_entry(round, "Sumcheck:univariate_" + idx, size_uni);
            std::string label = "Sumcheck:u_" + idx;
            manifest_expected.add_challenge(round, label);
        }

        round++;
        manifest_expected.add_entry(round, "Sumcheck:evaluations", size_evals);
        manifest_expected.add_challenge(round, "rho");

        round++;
        for (size_t i = 1; i < log_n; ++i) {
            std::string idx = std::to_string(i);
            manifest_expected.add_entry(round, "Gemini:FOLD_" + idx, size_G);
        }
        manifest_expected.add_challenge(round, "Gemini:r");

        round++;
        for (size_t i = 0; i < log_n; ++i) {
            std::string idx = std::to_string(i);
            manifest_expected.add_entry(round, "Gemini:a_" + idx, size_FF);
        }
        manifest_expected.add_challenge(round, "Shplonk:nu");

        round++;
        manifest_expected.add_entry(round, "Shplonk:Q", size_G);
        manifest_expected.add_challenge(round, "Shplonk:z");

        round++;
        manifest_expected.add_entry(round, "IPA:poly_degree", size_uint64);
        manifest_expected.add_challenge(round, "IPA:generator_challenge");

        auto log_poly_degree = static_cast<size_t>(numeric::get_msb(ipa_poly_degree));
        for (size_t i = 0; i < log_poly_degree; ++i) {
            round++;
            std::string idx = std::to_string(i);
            manifest_expected.add_entry(round, "IPA:L_" + idx, size_G);
            manifest_expected.add_entry(round, "IPA:R_" + idx, size_G);
            std::string label = "IPA:round_challenge_" + idx;
            manifest_expected.add_challenge(round, label);
        }

        round++;
        manifest_expected.add_entry(round, "IPA:a_0", size_FF);

        return manifest_expected;
    }
    ECCVMCircuitBuilder<Flavor> generate_trace(numeric::RNG* engine = nullptr)
    {
        ECCVMCircuitBuilder<Flavor> result;
        using G1 = typename Flavor::CycleGroup;
        using Fr = typename G1::Fr;

        auto generators = G1::derive_generators("test generators", 3);

        typename G1::element a = generators[0];
        typename G1::element b = generators[1];
        typename G1::element c = generators[2];
        Fr x = Fr::random_element(engine);
        Fr y = Fr::random_element(engine);

        typename G1::element expected_1 = (a * x) + a + a + (b * y) + (b * x) + (b * x);
        typename G1::element expected_2 = (a * x) + c + (b * x);

        result.add_accumulate(a);
        result.mul_accumulate(a, x);
        result.mul_accumulate(b, x);
        result.mul_accumulate(b, y);
        result.add_accumulate(a);
        result.mul_accumulate(b, x);
        result.eq_and_reset(expected_1);
        result.add_accumulate(c);
        result.mul_accumulate(a, x);
        result.mul_accumulate(b, x);
        result.eq_and_reset(expected_2);
        result.mul_accumulate(a, x);
        result.mul_accumulate(b, x);
        result.mul_accumulate(c, x);

        return result;
    }
};

numeric::RNG& engine = numeric::get_debug_randomness();

using FlavorTypes = testing::Types<flavor::ECCVM>;

TYPED_TEST_SUITE(ECCVMTranscriptTests, FlavorTypes);
/**
 * @brief Ensure consistency between the manifest hard coded in this testing suite and the one generated by the
 * standard honk prover over the course of proof construction.
 */
TYPED_TEST(ECCVMTranscriptTests, ProverManifestConsistency)
{
    GTEST_SKIP() << "TODO(https://github.com/AztecProtocol/barretenberg/issues/782): update and reinstate after the "
                    "protocol is finalized.";
    using Flavor = TypeParam;

    // Construct a simple circuit
    auto builder = this->generate_trace(&engine);

    // Automatically generate a transcript manifest by constructing a proof
    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(builder);
    auto proof = prover.construct_proof();

    // Check that the prover generated manifest agrees with the manifest hard coded in this suite
    auto manifest_expected =
        this->construct_eccvm_honk_manifest(prover.key->circuit_size, prover.shplonk_output.witness.size());
    auto prover_manifest = prover.transcript->get_manifest();

    // Note: a manifest can be printed using manifest.print()
    for (size_t round = 0; round < manifest_expected.size(); ++round) {
        ASSERT_EQ(prover_manifest[round], manifest_expected[round]) << "Prover manifest discrepency in round " << round;
    }
}

/**
 * @brief Ensure consistency between the manifest generated by the ECCVM honk prover over the course of proof
 * construction and the one generated by the verifier over the course of proof verification.
 *
 */
TYPED_TEST(ECCVMTranscriptTests, VerifierManifestConsistency)
{
    GTEST_SKIP() << "TODO(https://github.com/AztecProtocol/barretenberg/issues/782): update and reinstate after the "
                    "protocol is finalized.";

    using Flavor = TypeParam;

    // Construct a simple circuit
    auto builder = this->generate_trace(&engine);

    // Automatically generate a transcript manifest in the prover by constructing a proof
    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(builder);
    auto proof = prover.construct_proof();

    // Automatically generate a transcript manifest in the verifier by verifying a proof
    auto verifier = composer.create_verifier(builder);
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
TYPED_TEST(ECCVMTranscriptTests, ChallengeGenerationTest)
{
    using Flavor = TypeParam;
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
    auto [a, b, c] = challenges_to_field_elements<typename Flavor::FF>(transcript->get_challenges("a", "b", "c"));

    ASSERT_NE(a, 0) << "Challenge a is 0";
    ASSERT_NE(b, 0) << "Challenge b is 0";
    ASSERT_NE(c, 0) << "Challenge c is 0";
}

TYPED_TEST(ECCVMTranscriptTests, StructureTest)
{
    GTEST_SKIP() << "TODO(https://github.com/AztecProtocol/barretenberg/issues/782): update and reinstate after the "
                    "protocol is finalized.";

    using Flavor = TypeParam;

    // Construct a simple circuit
    auto builder = this->generate_trace(&engine);

    // Automatically generate a transcript manifest by constructing a proof
    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(builder);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    EXPECT_TRUE(verifier.verify_proof(proof));

    // try deserializing and serializing with no changes and check proof is still valid
    prover.transcript->deserialize_full_transcript();
    prover.transcript->serialize_full_transcript();
    EXPECT_TRUE(verifier.verify_proof(prover.export_proof())); // we have changed nothing so proof is still valid

    typename Flavor::Commitment one_group_val = Flavor::Commitment::one();
    auto rand_val = Flavor::FF::random_element();
    prover.transcript->transcript_Px_comm = one_group_val * rand_val; // choose random object to modify
    EXPECT_TRUE(verifier.verify_proof(
        prover.export_proof())); // we have not serialized it back to the proof so it should still be fine

    prover.transcript->serialize_full_transcript();
    EXPECT_FALSE(verifier.verify_proof(prover.export_proof())); // the proof is now wrong after serializing it

    prover.transcript->deserialize_full_transcript();
    EXPECT_EQ(static_cast<typename Flavor::Commitment>(prover.transcript->transcript_Px_comm),
              one_group_val * rand_val);
}