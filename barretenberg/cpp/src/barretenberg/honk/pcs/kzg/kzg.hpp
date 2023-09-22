#pragma once

#include "../claim.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/pcs/verification_key.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

#include <memory>
#include <utility>

namespace proof_system::honk::pcs::kzg {

template <typename Curve> class KZG {
    using CK = CommitmentKey<Curve>;
    using VK = VerifierCommitmentKey<Curve>;
    using Fr = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;
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
                                      const OpeningPair<Curve>& opening_pair,
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
     *
     * @param vk is the verification key which has a pairing check function
     * @param claim OpeningClaim ({r, v}, C)
     * @return  e(P₀,[1]₁)e(P₁,[x]₂)≡ [1]ₜ where
     *      - P₀ = C − v⋅[1]₁ + r⋅[x]₁
     *      - P₁ = [Q(x)]₁
     */
    static bool verify(std::shared_ptr<VK> vk,
                       const OpeningClaim<Curve>& claim,
                       VerifierTranscript<Fr>& verifier_transcript)
    {
        auto quotient_commitment = verifier_transcript.template receive_from_prover<Commitment>("KZG:W");
        auto lhs = claim.commitment - (GroupElement::one() * claim.opening_pair.evaluation) +
                   (quotient_commitment * claim.opening_pair.challenge);
        auto rhs = -quotient_commitment;

        return vk->pairing_check(lhs, rhs);
    };

    /**
     * @brief Computes the input points for the pairing check needed to verify a KZG opening claim of a single
     * polynomial commitment. This reduction is non-interactive and always succeeds.
     * @details This is used in the recursive setting where we want to "aggregate" proofs, not verify them.
     *
     * @param claim OpeningClaim ({r, v}, C)
     * @return  {P₀, P₁} where
     *      - P₀ = C − v⋅[1]₁ + r⋅[W(x)]₁
     *      - P₁ = [W(x)]₁
     */
    static std::array<GroupElement, 2> compute_pairing_points(const OpeningClaim<Curve>& claim,
                                                              auto& verifier_transcript)
    {
        auto quotient_commitment = verifier_transcript.template receive_from_prover<Commitment>("KZG:W");

        GroupElement P_0;
        // Note: In the recursive setting, we only add the contribution if it is not the point at infinity (i.e. if the
        // evaluation is not equal to zero).
        if constexpr (Curve::is_stdlib_type) {
            auto builder = verifier_transcript.builder;
            auto one = Fr(builder, 1);
            std::vector<GroupElement> commitments = { claim.commitment, quotient_commitment };
            std::vector<Fr> scalars = { one, claim.opening_pair.challenge };
            P_0 = GroupElement::batch_mul(commitments, scalars);
            // Note: This implementation assumes the evaluation is zero (as is the case for shplonk).
            ASSERT(claim.opening_pair.evaluation.get_value() == 0);
        } else {
            P_0 = claim.commitment;
            P_0 += quotient_commitment * claim.opening_pair.challenge;
            P_0 -= GroupElement::one() * claim.opening_pair.evaluation;
        }

        auto P_1 = -quotient_commitment;

        return { P_0, P_1 };
    };
};
} // namespace proof_system::honk::pcs::kzg
