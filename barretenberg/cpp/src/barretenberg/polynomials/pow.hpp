#pragma once
#include "barretenberg/common/compiler_hints.hpp"
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/common/thread.hpp"

#include <cstddef>
#include <vector>
namespace bb {

template <typename FF> struct PowPolynomial {

    /**
     * @brief The challenges \f$(\beta_0,\ldots, \beta_{d-1}) \f$
     *
     */
    std::vector<FF> betas;

    /**
     * @brief The consecutive evaluations \f$ pow_{\ell}(\beta) =  pow_{\beta}(\vec \ell) \f$ for \f$\vec \ell\f$
     * identified with the integers \f$\ell = 0,\ldots, 2^d-1\f$
     *
     */
    std::vector<FF> pow_betas;
    /**
     * @brief In Round \f$ i\f$ of Sumcheck, it points to the \f$ i \f$-th element in \f$ \vec \beta \f$
     *
     */
    size_t current_element_idx = 0;
    /**
     * @brief In Round \f$ i\f$ of Sumcheck, the periodicity equals to \f$ 2^{i+1}\f$ and represents the fixed interval
     * at which elements not containing either of \f$ (\beta_0,\ldots ,β_i)\f$ appear in #pow_betas.
     *
     */
    size_t periodicity = 2;
    /**
     * @brief  The value \f$c_i\f$ obtained by partially evaluating one variable in the power polynomial at each round.
     * At the end of Round \f$ i \f$ in the sumcheck protocol, variable \f$X_i\f$ is replaced by the challenge \f$u_i
     * \f$. The partial evaluation result is updated to represent \f$ pow_{\beta}(u_0,.., u_{i}) = \prod_{k=0}^{i} (
     * (1-u_k) + u_k\cdot \beta_k) \f$.
     *
     */
    FF partial_evaluation_result = FF(1);

    explicit PowPolynomial(const std::vector<FF>& betas)
        : betas(betas)
    {}
    /**
     * @brief Retruns the element in #pow_betas at place #idx.
     *
     * @param idx
     * @return FF const&
     */
    FF const& operator[](size_t idx) const { return pow_betas[idx]; }
    /**
     * @brief Computes the component  at index #current_element_idx in #betas.
     *
     * @return FF
     */
    FF current_element() const { return betas[current_element_idx]; }

    /**
     * @brief Evaluate  \f$ ((1−X_{i}) + X_{i}\cdot \beta_{i})\f$ at the challenge point \f$ X_{i}=u_{i} \f$.
     */
    FF univariate_eval(FF challenge) const { return (FF(1) + (challenge * (betas[current_element_idx] - FF(1)))); };

    /**
     * @brief Partially evaluate the \f$pow_{\beta} \f$-polynomial at the new challenge and update \f$ c_i \f$
     * @details Update the constant \f$c_{i} \to c_{i+1} \f$ multiplying it by \f$pow_{\beta}\f$'s factor \f$\left(
     * (1-X_i) + X_i\cdot \beta_i\right)\vert_{X_i = u_i}\f$ computed by \ref univariate_eval.
     * @param challenge \f$ i \f$-th verifier challenge \f$ u_{i}\f$
     */
    void partially_evaluate(FF challenge)
    {
        FF current_univariate_eval = univariate_eval(challenge);
        partial_evaluation_result *= current_univariate_eval;
        current_element_idx++;
        periodicity *= 2;
    }

    /**
     * @brief Given \f$ \vec\beta = (\beta_0,...,\beta_{d-1})\f$ compute \f$ pow_{\ell}(\vec \beta) = pow_{\beta}(\vec
     * \ell)\f$ for \f$ \ell =0,\ldots,2^{d}-1\f$.
     *
     */
    BB_PROFILE void compute_values()
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

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/864): This computation is asymtotically slow as it
        // does pow_size * log(pow_size) work. However, in practice, its super efficient because its trivially
        // parallelizable and only takes 45ms for the whole 6 iter IVC benchmark. Its also very readable, so we're
        // leaving it unoptimized for now.
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
/**<
 * @struct PowPolynomial
 * @brief Implementation of the methods for the \f$pow_{\ell}\f$-polynomials used in ProtoGalaxy and
\f$pow_{\beta}\f$-polynomials used in Sumcheck.
 *
 * @details
 * ## PowPolynomial in Protogalaxy
 *
 * \todo Expand this while completing PG docs.
 *
 * For \f$0\leq \ell \leq 2^d-1 \f$, the \f$pow_{\ell} \f$-polynomials used in Protogalaxy is a multilinear polynomial
defined by the formula
 * \f{align} pow_{\ell}(X_0,\ldots, X_{d-1})
          =    \prod_{k=0}^{d-1} ( ( 1-\ell_k ) + \ell_k \cdot X_k )
          =      \prod_{k=0}^{d-1} X_{k}^{ \ell_k }
     \f}
 *where \f$(\ell_0,\ldots, \ell_{d-1})\f$ is the binary representation of \f$\ell \f$.
 *
 *
  ## Pow-contributions to Round Univariates in Sumcheck {#PowContributions}
 * For a fixed \f$ \vec \beta \in \mathbb{F}^d\f$, the map \f$ \ell \mapsto pow_{\ell} (\vec \beta)\f$ defines a
 polynomial \f{align}{ pow_{\beta} (X_0,\ldots, X_{d-1}) = \prod_{k=0}^{d-1} (1- X_k + X_k \cdot \beta_k)
 \f}
such that \f$ pow_{\beta} (\vec \ell) = pow_{\ell} (\vec \beta) \f$ for any \f$0\leq \ell \leq 2^d-1 \f$ and any vector
\f$(\beta_0,\ldots, \beta_{d-1})  \in \mathbb{F} ^d\f$.

 * Let \f$ i \f$ be the current Sumcheck round, \f$ i \in \{0, …, d-1\}\f$ and \f$ u_{0}, ..., u_{i-1} \f$ be the
challenges generated in Rounds \f$ 0 \f$ to \f$ i-1\f$.
 *
 * In Round \f$ i \f$, we iterate over the points \f$ (\ell_{i+1}, \ldots, \ell_{d-1}) \in
\{0,1\}^{d-1-i}\f$.
Define a univariate polynomial \f$pow_{\beta}^i(X_i, \vec \ell) \f$  as follows
 *   \f{align}{  pow_{\beta}^i(X_i, \vec \ell) =   pow_{\beta} ( u_{0}, ..., u_{i-1}, X_i, \ell_{i+1}, \ldots,
\ell_{d-1})            = c_i \cdot ( (1−X_i) + X_i \cdot \beta_i ) \cdot \beta_{i+1}^{\ell_{i+1}}\cdot \cdots \cdot
\beta_{d-1}^{\ell_{d-1}}, \f} where \f$ c_i = \prod_{k=0}^{i-1} (1- u_k + u_k \cdot \beta_k) \f$. It will be used below
to simplify the computation of Sumcheck round univariates.

 ### Computing Sumcheck Round Univariates
 * We identify \f$ \vec \ell = (\ell_{i+1}, \ldots, \ell_{d-1}) \in \{0,1\}^{d-1 - i}\f$ with the binary representation
of the integer \f$ \ell \in \{0,\ldots, 2^{d-1-i}-1 \}\f$.
 *
 * Set
  \f{align}{S^i_{\ell}( X_i ) = F( u_{0}, ..., u_{i-1}, X_{i},  \vec \ell ), \f}
 * i.e. \f$ S^{i}_{\ell}( X_i ) \f$  is the univariate of the full relation \f$ F \f$ defined by its partial evaluation
at \f$(u_0,\ldots,u_{i-1},  \ell_{i+1},\ldots, \ell_{d-1}) \f$
 * which  is an alpha-linear-combination of the subrelations evaluated at this point.
 *
 * In Round \f$i\f$, the prover
 * \ref bb::SumcheckProverRound< Flavor >::compute_univariate "computes the univariate polynomial" for the relation
defined by \f$ \tilde{F} (X_0,\ldots, X_{d-1}) = pow_{\beta}(X_0,\ldots, X_{d-1}) \cdot F\f$, namely
 * \f{align}{
    \tilde{S}^{i}(X_i) = \sum_{ \ell = 0} ^{2^{d-i-1}-1}  pow^i_\beta ( X_i, \ell_{i+1}, \ldots, \ell_{d-1} )
S^i_{\ell}( X_i )
 *        =  c_i \cdot ( (1−X_i) + X_i\cdot \beta_i )  \cdot \sum_{ \ell = 0} ^{2^{d-i-1}-1} \beta_{i+1}^{\ell_{i+1}}
\cdot \ldots \cdot \beta_{d-1}^{\ell_{d-1}} \cdot S^i_{\ell}( X_i ) \f}
 *
 * Define
 \f{align} T^{i}( X_i ) =  \sum_{\ell = 0}^{2^{d-i-1}-1} \beta_{i+1}^{\ell_{i+1}} \cdot \ldots \cdot
\beta_{d-1}^{\ell_{d-1}} \cdot S^{i}_{\ell}( X_i ) \f} then \f$ \deg_{X_i} (T^i) \leq \deg_{X_i} S^i \f$.
 ### Features of PowPolynomial used by Sumcheck Prover
 - The factor \f$ c_i \f$ is the #partial_evaluation_result, it is updated by \ref partially_evaluate.
 - The challenges \f$(\beta_0,\ldots, \beta_{d-1}) \f$ are recorded in #betas.
 - The consecutive evaluations \f$ pow_{\ell}(\vec \beta) = pow_{\beta}(\vec \ell) \f$ for \f$\vec \ell\f$ identified
with the integers \f$\ell = 0,\ldots, 2^d-1\f$ represented in binary are pre-computed by \ref compute_values and stored
in #pow_betas.
 *
 */

} // namespace bb