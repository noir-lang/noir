#pragma once

#include "../claim.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

#include <memory>
#include <utility>

namespace proof_system::honk::pcs::kzg {

template <typename Params> class KZG {
    using CK = typename Params::CommitmentKey;
    using VK = typename Params::VerificationKey;
    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using GroupElement = typename Params::GroupElement;
    using Polynomial = barretenberg::Polynomial<Fr>;

    /**
     * @brief Computes the KZG commitment to an opening proof polynomial at a single evaluation point
     *
     * @param ck The commitment key which has a commit function, the srs and pippenger_runtime_state
     * @param opening_pair OpeningPair = {r, v = p(r)}
     * @param polynomial The witness whose opening proof needs to be computed
     * @param prover_transcript Prover transcript
     */
  public:
    static void compute_opening_proof(std::shared_ptr<CK> ck,
                                      const OpeningPair<Params>& opening_pair,
                                      const Polynomial& polynomial,
                                      ProverTranscript<Fr>& prover_trancript)
    {
        Polynomial quotient(polynomial);
        quotient[0] -= opening_pair.evaluation;
        // Computes the coefficients for the quotient polynomial q(X) = (p(X) - v) / (X - r) through an FFT
        quotient.factor_roots(opening_pair.challenge);
        auto quotient_commitment = ck->commit(quotient);
        // TODO(#479): for now we compute the KZG commitment directly to unify the KZG and IPA interfaces but in the
        // future we might need to adjust this to use the incoming alternative to work queue (i.e. variation of
        // pthreads) or even the work queue itself
        prover_trancript.send_to_verifier("KZG:W", quotient_commitment);
    };

    /**
     * @brief Computes the KZG verification for an opening claim of a single polynomial commitment
     * This reduction is non-interactive and always succeeds.
     *
     * @param vk is the verification key which has a pairing check function
     * @param claim OpeningClaim ({r, v}, C)
     * @return  e(P₀,[1]₁)e(P₁,[x]₂)≡ [1]ₜ where
     *      - P₀ = C − v⋅[1]₁ + r⋅[x]₁
     *      - P₁ = [Q(x)]₁
     */
    static bool verify(std::shared_ptr<VK> vk,
                       const OpeningClaim<Params>& claim,
                       VerifierTranscript<Fr>& verifier_transcript)
    {
        auto quotient_commitment = verifier_transcript.template receive_from_prover<Commitment>("KZG:W");
        auto lhs = claim.commitment - (GroupElement::one() * claim.opening_pair.evaluation) +
                   (quotient_commitment * claim.opening_pair.challenge);
        auto rhs = -quotient_commitment;

        return vk->pairing_check(lhs, rhs);
    };
};
} // namespace proof_system::honk::pcs::kzg
