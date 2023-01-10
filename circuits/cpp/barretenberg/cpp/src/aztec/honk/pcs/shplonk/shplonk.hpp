#pragma once

#include "../claim.hpp"
#include "polynomials/polynomial.hpp"

/**
 * @brief Reduces multiple claims about commitments opened at multiple points
 *  into a single claim for a single polynomial opened at a single point.
 *
 * We use the following terminology:
 * - Bₖ(X) is a random linear combination of all polynomials opened at Ωₖ
 *   we refer to it a 'merged_polynomial'.
 * - Tₖ(X) is the polynomial that interpolates Bₖ(X) over Ωₖ,
 * - zₖ(X) is the product of all (X-x), for x ∈ Ωₖ
 * - ẑₖ(X) = 1/zₖ(X)
 *
 * The challenges are ρ (batching) and r (random evaluation).
 *
 */
namespace honk::pcs::shplonk {

/**
 * @brief Single commitment to  Q(X) = ∑ₖ ( Bₖ(X) − Tₖ(X) ) / zₖ(X)
 *
 */
template <typename Params> using Proof = typename Params::Commitment;

/**
 * @brief Single opening claim ([G], r, 0) so that G(r) = 0
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> using OutputClaim = OpeningClaim<Params>;

/**
 * @brief Polynomial G(X) = Q(X) - ∑ₖ ẑₖ(r)⋅( Bₖ(X) − Tₖ(z) )
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> using OutputWitness = barretenberg::Polynomial<typename Params::Fr>;

/**
 * @brief Prover output (claim=([G], r, 0), witness = G(X), proof = [Q])
 * that can be passed on to a univariate opening protocol.
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> struct ProverOutput {
    OutputClaim<Params> claim;
    OutputWitness<Params> witness;
    Proof<Params> proof;
};

} // namespace honk::pcs::shplonk