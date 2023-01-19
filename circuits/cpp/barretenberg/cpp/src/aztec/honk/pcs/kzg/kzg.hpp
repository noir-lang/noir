#pragma once

#include "../claim.hpp"
#include "polynomials/polynomial.hpp"

#include <memory>
#include <utility>

namespace honk::pcs::kzg {
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
        : lhs(claim.commitment - (Commitment::one() * claim.eval) + (proof * claim.opening_point))
        , rhs(-proof)
    {}

    /**
     * @brief verifies the accumulator with a pairing check
     *
     * @param vk VerificationKey
     * @return e(P₀,[1]₁)e(P₁,[x]₂)≡ [1]ₜ
     */
    bool verify(VK* vk) const { return vk->pairing_check(lhs, rhs); };

    bool operator==(const BilinearAccumulator& other) const = default;

    Commitment lhs, rhs;
};

template <typename Params> class UnivariateOpeningScheme {
    using CK = typename Params::CK;

    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    using Accumulator = BilinearAccumulator<Params>;
    using Proof = Commitment;

    struct Output {
        Accumulator accumulator;
        Proof proof;
    };

    /**
     * @brief Compute an accumulator for a single polynomial commitment opening claim
     *
     * @param ck CommitmentKey
     * @param claim OpeningClaim = (C,r,v)
     * @param polynomial the witness polynomial for C
     * @return Output{Accumulator, Proof}
     */
    static Output reduce_prove(std::shared_ptr<CK> ck, const OpeningClaim<Params>& claim, const Polynomial& polynomial)
    {
        Polynomial quotient(polynomial);
        quotient[0] -= claim.eval;
        quotient.factor_roots(claim.opening_point);
        Proof proof = ck->commit(quotient);

        return Output{ Accumulator(claim, proof), proof };
    };

    /**
     * @brief Computes the accumulator for a single polynomial commitment opening claim
     * This reduction is non-interactive and always succeeds.
     *
     * @param claim OpeningClaim (C,r,v)
     * @param proof a commitment to Q(X) = ( P(X) - v )/( X - r)
     * @return Accumulator
     */
    static Accumulator reduce_verify(const OpeningClaim<Params>& claim, const Proof& proof)
    {
        return Accumulator(claim, proof);
    };
};
} // namespace honk::pcs::kzg