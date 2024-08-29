#include "barretenberg/vm/avm/recursion/avm_recursive_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

template <typename Flavor>
AvmRecursiveVerifier_<Flavor>::AvmRecursiveVerifier_(
    Builder* builder, const std::shared_ptr<NativeVerificationKey>& native_verification_key)
    : key(std::make_shared<VerificationKey>(builder, native_verification_key))
    , builder(builder)
{}

template <typename Flavor>
AvmRecursiveVerifier_<Flavor>::AvmRecursiveVerifier_(Builder* builder, const std::shared_ptr<VerificationKey>& vkey)
    : key(vkey)
    , builder(builder)
{}

// TODO(#991): (see https://github.com/AztecProtocol/barretenberg/issues/991)
template <typename Flavor>
std::array<typename Flavor::GroupElement, 2> AvmRecursiveVerifier_<Flavor>::verify_proof(const HonkProof& proof)
{
    using Curve = typename Flavor::Curve;
    using Zeromorph = ZeroMorphVerifier_<Curve>;
    using PCS = typename Flavor::PCS;
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using RelationParams = ::bb::RelationParameters<typename Flavor::FF>;
    using Transcript = typename Flavor::Transcript;

    StdlibProof<Builder> stdlib_proof = bb::convert_proof_to_witness(builder, proof);
    transcript = std::make_shared<Transcript>(stdlib_proof);

    RelationParams relation_parameters;
    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<FF>("circuit_size");
    if (static_cast<uint32_t>(circuit_size.get_value()) != key->circuit_size) {
        throw_or_abort("AvmRecursiveVerifier::verify_proof: proof circuit size does not match verification key!");
    }

    // Get commitments to VM wires
    for (auto [comm, label] : zip_view(commitments.get_wires(), commitment_labels.get_wires())) {
        comm = transcript->template receive_from_prover<Commitment>(label);
    }

    auto [beta, gamma] = transcript->template get_challenges<FF>("beta", "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;

    // Get commitments to inverses
    for (auto [label, commitment] : zip_view(commitment_labels.get_derived(), commitments.get_derived())) {
        commitment = transcript->template receive_from_prover<Commitment>(label);
    }

    // unconstrained
    const size_t log_circuit_size = numeric::get_msb(static_cast<uint32_t>(circuit_size.get_value()));
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);

    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");

    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    // No need to constrain that sumcheck_verified is true as this is guaranteed by the implementation of
    // when called over a "circuit field" types.
    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    vinfo("verified sumcheck: ", (sumcheck_verified.has_value() && sumcheck_verified.value()));

    auto opening_claim = Zeromorph::verify(circuit_size,
                                           commitments.get_unshifted(),
                                           commitments.get_to_be_shifted(),
                                           claimed_evaluations.get_unshifted(),
                                           claimed_evaluations.get_shifted(),
                                           multivariate_challenge,
                                           Commitment::one(builder),
                                           transcript);

    auto pairing_points = PCS::reduce_verify(opening_claim, transcript);

    return pairing_points;

    // Probably we will have to return an aggregation object (see ultra_recursive_verifier.cpp) once we interface
    // with noir for public_kernel integration. Follow, the same recipe as in ultra_recursive_verifier.cpp in this case.
}

template class AvmRecursiveVerifier_<AvmRecursiveFlavor_<UltraCircuitBuilder>>;
} // namespace bb