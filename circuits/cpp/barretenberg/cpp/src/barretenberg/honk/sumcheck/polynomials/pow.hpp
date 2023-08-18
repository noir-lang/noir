#pragma once

namespace proof_system::honk::sumcheck {

/**
 * @brief Succinct representation of the `pow` polynomial that can be partially evaluated variable-by-variable.
 * pow(X) = ∏_{0≤l<d} ((1−X_l) + X_l⋅ζ_l)
 *
 * @details Let
 * - d be the number of variables
 * - l be the current Sumcheck round ( l ∈ {0, …, d-1} )
 * - u_{0}, ..., u_{l-1} the challenges sent by the verifier in rounds 0 to l-1.
 *
 * We define
 *
 * - ζ_{0}, ..., ζ_{d-1}, as ζ_{l} = ζ^{ 2^l }.
 *   When 0 ≤ i < 2^d is represented in bits [i_{0}, ..., i_{d-1}] where i_{0} is the MSB, we have
 *   ζ^{i} = ζ^{ ∑_{0≤l<d} i_l ⋅ 2^l }
 *         =     ∏_{0≤l<d} ζ^{ i_l ⋅ 2^l }
 *         =     ∏_{0≤l<d} ζ_l^{ i_l }
 *         =     ∏_{0≤l<d} { ( 1-i_l ) + i_l ⋅ ζ_l } // As i_{l} \in \{0, 1\}
 *   Note that
 *   - ζ_{0} = ζ,
 *   - ζ_{l+1} = ζ_{l}^2,
 *   - ζ_{d-1}   = ζ^{ 2^{d-1} }
 *
 * - pow(X) = ∏_{0≤l<d} ((1−X_l) + X_l⋅ζ_l) is the multi-linear polynomial whose evaluation at the i-th index
 *   of the full hypercube, equals ζⁱ.
 *   We can also see it as the multi-linear extension of the vector (1, ζ, ζ², ...).
 *
 * - At round l, we iterate over all remaining vertices (i_{l+1}, ..., i_{d-1}) ∈ {0,1}^{d-l-1}.
 *   Let i = ∑_{l<k<d} i_k ⋅ 2^{k-(l+1)} be the index of the current edge over which we are evaluating the relation.
 *   We define the edge univariate for the pow polynomial as powˡᵢ( X_l ) and it can be represented as:
 *
 *   powˡᵢ( X_{l} ) = pow( u_{0}, ..., u_{l-1},
 *                         X_{l},
 *                         i_{l+1}, ..., i_{d-1})
 *                  = ∏_{0≤k<l} ( (1-u_k) + u_k⋅ζ_k )
 *                             ⋅( (1−X_l) + X_l⋅ζ_l )
 *                    ∏_{l<k<d} ( (1-i_k) + i_k⋅ζ_k )
 *                  = c_l ⋅ ( (1−X_l) + X_l⋅ζ^{2^l} ) ⋅ ∏_{l<k<d} ( (1-i_k) + i_k⋅ζ^{2^k} )
 *                  = c_l ⋅ ( (1−X_l) + X_l⋅ζ^{2^l} ) ⋅ ζ^{ ∑_{l<k<d} i_k ⋅ 2^k }
 *                  = c_l ⋅ ( (1−X_l) + X_l⋅ζ^{2^l} ) ⋅(ζ^2^{l+1})^{ ∑_{l<k<d} i_k ⋅ 2^{k-(l+1)} }
 *                  = c_l ⋅ ( (1−X_l) + X_l⋅ζ^{2^l} ) ⋅ζ_{l+1}^{i}
 *
 *   This is the pow polynomial, partially evaluated in
 *     (X_{0}, ..., X_{l-1}) = (u_{0}, ..., u_{l-1}),
 *   at the index 0 ≤ i < 2^{d-l-1} of the dimension-{d-l-1} hypercube.
 *
 * - Sˡᵢ( X_l ) is the univariate of the full relation at edge pair i
 * i.e. it is the alpha-linear-combination of the relations evaluated in the edge at index i.
 * If our composed Sumcheck relation is a multi-variate polynomial P(X_{0}, ..., X_{d-1}),
 * Then Sˡᵢ( X_l ) = P( u_{0}, ..., u_{l-1}, X_{l}, i_{l+1}, ..., i_{d-1} ).
 * The l-th univariate would then be Sˡ( X_l ) = ∑_{ 0 ≤ i < 2^{d-l-1} }  Sˡᵢ( X_l ) .
 *
 * We want to check that P(i)=0 for all i ∈ {0,1}^d. So we use Sumcheck over the polynomial
 * P'(X) = pow(X)⋅P(X).
 * The claimed sum is 0 and is equal to ∑_{i ∈ {0,1}^d} pow(i)⋅P(i) = ∑_{i ∈ {0,1}^d} ζ^{i}⋅P(i)
 * If the Sumcheck passes, then with it must hold with high-probability that all P(i) are 0.
 *
 * The trivial implementation using P'(X) directly would increase the degree of our combined relation by 1.
 * Instead, we exploit the special structure of pow to preserve the same degree.
 *
 * In each round l, the prover should compute the univariate polynomial for the relation defined by P'(X)
 * S'ˡ(X_l) = ∑_{ 0 ≤ i < 2^{d-l-1} } powˡᵢ( X_l ) Sˡᵢ( X_l ) .
 *        = ∑_{ 0 ≤ i < 2^{d-l-1} } [ ζ_{l+1}ⁱ⋅( (1−X_l) + X_l⋅ζ_l )⋅c_l ]⋅Sˡᵢ( X_l )
 *        = ( (1−X_l) + X_l⋅ζ_l ) ⋅ ∑_{ 0 ≤ i < 2^{d-l-1} } [ c_l ⋅ ζ_{l+1}ⁱ ⋅ Sˡᵢ( X_l ) ]
 *        = ( (1−X_l) + X_l⋅ζ_l ) ⋅ ∑_{ 0 ≤ i < 2^{d-l-1} } [ c_l ⋅ ζ_{l+1}ⁱ ⋅ Sˡᵢ( X_l ) ]
 *
 * If we define Tˡ( X_l ) := ∑_{0≤i<2ˡ} [ c_l ⋅ ζ_{l+1}ⁱ ⋅ Sˡᵢ( X_l ) ], then Tˡ has the same degree as the original Sˡ(
 * X_l ) for the relation P(X) and is only slightly more expensive to compute than Sˡ( X_l ). Moreover, given Tˡ( X_l ),
 * the verifier can evaluate S'ˡ( u_l ) by evaluating ( (1−u_l) + u_l⋅ζ_l )Tˡ( u_l ). When the verifier checks the
 * claimed sum, the procedure is modified as follows
 *
 * Init:
 * - σ_{ 0 } <-- 0 // Claimed Sumcheck sum
 * - c_{ 0 } <-- 1 // Partial evaluation constant, before any evaluation
 * - ζ_{ 0 } <-- ζ // Initial power of ζ
 *
 * Round 0≤l<d-1:
 * - σ_{ l } =?= S'ˡ(0) + S'ˡ(1) = Tˡ(0) + ζ_{l}⋅Tˡ(1)  // Check partial sum
 * - σ_{l+1} <-- ( (1−u_{l}) + u_{l}⋅ζ_{l} )⋅Tʲ(u_{l})  // Compute next partial sum
 * - c_{l+1} <-- ( (1−u_{l}) + u_{l}⋅ζ_{l} )⋅c_{l}      // Partially evaluate pow in u_{l}
 * - ζ_{l+1} <-- ζ_{l}^2                                // Get next power of ζ
 *
 * Final round l=d-1:
 * - σ_{d-1} =?= S'ᵈ⁻¹(0) + S'ᵈ⁻¹(1) = Tᵈ⁻¹(0) + ζ_{d-1}⋅Tᵈ⁻¹(1)    // Check partial sum
 * - σ_{ d } <-- ( (1−u_{d-1}) + u_{d-1}⋅ζ_{0} )⋅Tᵈ⁻¹(u_{d-1})      // Compute purported evaluation of P'(u)
 * - c_{ d } <-- ∏_{0≤l<d} ( (1-u_{l}) + u_{l}⋅ζ_{l} )
 *             = pow(u_{0}, ..., u_{d-1})                           // Full evaluation of pow
 * - σ_{ d } =?= c_{d}⋅P(u_{0}, ..., u_{d-1})                       // Compare against real evaluation of P'(u)
 */
template <typename FF> struct PowUnivariate {
    // ζ_{l}, initialized as ζ_{0} = ζ
    // At round l, equals ζ^{ 2^l }
    FF zeta_pow;
    // ζ_{l+1}, initialized as ζ_{1} = ζ^2
    // Always equal to zeta_pow^2
    // At round l, equals ζ^{ 2^{l+1} }
    FF zeta_pow_sqr;
    // c_{l}, initialized as c_{0} = 1
    // c_{l} = ∏_{0 ≤ k < l-1} ( (1-u_{k}) + u_{k}⋅ζ_{k} )
    // At round d-1, equals pow(u_{0}, ..., u_{d-1}).
    FF partial_evaluation_constant = FF(1);

    // Initialize with the random zeta
    explicit PowUnivariate(FF zeta_pow)
        : zeta_pow(zeta_pow)
        , zeta_pow_sqr(zeta_pow.sqr())
    {}

    // Evaluate the monomial ((1−X_{l}) + X_{l}⋅ζ_{l}) in the challenge point X_{l}=u_{l}.
    FF univariate_eval(FF challenge) const { return (FF(1) + (challenge * (zeta_pow - FF(1)))); };

    /**
     * @brief Parially evaluate the polynomial in the new challenge, by updating the constant c_{l} -> c_{l+1}.
     * Also update (ζ_{l}, ζ_{l+1}) -> (ζ_{l+1}, ζ_{l+1}^2)
     *
     * @param challenge l-th verifier challenge u_{l}
     */
    void partially_evaluate(FF challenge)
    {
        FF current_univariate_eval = univariate_eval(challenge);
        zeta_pow = zeta_pow_sqr;
        // TODO(luke): for native FF, this could be self_sqr()
        zeta_pow_sqr = zeta_pow_sqr.sqr();

        partial_evaluation_constant *= current_univariate_eval;
    }
};
} // namespace proof_system::honk::sumcheck