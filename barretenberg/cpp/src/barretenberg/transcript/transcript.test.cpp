#include "barretenberg/transcript/transcript.hpp"
#include <gtest/gtest.h>

namespace bb::honk_transcript_tests {

using FF = bb::fr;
using Fr = bb::fr;
using Fq = bb::fq;
using Transcript = bb::honk::BaseTranscript;

/**
 * @brief Test sending, receiving, and exporting proofs
 *
 */
TEST(BaseTranscript, TwoProversTwoFields)
{
    const auto EXPECT_STATE = [](const Transcript& transcript, size_t start, size_t written, size_t read) {
        EXPECT_EQ(transcript.proof_start, static_cast<std::ptrdiff_t>(start));
        EXPECT_EQ(transcript.num_bytes_written, written);
        EXPECT_EQ(transcript.num_bytes_read, read);
    };

    Transcript prover_transcript;
    // state initializes to zero
    EXPECT_STATE(prover_transcript, /*start*/ 0, /*written*/ 0, /*read*/ 0);
    Fr elt_a = 1377;
    prover_transcript.send_to_verifier("a", elt_a);
    EXPECT_STATE(prover_transcript, /*start*/ 0, /*written*/ 32, /*read*/ 0);
    Transcript verifier_transcript{ prover_transcript.export_proof() };
    // export resets read/write state and sets start in prep for next export
    EXPECT_STATE(prover_transcript, /*start*/ 32, /*written*/ 0, /*read*/ 0);
    // state initializes to zero
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 0, /*read*/ 0);
    Fr received_a = verifier_transcript.receive_from_prover<Fr>("a");
    // receiving is reading bytes input and writing them to an internal proof_data buffer
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 32, /*read*/ 32);
    EXPECT_EQ(received_a, elt_a);

    Fq elt_b = 773;
    prover_transcript.send_to_verifier("b", elt_b);
    EXPECT_STATE(prover_transcript, /*start*/ 32, /*written*/ 32, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 64, /*written*/ 0, /*read*/ 0);
    // load proof is not an action by a prover or verifeir, so it does not change read/write counts
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 32, /*read*/ 32);
    Fq received_b = verifier_transcript.receive_from_prover<Fq>("b");
    EXPECT_STATE(verifier_transcript, 0, 64, 64);
    EXPECT_EQ(received_b, elt_b);
}

} // namespace bb::honk_transcript_tests
