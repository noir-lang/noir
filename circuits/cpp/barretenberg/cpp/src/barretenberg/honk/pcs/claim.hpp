#pragma once

#include "barretenberg/polynomials/polynomial.hpp"

namespace proof_system::honk::pcs {
/**
 * @brief Opening pair (r,v) for some witness polynomial p(X) such that p(r) = v
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Params> class OpeningPair {
    using Fr = typename Params::Fr;

  public:
    Fr challenge;  // r
    Fr evaluation; // v = p(r)

    bool operator==(const OpeningPair& other) const = default;
};

/**
 * @brief Unverified claim (C,r,v) for some witness polynomial p(X) such that
 *  - C = Commit(p(X))
 *  - p(r) = v
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Params> class OpeningClaim {
    using CK = typename Params::CommitmentKey;
    using Commitment = typename Params::Commitment;
    using Fr = typename Params::Fr;

  public:
    // (challenge r, evaluation v = p(r))
    OpeningPair<Params> opening_pair;
    // commitment to univariate polynomial p(X)
    Commitment commitment;

    /**
     * @brief inefficiently check that the claim is correct by recomputing the commitment
     * and evaluating the polynomial in r.
     *
     * @param ck CommitmentKey used
     * @param polynomial the claimed witness polynomial p(X)
     * @return C = Commit(p(X)) && p(r) = v
     */
    bool verify(std::shared_ptr<CK> ck, const barretenberg::Polynomial<Fr>& polynomial) const
    {
        Fr real_eval = polynomial.evaluate(opening_pair.challenge);
        if (real_eval != opening_pair.evaluation) {
            return false;
        }
        // Note: real_commitment is a raw type, while commitment may be a linear combination.
        auto real_commitment = ck->commit(polynomial);
        return (real_commitment == commitment);
    };

    bool operator==(const OpeningClaim& other) const = default;
};

/**
 * @brief stores a claim of the form (C, v) for u=(u₀,…,uₘ₋₁)
 * where C is a univariate commitment to a polynomial
 *
 * f(X) = a₀ + a₁⋅X + … + aₙ₋₁⋅Xⁿ⁻¹
 *
 * and v is a multi-linear evaluation of f(X₀,…,Xₘ₋₁)
 * which has the same coefficients as f.
 * v = ∑ᵢ aᵢ⋅Lᵢ(u)
 *
 * If the evaluations is shift, we assume that a₀ = 0 and
 * take g(X) = f↺(X), so that
 * g(X) = a₁ + … + aₙ₋₁⋅Xⁿ⁻² = f(X)/X
 * The evaluation will be
 * v↺ = a₁⋅L₀(u) + … + aₙ₋₁⋅Lₙ₋₂(u)
 * The commitment C is [f].
 *
 * @tparam CommitmentKey
 */
template <typename Params> class MLEOpeningClaim {
    using Commitment = typename Params::Commitment;
    using Fr = typename Params::Fr;

  public:
    // commitment to a univariate polynomial
    // whose coefficients are the multi-linear evaluations
    // of C = [f]
    Commitment commitment;
    // v  = f(u) = ∑ᵢ aᵢ⋅Lᵢ(u)
    // v↺ = g(u) = a₁⋅L₀(u) + … + aₙ₋₁⋅Lₙ₋₂(u)
    Fr evaluation;
};
} // namespace proof_system::honk::pcs
