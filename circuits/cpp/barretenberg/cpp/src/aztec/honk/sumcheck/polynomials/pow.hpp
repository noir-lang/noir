#pragma once

namespace honk::sumcheck {

/**
 * @brief Succinct representation of the `pow` polynomial that can be partially evaluated variable-by-variable.
 * pow(X) = ∏_{0≤l<d} ((1−Xₗ) + Xₗ⋅ζₗ)
 *
 * @details Let
 * - d be the number of variables
 * - l be the current Sumcheck round ( l ∈ {d-1, …, 0} )
 * - u_{d-1}, ..., u_{l+1} the challenges sent by the verifier in rounds d-1 to l+1.
 *
 * We define
 *
 * - ζ_{0}, ..., ζ_{d-1}, as ζ_{l} = ζ^{ 2^{d-l-1} }.
 *   When 0 ≤ i < 2^d is represented in bits [i_{0}, ..., i_{d-1}] where i_{0} is the MSB, we have
 *   ζ^{i} = ζ^{ ∑_{0≤l<d} i_{l}⋅2^{d-l-1} }
 *         =     ∏_{0≤l<d} ζ^{ i_{l}⋅2^{d-l-1} }
 *         =     ∏_{0≤l<d} ζ_{l}^{ i_{l} }
 *         =     ∏_{0≤l<d} ( (1-i_{l}) + i_{l}⋅ζ_{l} )
 *   Note that
 *   - ζ_{d-1} = ζ,
 *   - ζ_{l-1} = ζ_{l}^2,
 *   - ζ_{0}   = ζ^{ 2^{d-1} }
 *
 * - pow(X) = ∏_{0≤l<d} ((1−Xₗ) + Xₗ⋅ζₗ) is the multi-linear polynomial whose evaluation at the i-th index
 *   of the full hypercube, equals ζⁱ.
 *   We can also see it as the multi-linear extension of the vector (1, ζ, ζ², ...).
 *
 * - powˡᵢ( X_{l} ) = pow( i_{0}, ..., i_{l-1},
 *                         X_{l},
 *                         u_{l+1}, ..., u_{d-1} )
 *                  = ∏_{0≤k<l} ( (1-iₖ) + iₖ⋅ζₖ )
 *                             ⋅( (1−Xₗ) + Xₗ⋅ζₗ )
 *                    ∏_{l<k<d} ( (1-uₖ) + uₖ⋅ζₖ )
 *                  = ζ^{2^{d-l}}^{i} ⋅ ( (1−Xₗ) + Xₗ⋅ζₗ ) ⋅ cₗ
 *                  = ζ_{  l-1  }^{i} ⋅ ( (1−Xₗ) + Xₗ⋅ζₗ ) ⋅ cₗ,
 *
 *   This is the pow polynomial, partially evaluated in
 *     (X_{l+1}, ..., X_{d-1}) = (u_{l+1}, ..., u_{d-1}),
 *   at the index 0 ≤ i < 2ˡ of the dimension-l hypercube.
 *
 * - Sˡᵢ( Xₗ ) is the univariate of the full relation at edge pair i
 * i.e. it is the alpha-linear-combination of the relations evaluated in the i-th edge.
 * If our composed Sumcheck relation is a multi-variate polynomial P(X_{0}, ..., X_{d-1}),
 * Then Sˡᵢ( Xₗ ) = P( i_{0}, ..., i_{l-1}, X_{l}, u_{l+1}, ..., u_{d-1} ).
 * The l-th univariate would then be Sˡ( Xₗ ) = ∑_{0≤i<2ˡ} Sˡᵢ( Xₗ ) .
 *
 * We want to check that P(i)=0 for all i ∈ {0,1}ᵈ. So we use Sumcheck over the polynomial
 * P'(X) = pow(X)⋅P(X).
 * The claimed sum is 0 and is equal to ∑_{i ∈ {0,1}ᵈ} pow(i)⋅P(i) = ∑_{i ∈ {0,1}ᵈ} ζ^{i}⋅P(i)
 * If the Sumcheck passes, then with it must hold with high-probability that all P(i) are 0.
 *
 * The trivial implementation using P'(X) directly would increase the degree of our combined relation by 1.
 * Instead, we exploit the special structure of pow to preserve the same degree.
 *
 * In each round l, the prover should compute the univariate polynomial for the relation defined by P'(X)
 * S'ˡ(Xₗ) = ∑_{0≤i<2ˡ} powˡᵢ( Xₗ ) Sˡᵢ( Xₗ ) .
 *        = ∑_{0≤i<2ˡ} [ ζₗ₋₁ⁱ⋅( (1−Xₗ) + Xₗ⋅ζₗ )⋅cₗ ]⋅Sˡᵢ( Xₗ )
 *        = ( (1−Xₗ) + Xₗ⋅ζₗ ) ⋅ ∑_{0≤i<2ˡ} [ cₗ ⋅ ζₗ₋₁ⁱ ⋅ Sˡᵢ( Xₗ ) ]
 *
 * If we define Tˡ( Xₗ ) := ∑_{0≤i<2ˡ} [ cₗ ⋅ ζₗ₋₁ⁱ ⋅ Sˡᵢ( Xₗ ) ], then Tˡ has the same degree as the original Sˡ( Xₗ )
 * for the relation P(X) and is only slightly more expensive to compute than Sˡ( Xₗ ).
 * Moreover, given Tˡ( Xₗ ), the verifier can evaluate S'ˡ( uₗ ) by evaluating ( (1−uₗ) + uₗ⋅ζₗ )Tˡ( uₗ ).
 * When the verifier checks the claimed sum, the procedure is modified as follows
 *
 * Init:
 * - σ_{ d } <-- 0 // Claimed Sumcheck sum
 * - c_{d-1} <-- 1 // Partial evaluation constant, before any evaluation
 * - ζ_{d-1} <-- ζ // Initial power of ζ
 *
 * Round l:
 * - σ_{l+1} =?= S'ˡ(0) + S'ˡ(1) = Tˡ(0) + ζ_{l}⋅Tˡ(1)  // Check partial sum
 * - σ_{ l } <-- ( (1−u_{l}) + u_{l}⋅ζ_{l} )⋅Tʲ(u_{l})  // Compute next partial sum
 * - c_{ l } <-- ( (1−u_{l}) + u_{l}⋅ζ_{l} )⋅c_{l-1}    // Partially evaluate pow in u_{l}
 * - ζ_{l-1} <-- ζ_{l}^2                                // Get next power of ζ
 *
 * Final round l=0:
 * - σ_{1} =?= S'⁰(0) + S'⁰(1) = T⁰(0) + ζ_{0}⋅T⁰(1)    // Check partial sum
 * - σ_{0} <-- ( (1−u_{0}) + u_{0}⋅ζ_{0} )⋅T⁰(u_{0})    // Compute purported evaluation of P'(u)
 * - c_{0} <-- ∏_{0≤l<d} ( (1-u_{l}) + u_{l}⋅ζ_{l} )
 *           = pow(u_{0}, ..., u_{d-1})                 // Full evaluation of pow
 * - σ_{0} =?= c_{0}⋅P(u_{0}, ..., u_{d-1})             // Compare against real evaluation of P'(u)
 * @todo(Adrian): Eventually re-index polynomials with LSB first, and also rework the unicode symbols
 */
template <typename FF> struct PowUnivariate {
    // ζ_{l}, initialized as ζ_{d-1} = ζ
    // At round l, equals ζ^{ 2^{d-l-1} }
    FF zeta_pow;
    // ζ_{l-1}, initialized as ζ_{d-2} = ζ^2
    // Always equal to zeta_pow^2
    // At round l, equals ζ^{ 2^{d-l} }
    FF zeta_pow_sqr;
    // c_{l}, initialized as c_{d-1} = 1
    // c_{l} = ∏_{l<k<d} ( (1-u_{k}) + u_{k}⋅ζ_{k} )
    // At round 0, equals pow(u_{0}, ..., u_{d-1}).
    FF partial_evaluation_constant = FF::one();

    // Initialize with the random zeta
    PowUnivariate(FF zeta_pow)
        : zeta_pow(zeta_pow)
        , zeta_pow_sqr(zeta_pow.sqr())
    {}

    // Evaluate the monomial ((1−X_{l}) + X_{l}⋅ζ_{l}) in the challenge point X_{l}=u_{l}.
    FF univariate_eval(FF challenge) const { return (FF::one() + (challenge * (zeta_pow - FF::one()))); };

    /**
     * @brief Parially evaluate the polynomial in the new challenge, by updating the constant c_{l} -> c_{l-1}.
     * Also update (ζ_{l}, ζ_{l-1}) -> (ζ_{l-1}, ζ_{l-1}^2)
     *
     * @param challenge l-th verifier challenge u_{l}
     */
    void partially_evaluate(FF challenge)
    {
        FF current_univariate_eval = univariate_eval(challenge);
        zeta_pow = zeta_pow_sqr;
        zeta_pow_sqr.self_sqr();
        partial_evaluation_constant *= current_univariate_eval;
    }
};
} // namespace honk::sumcheck