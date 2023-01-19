#pragma once

#include "polynomials/polynomial.hpp"

namespace honk::pcs {
/**
 * @brief Unverified claim (C,r,v) for some witness polynomial p(X) such that
 *  - C = Commit(p(X))
 *  - p(r) = v
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Params> class OpeningClaim {
    using CK = typename Params::CK;
    using Commitment = typename Params::Commitment;
    using Fr = typename Params::Fr;

  public:
    // commitment to univariate polynomial p(X)
    Commitment commitment;
    // query r
    Fr opening_point;
    // evaluation v = p(r)
    Fr eval;

    /**
     * @brief inefficiently check that the claim is correct by recomputing the commitment
     * and evaluating the polynomial in r.
     *
     * @param ck CommitmentKey used
     * @param polynomial the claimed witness polynomial p(X)
     * @return C = Commit(p(X)) && p(r) = v
     */
    bool verify(CK* ck, const barretenberg::Polynomial<Fr>& polynomial) const
    {
        Fr real_eval = polynomial.evaluate(opening_point);
        if (real_eval != eval) {
            return false;
        }
        // Note: real_commitment is a raw type, while commitment may be a linear combination.
        auto real_commitment = ck->commit(polynomial);
        if (real_commitment != commitment) {
            // if (commitment != real_commitment) {
            return false;
        }
        return true;
    };

    bool operator==(const OpeningClaim& other) const = default;
};

/**
 * @brief A claim for multiple polynomials opened at various points.
 *
 * @details Each polynomial pⱼ(X) is opened at mⱼ points.
 * This gives a triple (Cⱼ, {xʲ₁, …, xʲₘⱼ}, {yʲ₁, …, yʲₘⱼ}) for each j,
 * where yʲᵢ = pⱼ(xʲᵢ).
 * We refer to 'queries' the set of opening points Ωⱼ = {xʲ₁, …, xʲₘⱼ},
 * and 'evals' is Yⱼ = {yʲ₁, …, yʲₘⱼ}.
 *
 * This structure groups all the triples by their common opening points sets.
 * A 'SubClaim' is indexed by k, and is defined by Ωₖ = {xᵏ₁, …, xᵏₘₖ},
 * and a vector of pairs [(Cⱼ, Yⱼ)]ⱼ for all j such that Ωₖ = Ωⱼ.
 * We refer to the latter as an 'Opening' of a 'SubClaim'.
 * For Shplonk, it is also necessary to include a vector 'all_queries' which we
 * define as Ω = ⋃ₖ Ωₖ.
 *
 * @invariant the following conditions are assumed to hold when BatchOpeningClaim
 * is consumed by functions:
 * - Each opening query set 'queries' Ωₖ is unique among all 'SubClaims'
 * - Each commitment Cⱼ belongs to a single 'SubClaim'
 * - The set 'all_queries' Ω is exactly the union of all 'queries' Ωₖ,
 *   and does not contain additional elements.
 *   The order is not important.
 * - All 'evals' vectors must follow the same order defined by Ωₖ.
 *
 * SubClaim is a pair (Ωₖ, [(Cⱼ, Yⱼ)]ⱼ) where
 * - 'queries' = Ωₖ = {xᵏ₁, …, xᵏₘₖ}
 * - 'openings' = [(Cⱼ, Yⱼ)]ⱼ is a vector of pairs
 *   - 'commitment' = Cⱼ = Commit(pⱼ(X)) for each j
 *   - 'evals' = Yⱼ = {yʲ₁, …, yʲₘₖ}, and yʲᵢ = pⱼ(xʲᵢ)
 *     and follows the same order as 'queries'
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Params> class MultiOpeningClaim {
    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;

  public:
    struct Opening {
        // Cⱼ = Commit(pⱼ(X)) for each j
        Commitment commitment;
        // Yⱼ = {yʲ₁, …, yʲₘₖ}
        std::vector<Fr> evals;

        bool operator==(const Opening& other) const = default;
    };
    // Ωₖ = {xᵏ₁, …, xᵏₘₖ}
    std::vector<Fr> queries;
    // [(Cⱼ, Yⱼ)]ⱼ
    std::vector<Opening> openings;

    bool operator==(const MultiOpeningClaim&) const = default;
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
template <typename Params> struct MLEOpeningClaim {
    using Commitment = typename Params::Commitment;
    using Fr = typename Params::Fr;

    MLEOpeningClaim(auto commitment, auto evaluation)
        : commitment(commitment)
        , evaluation(evaluation)
    {}

    // commitment to a univariate polynomial
    // whose coefficients are the multi-linear evaluations
    // of C = [f]
    Commitment commitment;
    // v  = f(u) = ∑ᵢ aᵢ⋅Lᵢ(u)
    // v↺ = g(u) = a₁⋅L₀(u) + … + aₙ₋₁⋅Lₙ₋₂(u)
    Fr evaluation;
};
} // namespace honk::pcs