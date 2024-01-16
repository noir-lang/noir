#pragma once
#include "barretenberg/common/thread.hpp"
#include "barretenberg/common/thread_utils.hpp"
#include <cstddef>
#include <vector>
namespace bb {

/**
 * @brief Succinct representation of the `pow` polynomial that can be partially evaluated variable-by-variable.
 * pow_{\vec{β}}(X_0,X_1,..,X_{d-1}) = \prod_{0≤l<d} ((1−X_l) + X_l⋅β_l) for a vector {β_0,...,β_{d-1}} used in
sumcheck.
 *
 * @details Let
 * - d be the number of variables
 * - l be the current Sumcheck round ( l ∈ {0, …, d-1} )
 * - u_0, ..., u_{l-1} the challenges sent by the verifier in rounds 0 to l-1.
 *
 * - pow_\vec{β}(X) = ∏_{0≤l<d} ((1−X_l) + X_l⋅β_l) is the multilinear polynomial whose evaluation at the i-th index
 *   of the full hypercube equals pow_\vec{β}(i) = ∏_{j ∈ bin(i), j = 1} β_j. More explicitly, the product contains the
elements of \vec{β} whose
 *  indexes j represent bits set to 1 in the binary representation of i.
 *
 *  We can also see it as the multi-linear extension of the vector \vec{β}.
 *
 * - At round l, we iterate over a boolean hypercube of dimension (d-l).
 *   Let i = ∑_{l<k<d} i_k ⋅ 2^{k-(l+1)} be the index of the current edge over which we are evaluating the relation.
 *   We define the edge univariate for the pow polynomial as powˡᵢ( X_l ) and it can be represented as:
 *
 *   powˡᵢ( X_{l} ) = pow( u_{0}, ..., u_{l-1},
 *                         X_{l},
 *                         i_{l+1}, ..., i_{d-1})
 *                  = ∏_{0≤k<l} ( (1-u_k) + u_k⋅β_k )
 *                             ⋅( (1−X_l) + X_l⋅β_l )
 *                    ∏_{l<k<d} ( (1-i_k) + i_k⋅β_k )
 *
 * Product ∏_{0≤k<l} ( (1-u_k) + u_k⋅β_k ) gets updated at each round of sumcheck and represents the
 * partial_evaluation_result in the implementation. This is the pow polynomial, partially evaluated in the first l-1
 * variables as (X_{0}, ..., X_{l-1}) = (u_{0}, ...,u_{l-1}).
 *
 * As we iterate over the other points in the boolean hypercube (i_{l+1}, ..., i_{d-1}) ∈ {0,1}^{d-l-1},
 * the subproducts
 * ∏_{l<k<d} ( (1-i_k) + i_k⋅β_k ) represent the terms of pow(\vec{β}) that do not contain β_0,...,β_l. These appear in
 * the set {pow_\vec{β}(i)| i =0,..,2^{d}-1} at indices 2^{l+1} * p where p ≥ 1 and 2^{l+1} * p < 2^d
 *
 *
 * - Sˡᵢ( X_l ) is the univariate of the full relation at edge pair i
 * i.e. it is the alpha-linear-combination of the relations evaluated in the edge at index i.
 * If our composed Sumcheck relation is a multi-variate polynomial P(X_{0}, ..., X_{d-1}),
 * Then Sˡᵢ( X_l ) = P( u_{0}, ..., u_{l-1}, X_{l}, i_{l+1}, ..., i_{d-1} ).
 * The l-th univariate would then be Sˡ( X_l ) = ∑_{ 0 ≤ i < 2^{d-l-1} }  Sˡᵢ( X_l ) .
 *
 * We want to check that P(i)=0 for all i ∈ {0,1}^d. So we use Sumcheck over the polynomial
 * P'(X) = pow(X)⋅P(X).
 * The claimed sum is 0 and is equal to ∑_{i ∈ {0,1}^d} pow(i)⋅P(i).
 * If the Sumcheck passes, then with it must hold with high-probability that all P(i) are 0.
 *
 * Init:
 * - σ_0 <-- 0 // Claimed Sumcheck sum
 * - c_0  <-- 1 // Partial evaluation constant, before any evaluation
 *
 * Round 0≤l<d-1:
 * - σ_{ l } =?= S'ˡ(0) + S'ˡ(1) = Tˡ(0) + ζ_{l}⋅Tˡ(1)  // Check partial sum
 * - σ_{l+1} <-- ( (1−u_{l}) + u_{l}⋅β_{l} )⋅Tʲ(u_{l})  // Compute next partial sum
 * - c_{l+1} <-- ( (1−u_{l}) + u_{l}⋅β_{l} )⋅c_{l}      // Partially evaluate pow in u_{l}
 *
 * Final round l=d-1:
 * - σ_{d-1} =?= S'ᵈ⁻¹(0) + S'ᵈ⁻¹(1) = Tᵈ⁻¹(0) + β_{d-1}⋅Tᵈ⁻¹(1)    // Check partial sum
 * - σ_{ d } <-- ( (1−u_{d-1}) + u_{d-1}⋅ζ_{0} )⋅Tᵈ⁻¹(u_{d-1})      // Compute purported evaluation of P'(u)
 * - c_{ d } <-- ∏_{0≤l<d} ( (1-u_{l}) + u_{l}⋅β_{l} )
 *             = pow(u_{0}, ..., u_{d-1})                           // Full evaluation of pow
 * - σ_{ d } =?= c_{d}⋅P(u_{0}, ..., u_{d-1})                       // Compare against real evaluation of P'(u)
 */

template <typename FF> struct PowPolynomial {

    // \vec{β} = {β_0, β_1,.., β_{d-1}}
    std::vector<FF> betas;

    // The values of pow_\vec{β}(i) for i=0,..,2^d - 1 for the given \vec{β}
    std::vector<FF> pow_betas;

    // At round l of sumcheck this will point to the l-th element in \vec{β}
    size_t current_element_idx = 0;

    // At round l of sumcheck, the periodicity represents the fixed interval at which elements not containing either of
    // β_0,..,β_l appear in pow_betas
    size_t periodicity = 2;

    // The value c_l obtained by partially evaluating one variable in the power polynomial at each round. At the
    // end of round l in the sumcheck protocol, variable X_l is replaced by a verifier challenge u_l. The partial
    // evaluation result is updated to represent pow(u_0,.., u_{l-1}) = \prod_{0 ≤ k < l} ( (1-u_k) + u_k⋅β_k).
    FF partial_evaluation_result = FF(1);

    explicit PowPolynomial(const std::vector<FF>& betas)
        : betas(betas)
    {}

    FF const& operator[](size_t idx) const { return pow_betas[idx]; }

    FF current_element() const { return betas[current_element_idx]; }

    /**
     * @brief Evaluate the monomial ((1−X_l) + X_l⋅β_l) in the challenge point X_l=u_l.
     */
    FF univariate_eval(FF challenge) const { return (FF(1) + (challenge * (betas[current_element_idx] - FF(1)))); };

    /**
     * @brief Parially evaluate the pow polynomial in X_l and updating the value c_l -> c_{l+1}.
     *
     * @param challenge l-th verifier challenge u_l
     */
    void partially_evaluate(FF challenge)
    {
        FF current_univariate_eval = univariate_eval(challenge);
        partial_evaluation_result *= current_univariate_eval;
        current_element_idx++;
        periodicity *= 2;
    }

    /**
     * @brief Given \vec{β} = {β_0,...,β_{d-1}} compute pow_\vec{β}(i) for i=0,...,2^{d}-1
     *
     */
    void compute_values()
    {
        size_t pow_size = 1 << betas.size();
        pow_betas = std::vector<FF>(pow_size);

        // Determine number of threads for multithreading.
        // Note: Multithreading is "on" for every round but we reduce the number of threads from the max available based
        // on a specified minimum number of iterations per thread. This eventually leads to the use of a single thread.
        // For now we use a power of 2 number of threads simply to ensure the round size is evenly divided.
        size_t max_num_threads = get_num_cpus_pow2(); // number of available threads (power of 2)
        size_t min_iterations_per_thread = 1 << 6; // min number of iterations for which we'll spin up a unique thread
        size_t desired_num_threads = pow_size / min_iterations_per_thread;
        size_t num_threads = std::min(desired_num_threads, max_num_threads); // fewer than max if justified
        num_threads = num_threads > 0 ? num_threads : 1;                     // ensure num threads is >= 1
        size_t iterations_per_thread = pow_size / num_threads;               // actual iterations per thread
        parallel_for(num_threads, [&](size_t thread_idx) {
            size_t start = thread_idx * iterations_per_thread;
            size_t end = (thread_idx + 1) * iterations_per_thread;
            for (size_t i = start; i < end; i++) {
                auto res = FF(1);
                for (size_t j = i, beta_idx = 0; j > 0; j >>= 1, beta_idx++) {
                    if ((j & 1) == 1) {
                        res *= betas[beta_idx];
                    }
                }
                pow_betas[i] = res;
            }
        });
    }
};
} // namespace bb