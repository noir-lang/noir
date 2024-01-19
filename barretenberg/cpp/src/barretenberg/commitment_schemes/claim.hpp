#pragma once

#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

namespace bb::honk::pcs {
/**
 * @brief Opening pair (r,v) for some witness polynomial p(X) such that p(r) = v
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Curve> class OpeningPair {
    using Fr = typename Curve::ScalarField;

  public:
    Fr challenge;  // r
    Fr evaluation; // v = p(r)

    bool operator==(const OpeningPair& other) const = default;
};

/**
 * @brief Polynomial p and an opening pair (r,v) such that p(r) = v
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Curve> class ProverOpeningClaim {
    using Fr = typename Curve::ScalarField;
    using Polynomial = bb::Polynomial<Fr>;

  public:
    Polynomial polynomial;           // p
    OpeningPair<Curve> opening_pair; // (challenge r, evaluation v = p(r))
};

/**
 * @brief Unverified claim (C,r,v) for some witness polynomial p(X) such that
 *  - C = Commit(p(X))
 *  - p(r) = v
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Curve> class OpeningClaim {
    using CK = CommitmentKey<Curve>;
    using Commitment = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

  public:
    // (challenge r, evaluation v = p(r))
    OpeningPair<Curve> opening_pair;
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
    bool verify(std::shared_ptr<CK> ck, const bb::Polynomial<Fr>& polynomial) const
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
} // namespace bb::honk::pcs
