

#include "./Fib_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/honk/proof_system/power_polynomial.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace barretenberg;
using namespace proof_system::honk::sumcheck;

namespace proof_system::honk {
FibVerifier::FibVerifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

FibVerifier::FibVerifier(FibVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

FibVerifier& FibVerifier::operator=(FibVerifier&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

/**
 * @brief This function verifies an Fib Honk proof for given program settings.
 *
 */
bool FibVerifier::verify_proof(const plonk::proof& proof)
{
    using Flavor = honk::flavor::FibFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using Curve = Flavor::Curve;
    // using ZeroMorph = pcs::zeromorph::ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = Flavor::VerifierCommitments;
    using CommitmentLabels = Flavor::CommitmentLabels;

    RelationParameters<FF> relation_parameters;

    transcript = BaseTranscript<FF>{ proof.proof_data };

    auto commitments = VerifierCommitments(key, transcript);
    auto commitment_labels = CommitmentLabels();

    const auto circuit_size = transcript.template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    commitments.Fibonacci_x = transcript.template receive_from_prover<Commitment>(commitment_labels.Fibonacci_x);
    commitments.Fibonacci_y = transcript.template receive_from_prover<Commitment>(commitment_labels.Fibonacci_y);

    // Execute Sumcheck Verifier
    auto sumcheck = SumcheckVerifier<Flavor>(circuit_size);

    auto alpha = transcript.get_challenge("alpha");
    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, transcript);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    // NOTE: temporarily disabled - facing integration issues
    // auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
    //                                         commitments.get_to_be_shifted(),
    //                                         claimed_evaluations.get_unshifted(),
    //                                         claimed_evaluations.get_shifted(),
    //                                         multivariate_challenge,
    //                                         transcript);

    // auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);
    // return sumcheck_verified.value() && verified;
    return sumcheck_verified.value();
}

} // namespace proof_system::honk
