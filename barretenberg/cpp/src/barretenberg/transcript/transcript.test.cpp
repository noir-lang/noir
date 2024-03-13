#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include <gtest/gtest.h>

using namespace bb;

using FF = bb::fr;
using Fr = bb::fr;
using Fq = bb::fq;
using Transcript = NativeTranscript;

/**
 * @brief Test sending, receiving, and exporting proofs
 *
 */
TEST(NativeTranscript, TwoProversTwoFields)
{
    const auto EXPECT_STATE = [](const Transcript& transcript, size_t start, size_t written, size_t read) {
        EXPECT_EQ(transcript.proof_start, static_cast<std::ptrdiff_t>(start));
        EXPECT_EQ(transcript.num_frs_written, written);
        EXPECT_EQ(transcript.num_frs_read, read);
    };

    Transcript prover_transcript;
    // state initializes to zero
    EXPECT_STATE(prover_transcript, /*start*/ 0, /*written*/ 0, /*read*/ 0);
    Fr elt_a = 1377;
    prover_transcript.send_to_verifier("a", elt_a);
    EXPECT_STATE(prover_transcript, /*start*/ 0, /*written*/ 1, /*read*/ 0);
    Transcript verifier_transcript{ prover_transcript.export_proof() };
    // export resets read/write state and sets start in prep for next export
    EXPECT_STATE(prover_transcript, /*start*/ 1, /*written*/ 0, /*read*/ 0);
    // state initializes to zero
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 0, /*read*/ 0);
    Fr received_a = verifier_transcript.receive_from_prover<Fr>("a");
    // receiving is reading frs input and writing them to an internal proof_data buffer
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 1, /*read*/ 1);
    EXPECT_EQ(received_a, elt_a);

    // send grumpkin::fr
    Fq elt_b = 773;
    prover_transcript.send_to_verifier("b", elt_b);
    EXPECT_STATE(prover_transcript, /*start*/ 1, /*written*/ 2, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 3, /*written*/ 0, /*read*/ 0);
    // load proof is not an action by a prover or verifier, so it does not change read/write counts
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 1, /*read*/ 1);
    Fq received_b = verifier_transcript.receive_from_prover<Fq>("b");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 3, /*read*/ 3);
    EXPECT_EQ(received_b, elt_b);

    // send uint32_t
    uint32_t elt_c = 43;
    prover_transcript.send_to_verifier("c", elt_c);
    EXPECT_STATE(prover_transcript, /*start*/ 3, /*written*/ 1, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 4, /*written*/ 0, /*read*/ 0);
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 3, /*read*/ 3);
    auto received_c = verifier_transcript.receive_from_prover<uint32_t>("c");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 4, /*read*/ 4);
    EXPECT_EQ(received_c, elt_c);

    // send curve::BN254::AffineElement
    curve::BN254::AffineElement elt_d = bb::g1::affine_one;
    prover_transcript.send_to_verifier("d", elt_d);
    EXPECT_STATE(prover_transcript, /*start*/ 4, /*written*/ 4, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 8, /*written*/ 0, /*read*/ 0);
    auto received_d = verifier_transcript.receive_from_prover<curve::BN254::AffineElement>("d");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 8, /*read*/ 8);
    EXPECT_EQ(received_d, elt_d);

    // send std::array<bb::fr, 4>
    std::array<bb::fr, 5> elt_e = { 1, 2, 3, 4, 5 };
    prover_transcript.send_to_verifier("e", elt_e);
    EXPECT_STATE(prover_transcript, /*start*/ 8, /*written*/ 5, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 13, /*written*/ 0, /*read*/ 0);
    auto received_e = verifier_transcript.receive_from_prover<std::array<bb::fr, 5>>("e");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 13, /*read*/ 13);
    EXPECT_EQ(received_e, elt_e);

    // send std::array<grumpkin::fr>
    std::array<grumpkin::fr, 7> elt_f = { 9, 12515, 1231, 745, 124, 6231, 957 };
    prover_transcript.send_to_verifier("f", elt_f);
    EXPECT_STATE(prover_transcript, /*start*/ 13, /*written*/ 14, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 27, /*written*/ 0, /*read*/ 0);
    auto received_f = verifier_transcript.receive_from_prover<std::array<grumpkin::fr, 7>>("f");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 27, /*read*/ 27);
    EXPECT_EQ(received_f, elt_f);

    // send Univariate<bb::fr>
    bb::Univariate<bb::fr, 4> elt_g{ std::array<bb::fr, 4>({ 5, 6, 7, 8 }) };
    prover_transcript.send_to_verifier("g", elt_g);
    EXPECT_STATE(prover_transcript, /*start*/ 27, /*written*/ 4, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 31, /*written*/ 0, /*read*/ 0);
    auto received_g = verifier_transcript.receive_from_prover<bb::Univariate<bb::fr, 4>>("g");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 31, /*read*/ 31);
    EXPECT_EQ(received_g, elt_g);

    // send Univariate<grumpkin::fr>
    bb::Univariate<grumpkin::fr, 3> elt_h{ std::array<grumpkin::fr, 3>({ 9, 10, 11 }) };
    prover_transcript.send_to_verifier("h", elt_h);
    EXPECT_STATE(prover_transcript, /*start*/ 31, /*written*/ 6, /*read*/ 0);
    verifier_transcript.load_proof(prover_transcript.export_proof());
    EXPECT_STATE(prover_transcript, /*start*/ 37, /*written*/ 0, /*read*/ 0);
    auto received_h = verifier_transcript.receive_from_prover<bb::Univariate<grumpkin::fr, 3>>("h");
    EXPECT_STATE(verifier_transcript, /*start*/ 0, /*written*/ 37, /*read*/ 37);
    EXPECT_EQ(received_h, elt_h);
}
