#include <gtest/gtest.h>

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/honk/sumcheck/polynomials/univariate.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/stdlib/recursion/honk/transcript/trancript.hpp"

namespace proof_system::plonk::stdlib::recursion::honk {

using Builder = UltraCircuitBuilder;

using FF = barretenberg::fr;
using Commitment = barretenberg::g1::affine_element;
using Point = barretenberg::g1::element;
constexpr size_t LENGTH = 8; // arbitrary
using Univariate = proof_system::honk::sumcheck::Univariate<FF, LENGTH>;
using ProverTranscript = ::proof_system::honk::ProverTranscript<FF>;
using VerifierTranscript = ::proof_system::honk::VerifierTranscript<FF>;

/**
 * @brief Create some mock data and then add it to the transcript in various mock rounds
 *
 * @param prover_transcript
 * @return auto proof_data
 */
auto generate_mock_proof_data(auto prover_transcript)
{
    uint32_t data = 25;
    auto scalar = FF::random_element();
    auto commitment = Commitment::one();

    std::array<FF, LENGTH> evaluations;
    for (auto& eval : evaluations) {
        eval = FF::random_element();
    }
    auto univariate = Univariate(evaluations);

    // round 0
    prover_transcript.send_to_verifier("data", data);
    prover_transcript.get_challenge("alpha");

    // round 1
    prover_transcript.send_to_verifier("scalar", scalar);
    prover_transcript.send_to_verifier("commitment", commitment);
    prover_transcript.get_challenges("beta, gamma");

    // round 2
    prover_transcript.send_to_verifier("univariate", univariate);
    prover_transcript.get_challenges("gamma", "delta");

    return prover_transcript.proof_data;
}

/**
 * @brief Perform series of verifier transcript operations
 * @details Operations are designed to correspond to those performed by a prover transcript from which the verifier
 * transcript was initialized.
 *
 * @param transcript Either a native or stdlib verifier transcript
 */
void perform_mock_verifier_transcript_operations(auto transcript)
{
    // round 0
    transcript.template receive_from_prover<uint32_t>("data");
    transcript.get_challenge("alpha");

    // round 1
    transcript.template receive_from_prover<FF>("scalar");
    transcript.template receive_from_prover<Commitment>("commitment");
    transcript.get_challenges("beta, gamma");

    // round 2
    transcript.template receive_from_prover<Univariate>("univariate");
    transcript.get_challenges("gamma", "delta");
}

/**
 * @brief Test basic transcript functionality and check circuit
 * @details Implicitly ensures stdlib interface is identical to native
 * @todo(luke): Underlying circuit is nearly trivial until transcript implements hashing constraints
 */
TEST(stdlib_honk_transcript, basic_transcript_operations)
{
    Builder builder;

    // Instantiate a Prover Transcript and use it to generate some mock proof data
    ProverTranscript prover_transcript;
    auto proof_data = generate_mock_proof_data(prover_transcript);

    // Instantiate a (native) Verifier Transcript with the proof data and perform some mock transcript operations
    VerifierTranscript native_transcript(proof_data);
    perform_mock_verifier_transcript_operations(native_transcript);

    // Confirm that Prover and Verifier transcripts have generated the same manifest via the operations performed
    EXPECT_EQ(prover_transcript.get_manifest(), native_transcript.get_manifest());

    // Instantiate a stdlib Transcript and perform the same operations
    Transcript<Builder> transcript{ &builder, proof_data };
    perform_mock_verifier_transcript_operations(transcript);

    // Confirm that the native and stdlib transcripts have generated the same manifest
    EXPECT_EQ(transcript.get_manifest(), native_transcript.get_manifest());

    // TODO(luke): This doesn't check much of anything until hashing is constrained in the stdlib transcript
    EXPECT_TRUE(builder.check_circuit());
}

/**
 * @brief Check that native and stdlib verifier transcript functions produce equivalent outputs
 *
 */
TEST(stdlib_honk_transcript, return_values)
{
    Builder builder;

    // Define some mock data for a mock proof
    auto scalar = FF::random_element();
    auto commitment = Commitment::one() * FF::random_element();

    const size_t LENGTH = 10; // arbitrary
    std::array<FF, LENGTH> evaluations;
    for (auto& eval : evaluations) {
        eval = FF::random_element();
    }

    // Construct a mock proof via the prover transcript
    ProverTranscript prover_transcript;
    prover_transcript.send_to_verifier("scalar", scalar);
    prover_transcript.send_to_verifier("commitment", commitment);
    prover_transcript.send_to_verifier("evaluations", evaluations);
    prover_transcript.get_challenges("alpha, beta");
    auto proof_data = prover_transcript.proof_data;

    // Perform the corresponding operations with the native verifier transcript
    VerifierTranscript native_transcript(proof_data);
    auto native_scalar = native_transcript.template receive_from_prover<FF>("scalar");
    auto native_commitment = native_transcript.template receive_from_prover<Commitment>("commitment");
    auto native_evaluations = native_transcript.template receive_from_prover<std::array<FF, LENGTH>>("evaluations");
    auto [native_alpha, native_beta] = native_transcript.get_challenges("alpha", "beta");

    // Perform the corresponding operations with the stdlib verifier transcript
    Transcript<Builder> stdlib_transcript{ &builder, proof_data };
    auto stdlib_scalar = stdlib_transcript.template receive_from_prover<FF>("scalar");
    auto stdlib_commitment = stdlib_transcript.template receive_from_prover<Commitment>("commitment");
    auto stdlib_evaluations = stdlib_transcript.template receive_from_prover<std::array<FF, LENGTH>>("evaluations");
    auto [stdlib_alpha, stdlib_beta] = stdlib_transcript.get_challenges("alpha", "beta");

    // Confirm that return values are equivalent
    EXPECT_EQ(native_scalar, stdlib_scalar.get_value());
    EXPECT_EQ(native_commitment, stdlib_commitment.get_value());
    for (size_t i = 0; i < LENGTH; ++i) {
        EXPECT_EQ(native_evaluations[i], stdlib_evaluations[i].get_value());
    }
    EXPECT_EQ(native_alpha, stdlib_alpha.get_value());
    EXPECT_EQ(native_beta, stdlib_beta.get_value());
}

} // namespace proof_system::plonk::stdlib::recursion::honk