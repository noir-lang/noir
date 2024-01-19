

#include "./Toy_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace bb;
using namespace bb::honk::sumcheck;

namespace bb::honk {
ToyVerifier::ToyVerifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

ToyVerifier::ToyVerifier(ToyVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

ToyVerifier& ToyVerifier::operator=(ToyVerifier&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

/**
 * @brief This function verifies an Toy Honk proof for given program settings.
 *
 */
bool ToyVerifier::verify_proof(const plonk::proof& proof)
{
    using Flavor = honk::flavor::ToyFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using Curve = Flavor::Curve;
    // using ZeroMorph = pcs::zeromorph::ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = Flavor::VerifierCommitments;
    using CommitmentLabels = Flavor::CommitmentLabels;

    RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof.proof_data);

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    commitments.toy_q_tuple_set =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_q_tuple_set);
    commitments.toy_set_1_column_1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_set_1_column_1);
    commitments.toy_set_1_column_2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_set_1_column_2);
    commitments.toy_set_2_column_1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_set_2_column_1);
    commitments.toy_set_2_column_2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_set_2_column_2);
    commitments.toy_xor_a = transcript->template receive_from_prover<Commitment>(commitment_labels.toy_xor_a);
    commitments.toy_xor_b = transcript->template receive_from_prover<Commitment>(commitment_labels.toy_xor_b);
    commitments.toy_xor_c = transcript->template receive_from_prover<Commitment>(commitment_labels.toy_xor_c);
    commitments.toy_table_xor_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_table_xor_a);
    commitments.toy_table_xor_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_table_xor_b);
    commitments.toy_table_xor_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_table_xor_c);
    commitments.toy_q_xor = transcript->template receive_from_prover<Commitment>(commitment_labels.toy_q_xor);
    commitments.toy_q_xor_table =
        transcript->template receive_from_prover<Commitment>(commitment_labels.toy_q_xor_table);
    commitments.two_column_perm =
        transcript->template receive_from_prover<Commitment>(commitment_labels.two_column_perm);
    commitments.lookup_xor = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_xor);
    commitments.lookup_xor_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_xor_counts);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);
    FF alpha = transcript->get_challenge("Sumcheck:alpha");
    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->get_challenge("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

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

} // namespace bb::honk
