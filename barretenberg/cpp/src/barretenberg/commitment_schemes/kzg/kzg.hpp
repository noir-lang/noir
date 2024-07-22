#pragma once

#include "../claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/verification_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript.hpp"

#include <memory>
#include <utility>

namespace bb {

template <typename Curve_> class KZG {
  public:
    using Curve = Curve_;
    using CK = CommitmentKey<Curve>;
    using VK = VerifierCommitmentKey<Curve>;
    using Fr = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;
    using Polynomial = bb::Polynomial<Fr>;
    using VerifierAccumulator = std::array<GroupElement, 2>;

    /**
     * @brief Computes the KZG commitment to an opening proof polynomial at a single evaluation point
     *
     * @param ck The commitment key which has a commit function, the srs and pippenger_runtime_state
     * @param opening_claim {p, (r, v = p(r))} where p is the witness polynomial whose opening proof needs to be
     * computed
     * @param prover_transcript Prover transcript
     */
    template <typename Transcript>
    static void compute_opening_proof(std::shared_ptr<CK> ck,
                                      const ProverOpeningClaim<Curve>& opening_claim,
                                      const std::shared_ptr<Transcript>& prover_trancript)
    {
        Polynomial quotient = opening_claim.polynomial;
        OpeningPair<Curve> pair = opening_claim.opening_pair;
        quotient[0] -= pair.evaluation;
        // Computes the coefficients for the quotient polynomial q(X) = (p(X) - v) / (X - r) through an FFT
        quotient.factor_roots(pair.challenge);
        auto quotient_commitment = ck->commit(quotient);
        // TODO(#479): for now we compute the KZG commitment directly to unify the KZG and IPA interfaces but in the
        // future we might need to adjust this to use the incoming alternative to work queue (i.e. variation of
        // pthreads) or even the work queue itself
        prover_trancript->send_to_verifier("KZG:W", quotient_commitment);
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
    template <typename Transcript>
    static VerifierAccumulator reduce_verify(const OpeningClaim<Curve>& claim,
                                             const std::shared_ptr<Transcript>& verifier_transcript)
    {
        auto quotient_commitment = verifier_transcript->template receive_from_prover<Commitment>("KZG:W");

        // Note: The pairing check can be expressed naturally as
        // e(C - v * [1]_1, [1]_2) = e([W]_1, [X - r]_2) where C =[p(X)]_1. This can be rearranged (e.g. see the plonk
        // paper) as e(C + r*[W]_1 - v*[1]_1, [1]_2) * e(-[W]_1, [X]_2) = 1, or e(P_0, [1]_2) * e(P_1, [X]_2) = 1
        GroupElement P_0;
        if constexpr (Curve::is_stdlib_type) {
            // Express operation as a batch_mul in order to use Goblinization if available
            auto builder = quotient_commitment.get_context();
            auto one = Fr(builder, 1);
            std::vector<GroupElement> commitments = { claim.commitment,
                                                      quotient_commitment,
                                                      GroupElement::one(builder) };
            std::vector<Fr> scalars = { one, claim.opening_pair.challenge, -claim.opening_pair.evaluation };
            P_0 = GroupElement::batch_mul(commitments, scalars);

        } else {
            P_0 = claim.commitment;
            P_0 += quotient_commitment * claim.opening_pair.challenge;
            P_0 -= GroupElement::one() * claim.opening_pair.evaluation;
        }

        auto P_1 = -quotient_commitment;
        return { P_0, P_1 };
    };
};
} // namespace bb
