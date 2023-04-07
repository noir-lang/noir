#pragma once

#include "../claim.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"

#include <memory>
#include <utility>

namespace proof_system::honk::pcs::kzg {
/**
 * @brief A transformed polynomial commitment opening claim of the form (P₀, P₁) ∈ 𝔾₁
 * which should satisfy e(P₀, [1]₂) ⋅ e(P₁, [x]₂)=1.
 *
 * @tparam Params CommitmentScheme parameters, where the verification key VK has a
 * `pairing_check` function.
 */
template <typename Params> class BilinearAccumulator {
    using VK = typename Params::VK;
    using Fr = typename Params::Fr;
    using CommitmentAffine = typename Params::C;
    using Commitment = typename Params::Commitment;

  public:
    /**
     * @brief Construct a new Bilinear Accumulator object given a claim (C,r,v) and proof π.
     *      - P₀ = C − v⋅[1]₁ + r⋅[x]₁
     *      - P₁ = −π
     * @param claim an OpeningClaim (C,r,v)
     * @param proof a Commitment π
     */
    BilinearAccumulator(const OpeningClaim<Params>& claim, const Commitment& proof)
        : lhs(claim.commitment - (Commitment::one() * claim.opening_pair.evaluation) +
              (proof * claim.opening_pair.challenge))
        , rhs(-proof)
    {}

    /**
     * @brief verifies the accumulator with a pairing check
     *
     * @param vk VerificationKey
     * @return e(P₀,[1]₁)e(P₁,[x]₂)≡ [1]ₜ
     */
    bool verify(std::shared_ptr<VK> vk) const { return vk->pairing_check(lhs, rhs); };

    bool operator==(const BilinearAccumulator& other) const = default;

    CommitmentAffine lhs, rhs;
};

template <typename Params> class UnivariateOpeningScheme {
    using CK = typename Params::CK;

    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using CommitmentAffine = typename Params::C;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    using Accumulator = BilinearAccumulator<Params>;

    /**
     * @brief Compute KZG opening proof polynomial
     *
     * @param opening_pair OpeningPair = {r, v = polynomial(r)}
     * @param polynomial the witness polynomial being opened
     * @return KZG quotient polynomial of the form (p(X) - v) / (X - r)
     */
    static Polynomial compute_opening_proof_polynomial(const OpeningPair<Params>& opening_pair,
                                                       const Polynomial& polynomial)
    {
        Polynomial quotient(polynomial);
        quotient[0] -= opening_pair.evaluation;
        quotient.factor_roots(opening_pair.challenge);

        return quotient;
    };

    /**
     * @brief Computes the accumulator for a single polynomial commitment opening claim
     * This reduction is non-interactive and always succeeds.
     *
     * @param claim OpeningClaim ({r, v}, C)
     * @param proof π, a commitment to Q(X) = ( P(X) - v )/( X - r)
     * @return Accumulator {C − v⋅[1]₁ + r⋅π, −π}
     */
    static Accumulator reduce_verify(const OpeningClaim<Params>& claim, VerifierTranscript<Fr>& verifier_transcript)
    {
        auto quotient_commitment = verifier_transcript.template receive_from_prover<CommitmentAffine>("KZG:W");
        return Accumulator(claim, quotient_commitment);
    };
};
} // namespace proof_system::honk::pcs::kzg
