#pragma once
#include <array>
#include <algorithm>
#include "univariate.hpp"

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

/* IMPROVEMENT(Cody): This could or should be improved in various ways. In no particular order:
   1) Edge cases are not considered. One non-use case situation (I forget which) leads to a segfault.

   2) This could all be constexpr.

   3) Precomputing for all possible size pairs is probably feasible and might be a better solution than instantiating
   many instances separately. Then perhaps we could infer input type to `extend`.

   4) There should be more thorough testing of this class in isolation.
 */
namespace honk::sumcheck {

/**
 * NOTE: We should definitely consider question of optimal choice of domain, but if decide on {0,1,...,t-1} then we can
 * simplify the implementation a bit.
 * NOTE: if we use this approach in the recursive setting, will use Plookup?
 */
template <class Fr, size_t domain_size, size_t num_evals> class BarycentricData {
  public:
    static constexpr size_t big_domain_size = std::max(domain_size, num_evals);
    // TODO(Cody): these should be static, also constexpr, but arrays are not constexpr
    std::array<Fr, big_domain_size> big_domain;
    std::array<Fr, domain_size> lagrange_denominators;
    std::array<Fr, domain_size * num_evals> precomputed_denominator_inverses;
    std::array<Fr, num_evals> full_numerator_values;

    constexpr BarycentricData()
    {
        // build big_domain, currently the set of x_i in {0, 1, ..., t-1}
        for (size_t i = 0; i != big_domain_size; ++i) {
            big_domain[i] = i;
        }

        // build set of lagrange_denominators d_i = \prod_{j!=i} x_i - x_j
        for (size_t i = 0; i != domain_size; ++i) {
            lagrange_denominators[i] = 1;
            for (size_t j = 0; j != domain_size; ++j) {
                if (j != i) {
                    lagrange_denominators[i] *= big_domain[i] - big_domain[j];
                }
            }
        }

        // for each x_k in the big domain, build set of domain size-many denominator inverses
        // 1/(d_i*(x_k - x_j)). will multiply against each of these (rather than to divide by something)
        // for each barycentric evaluation
        for (size_t k = domain_size; k != num_evals; ++k) {
            for (size_t j = 0; j != domain_size; ++j) {
                Fr inv = lagrange_denominators[j];
                inv *= (big_domain[k] - big_domain[j]);
                inv = Fr(1) / inv; // prob have self_inverse method; should be part of Field concept
                precomputed_denominator_inverses[k * domain_size + j] = inv;
            }
        }

        // get full numerator values
        // full numerator is M(x) = \prod_{i} (x-x_i)
        // these will be zero for i < domain_size, but that's ok because
        // at such entries we will already have the evaluations of the polynomial
        for (size_t i = 0; i != num_evals; ++i) {
            full_numerator_values[i] = 1;
            Fr v_i = i;
            for (size_t j = 0; j != domain_size; ++j) {
                full_numerator_values[i] *= v_i - big_domain[j];
            }
        }
    }

    /**
     * @brief Given A univariate f represented by {f(0), ..., f(t-1)}, compute {f(t), ..., f(u-1)}
     * and return the Univariate represented by {f(0), ..., f(u-1)}.
     *
     * @details Write v_i = f(x_i) on a the domain {x_0, ..., x_{t-1}}. To efficiently compute the needed values of f,
     * we  use the barycentric formula
     *      - f(x) = B(x) Σ_{i=0}^{t-1} v_i / (d_i*(x-x_i))
     * where
     *      - B(x) = Π_{i=0}^{t-1} (x-x_i)
     *      - d_i  = Π_{j ∈ {0, ..., t-1}, j≠i} (x_i-x_j) for i ∈ {0, ..., t-1}
     *
     * NOTE: just taking x_i = i for now and possibly forever. Hence can significantly optimize:
     *       extending an Edge f = v0(1-X) + v1X to a new value involves just one addition and a subtraction:
     *       setting Δ  = v1-v0, the values of f(X) are
     *       f(0)=v0, f(1)= v0 + Δ, v2 = f(1) + Δ, v3 = f(2) + Δ...
     *
     */
    Univariate<Fr, num_evals> extend(Univariate<Fr, domain_size> f)
    {
        // ASSERT(u>t);
        Univariate<Fr, num_evals> result;

        for (size_t k = 0; k != domain_size; ++k) {
            result.value_at(k) = f.value_at(k);
        }

        for (size_t k = domain_size; k != num_evals; ++k) {
            result.value_at(k) = 0;
            // compute each term v_j / (d_j*(x-x_j)) of the sum
            for (size_t j = 0; j != domain_size; ++j) {
                Fr term = f.value_at(j);
                term *= precomputed_denominator_inverses[domain_size * k + j];
                result.value_at(k) += term;
            }
            // scale the sum by the the value of of B(x)
            result.value_at(k) *= full_numerator_values[k];
        }
        return result;
    }

    /**
     * @brief Evaluate a univariate at a point u not known at compile time
     * and assumed not to be in the domain (else we divide by zero).
     * @param f
     * @return Fr
     */
    Fr evaluate(Univariate<Fr, domain_size>& f, const Fr& u)
    {

        Fr full_numerator_value = 1;
        for (size_t i = 0; i != domain_size; ++i) {
            full_numerator_value *= u - i;
        }

        // build set of domain size-many denominator inverses 1/(d_i*(x_k - x_j)). will multiply against each of
        // these (rather than to divide by something) for each barycentric evaluation
        std::array<Fr, domain_size> denominator_inverses;
        for (size_t i = 0; i != domain_size; ++i) {
            Fr inv = lagrange_denominators[i];
            inv *= u - big_domain[i]; // warning: need to avoid zero here
            inv = Fr(1) / inv;
            denominator_inverses[i] = inv;
        }

        Fr result = 0;
        // compute each term v_j / (d_j*(x-x_j)) of the sum
        for (size_t i = 0; i != domain_size; ++i) {
            Fr term = f.value_at(i);
            term *= denominator_inverses[i];
            result += term;
        }
        // scale the sum by the the value of of B(x)
        result *= full_numerator_value;
        return result;
    };
};
} // namespace honk::sumcheck
