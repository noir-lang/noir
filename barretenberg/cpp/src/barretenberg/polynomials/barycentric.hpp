#pragma once
#include "barretenberg/ecc/fields/field.hpp"
#include <array>

// TODO(#674): We need the functionality of BarycentricData for both field (native) and field_t (stdlib). The former is
// is compatible with constexpr operations, and the former is not. The functions for computing the
// pre-computable arrays in BarycentricData need to be constexpr and it takes some trickery to share these functions
// with the non-constexpr setting. Right now everything is more or less duplicated across BarycentricDataCompileTime and
// BarycentricDataRunTime. There should be a way to share more of the logic.

/* IMPROVEMENT(Cody): This could or should be improved in various ways. In no particular order:
   1) Edge cases are not considered. One non-use case situation (I forget which) leads to a segfault.

   2) Precomputing for all possible size pairs is probably feasible and might be a better solution than instantiating
   many instances separately. Then perhaps we could infer input type to `extend`.

   3) There should be more thorough testing of this class in isolation.
 */
namespace bb {

/**
 * @todo: TODO(https://github.com/AztecProtocol/barretenberg/issues/713) Optimize with lookup tables?
 * @tparam domain_end, domain_start specify the given evaluation domain {domain_start,..., domain_end - 1}
 * @tparam num_evals the number of evaluations that are computable with specific barycentric extension formula
 */

template <class Fr, size_t domain_end, size_t num_evals, size_t domain_start = 0> class BarycentricDataCompileTime {
  public:
    static constexpr size_t domain_size = domain_end - domain_start;
    static constexpr size_t big_domain_size = std::max(domain_size, num_evals);

    /**
     * Static constexpr methods for computing arrays of precomputable data used for barycentric extension and evaluation
     */

    // build big_domain, currently the set of x_i in {domain_start, ..., big_domain_end - 1 }
    static constexpr std::array<Fr, big_domain_size> construct_big_domain()
    {
        std::array<Fr, big_domain_size> result;
        for (size_t i = 0; i < big_domain_size; ++i) {
            result[i] = static_cast<Fr>(i + domain_start);
        }
        return result;
    }

    // build set of lagrange_denominators d_i = \prod_{j!=i} x_i - x_j
    static constexpr std::array<Fr, domain_size> construct_lagrange_denominators(const auto& big_domain)
    {
        std::array<Fr, domain_size> result;
        for (size_t i = 0; i != domain_size; ++i) {
            result[i] = 1;
            for (size_t j = 0; j != domain_size; ++j) {
                if (j != i) {
                    result[i] *= big_domain[i] - big_domain[j];
                }
            }
        }
        return result;
    }

    static constexpr std::array<Fr, domain_size * num_evals> batch_invert(
        const std::array<Fr, domain_size * num_evals>& coeffs)
    {
        constexpr size_t n = domain_size * num_evals;
        std::array<Fr, n> temporaries{};
        std::array<bool, n> skipped{};
        Fr accumulator = 1;
        for (size_t i = 0; i < n; ++i) {
            temporaries[i] = accumulator;
            if (coeffs[i] == 0) {
                skipped[i] = true;
            } else {
                skipped[i] = false;
                accumulator *= coeffs[i];
            }
        }
        accumulator = Fr(1) / accumulator;
        std::array<Fr, n> result{};
        Fr T0;
        for (size_t i = n - 1; i < n; --i) {
            if (!skipped[i]) {
                T0 = accumulator * temporaries[i];
                accumulator *= coeffs[i];
                result[i] = T0;
            }
        }
        return result;
    }
    // for each x_k in the big domain, build set of domain size-many denominator inverses
    // 1/(d_i*(x_k - x_j)). will multiply against each of these (rather than to divide by something)
    // for each barycentric evaluation
    static constexpr std::array<Fr, domain_size * num_evals> construct_denominator_inverses(
        const auto& big_domain, const auto& lagrange_denominators)
    {
        std::array<Fr, domain_size * num_evals> result{}; // default init to 0 since below does not init all elements
        for (size_t k = domain_size; k < num_evals; ++k) {
            for (size_t j = 0; j < domain_size; ++j) {
                Fr inv = lagrange_denominators[j];
                inv *= (big_domain[k] - big_domain[j]);
                result[k * domain_size + j] = inv;
            }
        }
        return batch_invert(result);
    }

    // get full numerator values
    // full numerator is M(x) = \prod_{i} (x-x_i)
    // these will be zero for i < domain_size, but that's ok because
    // at such entries we will already have the evaluations of the polynomial
    static constexpr std::array<Fr, num_evals> construct_full_numerator_values(const auto& big_domain)
    {
        std::array<Fr, num_evals> result;
        for (size_t i = 0; i != num_evals; ++i) {
            result[i] = 1;
            Fr v_i = i + domain_start;
            for (size_t j = 0; j != domain_size; ++j) {
                result[i] *= v_i - big_domain[j];
            }
        }
        return result;
    }

    static constexpr auto big_domain = construct_big_domain();
    static constexpr auto lagrange_denominators = construct_lagrange_denominators(big_domain);
    static constexpr auto precomputed_denominator_inverses =
        construct_denominator_inverses(big_domain, lagrange_denominators);
    static constexpr auto full_numerator_values = construct_full_numerator_values(big_domain);
};

template <class Fr, size_t domain_end, size_t num_evals, size_t domain_start = 0> class BarycentricDataRunTime {
  public:
    static constexpr size_t domain_size = domain_end - domain_start;
    static constexpr size_t big_domain_size = std::max(domain_size, num_evals);

    /**
     * Static constexpr methods for computing arrays of precomputable data used for barycentric extension and evaluation
     */

    // build big_domain, currently the set of x_i in {domain_start, ..., big_domain_end - 1 }
    static std::array<Fr, big_domain_size> construct_big_domain()
    {
        std::array<Fr, big_domain_size> result;
        for (size_t i = 0; i < big_domain_size; ++i) {
            result[i] = static_cast<Fr>(i + domain_start);
        }
        return result;
    }

    // build set of lagrange_denominators d_i = \prod_{j!=i} x_i - x_j
    static std::array<Fr, domain_size> construct_lagrange_denominators(const auto& big_domain)
    {
        std::array<Fr, domain_size> result;
        for (size_t i = 0; i != domain_size; ++i) {
            result[i] = 1;
            for (size_t j = 0; j != domain_size; ++j) {
                if (j != i) {
                    result[i] *= big_domain[i] - big_domain[j];
                }
            }
        }
        return result;
    }

    static std::array<Fr, domain_size * num_evals> batch_invert(const std::array<Fr, domain_size * num_evals>& coeffs)
    {
        constexpr size_t n = domain_size * num_evals;
        std::array<Fr, n> temporaries{};
        std::array<bool, n> skipped{};
        Fr accumulator = 1;
        for (size_t i = 0; i < n; ++i) {
            temporaries[i] = accumulator;
            if (coeffs[i].get_value() == 0) {
                skipped[i] = true;
            } else {
                skipped[i] = false;
                accumulator *= coeffs[i];
            }
        }
        accumulator = Fr(1) / accumulator;
        std::array<Fr, n> result{};
        Fr T0;
        for (size_t i = n - 1; i < n; --i) {
            if (!skipped[i]) {
                T0 = accumulator * temporaries[i];
                accumulator *= coeffs[i];
                result[i] = T0;
            }
        }
        return result;
    }
    // for each x_k in the big domain, build set of domain size-many denominator inverses
    // 1/(d_i*(x_k - x_j)). will multiply against each of these (rather than to divide by something)
    // for each barycentric evaluation
    static std::array<Fr, domain_size * num_evals> construct_denominator_inverses(const auto& big_domain,
                                                                                  const auto& lagrange_denominators)
    {
        std::array<Fr, domain_size * num_evals> result{}; // default init to 0 since below does not init all elements
        for (size_t k = domain_size; k < num_evals; ++k) {
            for (size_t j = 0; j < domain_size; ++j) {
                Fr inv = lagrange_denominators[j];
                inv *= (big_domain[k] - big_domain[j]);
                result[k * domain_size + j] = inv;
            }
        }
        return batch_invert(result);
    }

    // get full numerator values
    // full numerator is M(x) = \prod_{i} (x-x_i)
    // these will be zero for i < domain_size, but that's ok because
    // at such entries we will already have the evaluations of the polynomial
    static std::array<Fr, num_evals> construct_full_numerator_values(const auto& big_domain)
    {
        std::array<Fr, num_evals> result;
        for (size_t i = 0; i != num_evals; ++i) {
            result[i] = 1;
            Fr v_i = i + domain_start;
            for (size_t j = 0; j != domain_size; ++j) {
                result[i] *= v_i - big_domain[j];
            }
        }
        return result;
    }

    inline static const auto big_domain = construct_big_domain();
    inline static const auto lagrange_denominators = construct_lagrange_denominators(big_domain);
    inline static const auto precomputed_denominator_inverses =
        construct_denominator_inverses(big_domain, lagrange_denominators);
    inline static const auto full_numerator_values = construct_full_numerator_values(big_domain);
};

/**
 * @brief Helper to determine whether input is bberg::field type
 *
 * @tparam T
 */
template <typename T> struct is_field_type {
    static constexpr bool value = false;
};

template <typename Params> struct is_field_type<bb::field<Params>> {
    static constexpr bool value = true;
};

template <typename T> inline constexpr bool is_field_type_v = is_field_type<T>::value;

/**
 * @brief Exposes BarycentricData with compile time arrays if the type is bberg::field and runtime arrays otherwise
 * @details This method is also needed for stdlib field, for which the arrays are not compile time computable
 * @tparam Fr
 * @tparam domain_size
 * @tparam num_evals
 */
template <class Fr, size_t domain_end, size_t num_evals, size_t domain_start = 0>
using BarycentricData = std::conditional_t<is_field_type_v<Fr>,
                                           BarycentricDataCompileTime<Fr, domain_end, num_evals, domain_start>,
                                           BarycentricDataRunTime<Fr, domain_end, num_evals, domain_start>>;

} // namespace bb
