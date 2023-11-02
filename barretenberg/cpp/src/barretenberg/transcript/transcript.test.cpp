#include "barretenberg/transcript/transcript.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/gtest.h>

namespace barretenberg::honk_transcript_tests {

using FF = barretenberg::fr;
using Transcript = proof_system::honk::BaseTranscript<FF>;

TEST(BaseTranscript, Basic)
{
    Transcript transcript;
    FF elt = 561;
    transcript.send_to_verifier("something", elt);
    auto received = transcript.template receive_from_prover<FF>("something");
    EXPECT_EQ(received, elt);
}
} // namespace barretenberg::honk_transcript_tests