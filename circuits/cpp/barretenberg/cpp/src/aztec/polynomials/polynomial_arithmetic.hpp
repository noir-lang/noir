#pragma once
#include "evaluation_domain.hpp"

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace barretenberg {
namespace polynomial_arithmetic {
template <typename Fr> struct LagrangeEvaluations {
    Fr vanishing_poly;
    Fr l_start;
    Fr l_end;
};
using lagrange_evaluations = LagrangeEvaluations<fr>;

template <typename Fr> Fr evaluate(const Fr* coeffs, const Fr& z, const size_t n);
template <typename Fr> Fr evaluate(std::span<const Fr> coeffs, const Fr& z, const size_t n)
{
    ASSERT(n <= coeffs.size());
    return evaluate(coeffs.data(), z, n);
};
template <typename Fr> Fr evaluate(std::span<const Fr> coeffs, const Fr& z)
{
    return evaluate(coeffs, z, coeffs.size());
};
template <typename Fr> Fr evaluate(const std::vector<Fr*> coeffs, const Fr& z, const size_t large_n);
template <typename Fr>
void copy_polynomial(const Fr* src, Fr* dest, size_t num_src_coefficients, size_t num_target_coefficients);

//  2. Compute a lookup table of the roots of unity, and suffer through cache misses from nonlinear access patterns
template <typename Fr>
void fft_inner_serial(std::vector<Fr*> coeffs, const size_t domain_size, const std::vector<Fr*>& root_table);
template <typename Fr>
void fft_inner_parallel(std::vector<Fr*> coeffs,
                        const EvaluationDomain<Fr>& domain,
                        const Fr&,
                        const std::vector<Fr*>& root_table);

template <typename Fr> void fft(Fr* coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr> void fft(Fr* coeffs, Fr* target, const EvaluationDomain<Fr>& domain);
template <typename Fr> void fft(std::vector<Fr*> coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr> void fft_with_constant(Fr* coeffs, const EvaluationDomain<Fr>& domain, const Fr& value);

template <typename Fr> void coset_fft(Fr* coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr> void coset_fft(Fr* coeffs, Fr* target, const EvaluationDomain<Fr>& domain);
template <typename Fr> void coset_fft(std::vector<Fr*> coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr>
void coset_fft(Fr* coeffs,
               const EvaluationDomain<Fr>& small_domain,
               const EvaluationDomain<Fr>& large_domain,
               const size_t domain_extension);

template <typename Fr> void coset_fft_with_constant(Fr* coeffs, const EvaluationDomain<Fr>& domain, const Fr& constant);
template <typename Fr>
void coset_fft_with_generator_shift(Fr* coeffs, const EvaluationDomain<Fr>& domain, const Fr& constant);

template <typename Fr> void ifft(Fr* coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr> void ifft(Fr* coeffs, Fr* target, const EvaluationDomain<Fr>& domain);
template <typename Fr> void ifft(std::vector<Fr*> coeffs, const EvaluationDomain<Fr>& domain);

template <typename Fr> void ifft_with_constant(Fr* coeffs, const EvaluationDomain<Fr>& domain, const Fr& value);

template <typename Fr> void coset_ifft(Fr* coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr> void coset_ifft(std::vector<Fr*> coeffs, const EvaluationDomain<Fr>& domain);

template <typename Fr>
void partial_fft_serial_inner(Fr* coeffs,
                              Fr* target,
                              const EvaluationDomain<Fr>& domain,
                              const std::vector<Fr*>& root_table);
template <typename Fr>
void partial_fft_parellel_inner(Fr* coeffs,
                                const EvaluationDomain<Fr>& domain,
                                const std::vector<Fr*>& root_table,
                                Fr constant = 1,
                                bool is_coset = false);

template <typename Fr> void partial_fft_serial(Fr* coeffs, Fr* target, const EvaluationDomain<Fr>& domain);
template <typename Fr>
void partial_fft(Fr* coeffs, const EvaluationDomain<Fr>& domain, Fr constant = 1, bool is_coset = false);

template <typename Fr>
void add(const Fr* a_coeffs, const Fr* b_coeffs, Fr* r_coeffs, const EvaluationDomain<Fr>& domain);
template <typename Fr>
void sub(const Fr* a_coeffs, const Fr* b_coeffs, Fr* r_coeffs, const EvaluationDomain<Fr>& domain);

template <typename Fr>
void mul(const Fr* a_coeffs, const Fr* b_coeffs, Fr* r_coeffs, const EvaluationDomain<Fr>& domain);

// For L_1(X) = (X^{n} - 1 / (X - 1)) * (1 / n)
// Compute the size k*n-fft of L_1(X), where k is determined by the target domain (e.g. large_domain -> 4*n)
// We can use this to compute the k*n-fft evaluations of any L_i(X).
// We can consider `l_1_coefficients` to be a k*n-sized vector of the evaluations of L_1(X),
// for all X = k*n'th roots of unity.
// To compute the vector for the k*n-fft transform of L_i(X), we perform a (k*i)-left-shift of this vector
template <typename Fr>
void compute_lagrange_polynomial_fft(Fr* l_1_coefficients,
                                     const EvaluationDomain<Fr>& src_domain,
                                     const EvaluationDomain<Fr>& target_domain);

template <typename Fr>
void divide_by_pseudo_vanishing_polynomial(std::vector<Fr*> coeffs,
                                           const EvaluationDomain<Fr>& src_domain,
                                           const EvaluationDomain<Fr>& target_domain,
                                           const size_t num_roots_cut_out_of_vanishing_polynomial = 4);

// void populate_with_vanishing_polynomial(Fr* coeffs, const size_t num_non_zero_entries, const EvaluationDomain<Fr>&
// src_domain, const EvaluationDomain<Fr>& target_domain);

template <typename Fr> Fr compute_kate_opening_coefficients(const Fr* src, Fr* dest, const Fr& z, const size_t n);

// compute Z_H*(z), l_start(z), l_{end}(z) (= l_{n-4}(z))
template <typename Fr>
LagrangeEvaluations<Fr> get_lagrange_evaluations(const Fr& z,
                                                 const EvaluationDomain<Fr>& domain,
                                                 const size_t num_roots_cut_out_of_vanishing_polynomial = 4);
template <typename Fr>
Fr compute_barycentric_evaluation(const Fr* coeffs,
                                  const size_t num_coeffs,
                                  const Fr& z,
                                  const EvaluationDomain<Fr>& domain);
// Convert an fft with `current_size` point evaluations, to one with `current_size >> compress_factor` point evaluations
template <typename Fr>
void compress_fft(const Fr* src, Fr* dest, const size_t current_size, const size_t compress_factor);

template <typename Fr>
Fr evaluate_from_fft(const Fr* poly_coset_fft,
                     const EvaluationDomain<Fr>& large_domain,
                     const Fr& z,
                     const EvaluationDomain<Fr>& small_domain);

// This function computes sum of all scalars in a given array.
template <typename Fr> Fr compute_sum(const Fr* src, const size_t n);

// This function computes the polynomial (x - a)(x - b)(x - c)... given n distinct roots (a, b, c, ...).
template <typename Fr> void compute_linear_polynomial_product(const Fr* roots, Fr* dest, const size_t n);

// This function evaluates the polynomial (x - a)(x - b)(x - c)... given n distinct roots (a, b, c, ...)
// at x = z.
template <typename Fr> Fr compute_linear_polynomial_product_evaluation(const Fr* roots, const Fr z, const size_t n);

// This function computes the lagrange (or coset-lagrange) form of the polynomial (x - a)(x - b)(x - c)...
// given n distinct roots (a, b, c, ...).
template <typename Fr>
void fft_linear_polynomial_product(
    const Fr* roots, Fr* dest, const size_t n, const EvaluationDomain<Fr>& domain, const bool is_coset = false);

// This function interpolates from points {(z_1, f(z_1)), (z_2, f(z_2)), ...}.
// `src` contains {f(z_1), f(z_2), ...}
template <typename Fr> void compute_interpolation(const Fr* src, Fr* dest, const Fr* evaluation_points, const size_t n);

// This function interpolates from points {(z_1, f(z_1)), (z_2, f(z_2)), ...}
// using a single scalar inversion and Lagrange polynomial interpolation.
// `src` contains {f(z_1), f(z_2), ...}
template <typename Fr>
void compute_efficient_interpolation(const Fr* src, Fr* dest, const Fr* evaluation_points, const size_t n);

/**
 * @brief Divides p(X) by (X-r) in-place.
 */
template <typename Fr> void factor_roots(std::span<Fr> polynomial, const Fr& root)
{
    const size_t size = polynomial.size();
    if (root.is_zero()) {
        // if one of the roots is 0 after having divided by all other roots,
        // then p(X) = a₁⋅X + ⋯ + aₙ₋₁⋅Xⁿ⁻¹
        // so we shift the array of coefficients to the left
        // and the result is p(X) = a₁ + ⋯ + aₙ₋₁⋅Xⁿ⁻² and we subtract 1 from the size.
        std::copy_n(polynomial.begin() + 1, size - 1, polynomial.begin());
    } else {
        // assume
        //  • r != 0
        //  • (X−r) | p(X)
        //  • q(X) = ∑ᵢⁿ⁻² bᵢ⋅Xⁱ
        //  • p(X) = ∑ᵢⁿ⁻¹ aᵢ⋅Xⁱ = (X-r)⋅q(X)
        //
        // p(X)         0           1           2       ...     n-2             n-1
        //              a₀          a₁          a₂              aₙ₋₂            aₙ₋₁
        //
        // q(X)         0           1           2       ...     n-2             n-1
        //              b₀          b₁          b₂              bₙ₋₂            0
        //
        // (X-r)⋅q(X)   0           1           2       ...     n-2             n-1
        //              -r⋅b₀       b₀-r⋅b₁     b₁-r⋅b₂         bₙ₋₃−r⋅bₙ₋₂      bₙ₋₂
        //
        // b₀   = a₀⋅(−r)⁻¹
        // b₁   = (a₁ - b₀)⋅(−r)⁻¹
        // b₂   = (a₂ - b₁)⋅(−r)⁻¹
        //      ⋮
        // bᵢ   = (aᵢ − bᵢ₋₁)⋅(−r)⁻¹
        //      ⋮
        // bₙ₋₂ = (aₙ₋₂ − bₙ₋₃)⋅(−r)⁻¹
        // bₙ₋₁ = 0

        // For the simple case of one root we compute (−r)⁻¹ and
        Fr root_inverse = (-root).invert();
        // set b₋₁ = 0
        Fr temp = 0;
        // We start multiplying lower coefficient by the inverse and subtracting those from highter coefficients
        // Since (x - r) should divide the polynomial cleanly, we can guide division with lower coefficients
        for (size_t i = 0; i < size - 1; ++i) {
            // at the start of the loop, temp = bᵢ₋₁
            // and we can compute bᵢ   = (aᵢ − bᵢ₋₁)⋅(−r)⁻¹
            temp = (polynomial[i] - temp);
            temp *= root_inverse;
            polynomial[i] = temp;
        }
    }
    polynomial[size - 1] = Fr::zero();
}

/**
 * @brief Divides p(X) by (X-r₁)⋯(X−rₘ) in-place.
 * Assumes that p(rⱼ)=0 for all j
 *
 * @details we specialize the method when only a single root is given.
 * if one of the roots is 0, then we first factor all other roots.
 * dividing by X requires only a left shift of all coefficient.
 *
 * @param roots list of roots (r₁,…,rₘ)
 */
template <typename Fr> void factor_roots(std::span<Fr> polynomial, std::span<const Fr> roots)
{
    const size_t size = polynomial.size();
    if (roots.size() == 1) {
        factor_roots(polynomial, roots[0]);
    } else {
        // For division by several roots at once we need cache.
        // Let's say we are dividing a₀, a₁, a₂, ... by (r₀, r₁, r₂)
        // What we need to compute are the inverses: ((-r₀)⁻¹, (-r₁)⁻¹,(-r₂)⁻¹)
        // Then for a₀ we compute:
        //      a₀'   = a₀   * (-r₀)⁻¹
        //      a₀''  = a₀'  * (-r₁)⁻¹
        //      a₀''' = a₀'' * (-r₂)⁻¹
        // a₀''' is the lowest coefficient of the resulting polynomial
        // For a₁ we compute:
        //      a₁'   = (a₁   - a₀')   * (-r₀)⁻¹
        //      a₁''  = (a₁'  - a₀'')  * (-r₁)⁻¹
        //      a₁''' = (a₁'' - a₀''') * (-r₂)⁻¹
        // a₁''' is the second lowest coefficient of the resulting polynomial
        // As you see, we only need the intermediate results of the previous round in addition to inversed roots and the
        // original coefficient to calculate the resulting monomial coefficient. If we cache these results, we don't
        // have to do several passes over the polynomial

        const size_t num_roots = roots.size();
        ASSERT(num_roots < size);
        const size_t new_size = size - num_roots;

        std::vector<Fr> minus_root_inverses;
        minus_root_inverses.reserve(num_roots);

        // after the loop, this iterator points to the start of the polynomial
        // after having divided by all zero roots.
        size_t num_zero_roots{ 0 };
        // Compute negated root inverses, and skip the 0 root
        for (const auto& root : roots) {
            if (root.is_zero()) {
                // if one of the roots is zero, then the first coefficient must be as well
                // so we need to start the iteration from the second coefficient on-wards
                ++num_zero_roots;
            } else {
                minus_root_inverses.emplace_back(-root);
            }
        }
        // If there are M zero roots, then the first M coefficients of polynomial must be zero
        for (size_t i = 0; i < num_zero_roots; ++i) {
            ASSERT(polynomial[i].is_zero());
        }

        // View over the polynomial factored by all the zeros
        // If there are no zeros, then zero_factored == polynomial
        auto zero_factored = polynomial.subspan(num_zero_roots);

        const size_t num_non_zero_roots = minus_root_inverses.size();
        if (num_non_zero_roots > 0) {
            Fr::batch_invert(minus_root_inverses);

            std::vector<Fr> division_cache;
            division_cache.reserve(num_non_zero_roots);

            // Compute the a₀', a₀'', a₀''' and put them in cache
            Fr temp = zero_factored[0];
            for (const auto& minus_root_inverse : minus_root_inverses) {
                temp *= minus_root_inverse;
                division_cache.emplace_back(temp);
            }
            // We already know the lower coefficient of the result
            polynomial[0] = division_cache.back();

            // Compute the resulting coefficients one by one
            for (size_t i = 1; i < zero_factored.size() - num_non_zero_roots; i++) {
                temp = zero_factored[i];
                // Compute the intermediate values for the coefficient and save in cache
                for (size_t j = 0; j < num_non_zero_roots; j++) {
                    temp -= division_cache[j];
                    temp *= minus_root_inverses[j];
                    division_cache[j] = temp;
                }
                // Save the resulting coefficient
                polynomial[i] = temp;
            }
        } else if (num_zero_roots > 0) {
            // if one of the roots is 0 after having divided by all other roots,
            // then p(X) = a₁⋅X + ⋯ + aₙ₋₁⋅Xⁿ⁻¹
            // so we shift the array of coefficients to the left
            // and the result is p(X) = a₁ + ⋯ + aₙ₋₁⋅Xⁿ⁻² and we subtract 1 from the size.
            std::copy_n(zero_factored.begin(), zero_factored.size(), polynomial.begin());
        }

        // Clear the last coefficients to prevent accidents
        for (size_t i = new_size; i < size; ++i) {
            polynomial[i] = Fr::zero();
        }
    }
}

/**
 * TODO: Get rid of all this mess...
 *
 * Because this file was originally created for `barretenberg::fr`, it was probably natural to use .cpp/.hpp separation.
 * When we template all the functions, we need to actually instantiate it with `barretenberg::fr` or `grumpkin::fr`.
 * The corresponding definitions reside in the .cpp, so we need to tell the file that includes
 * `polynomial_arithmetic.hpp` that these functions are actually compiled somewhere.
 * This is the reason why we put the `extern` keyword, it tells the compiler, "don't worry, these functions actually
 * exist". In the .cpp, we place the same definitions without the `extern` keyword to tell the compiler "create these
 * functions with these types", because we told the compiler in .hpp that these functions must exist.
 *
 *
 *
 * Solution: we should probably put all these function inside a header.
 */

extern template fr evaluate<fr>(const fr*, const fr&, const size_t);
extern template fr evaluate<fr>(const std::vector<fr*>, const fr&, const size_t);
extern template void copy_polynomial<fr>(const fr*, fr*, size_t, size_t);
extern template void fft_inner_serial<fr>(std::vector<fr*>, const size_t, const std::vector<fr*>&);
extern template void fft_inner_parallel<fr>(std::vector<fr*>,
                                            const EvaluationDomain<fr>&,
                                            const fr&,
                                            const std::vector<fr*>&);
extern template void fft<fr>(fr*, const EvaluationDomain<fr>&);
extern template void fft<fr>(fr*, fr*, const EvaluationDomain<fr>&);
extern template void fft<fr>(std::vector<fr*>, const EvaluationDomain<fr>&);
extern template void fft_with_constant<fr>(fr*, const EvaluationDomain<fr>&, const fr&);
extern template void coset_fft<fr>(fr*, const EvaluationDomain<fr>&);
extern template void coset_fft<fr>(fr*, fr*, const EvaluationDomain<fr>&);
extern template void coset_fft<fr>(std::vector<fr*>, const EvaluationDomain<fr>&);
extern template void coset_fft<fr>(fr*, const EvaluationDomain<fr>&, const EvaluationDomain<fr>&, const size_t);
extern template void coset_fft_with_constant<fr>(fr*, const EvaluationDomain<fr>&, const fr&);
extern template void coset_fft_with_generator_shift<fr>(fr*, const EvaluationDomain<fr>&, const fr&);
extern template void ifft<fr>(fr*, const EvaluationDomain<fr>&);
extern template void ifft<fr>(fr*, fr*, const EvaluationDomain<fr>&);
extern template void ifft<fr>(std::vector<fr*>, const EvaluationDomain<fr>&);
extern template void ifft_with_constant<fr>(fr*, const EvaluationDomain<fr>&, const fr&);
extern template void coset_ifft<fr>(fr*, const EvaluationDomain<fr>&);
extern template void coset_ifft<fr>(std::vector<fr*>, const EvaluationDomain<fr>&);
extern template void partial_fft_serial_inner<fr>(fr*, fr*, const EvaluationDomain<fr>&, const std::vector<fr*>&);
extern template void partial_fft_parellel_inner<fr>(
    fr*, const EvaluationDomain<fr>&, const std::vector<fr*>&, fr, bool);
extern template void partial_fft_serial<fr>(fr*, fr*, const EvaluationDomain<fr>&);
extern template void partial_fft<fr>(fr*, const EvaluationDomain<fr>&, fr, bool);
extern template void add<fr>(const fr*, const fr*, fr*, const EvaluationDomain<fr>&);
extern template void sub<fr>(const fr*, const fr*, fr*, const EvaluationDomain<fr>&);
extern template void mul<fr>(const fr*, const fr*, fr*, const EvaluationDomain<fr>&);
extern template void compute_lagrange_polynomial_fft<fr>(fr*, const EvaluationDomain<fr>&, const EvaluationDomain<fr>&);
extern template void divide_by_pseudo_vanishing_polynomial<fr>(std::vector<fr*>,
                                                               const EvaluationDomain<fr>&,
                                                               const EvaluationDomain<fr>&,
                                                               const size_t);
extern template fr compute_kate_opening_coefficients<fr>(const fr*, fr*, const fr&, const size_t);
extern template LagrangeEvaluations<fr> get_lagrange_evaluations<fr>(const fr&,
                                                                     const EvaluationDomain<fr>&,
                                                                     const size_t);
extern template fr compute_barycentric_evaluation<fr>(const fr*, const size_t, const fr&, const EvaluationDomain<fr>&);
extern template void compress_fft<fr>(const fr*, fr*, const size_t, const size_t);
extern template fr evaluate_from_fft<fr>(const fr*,
                                         const EvaluationDomain<fr>&,
                                         const fr&,
                                         const EvaluationDomain<fr>&);
extern template fr compute_sum<fr>(const fr*, const size_t);
extern template void compute_linear_polynomial_product<fr>(const fr*, fr*, const size_t);
extern template fr compute_linear_polynomial_product_evaluation<fr>(const fr*, const fr, const size_t);
extern template void fft_linear_polynomial_product<fr>(
    const fr* roots, fr*, const size_t n, const EvaluationDomain<fr>&, const bool);
extern template void compute_interpolation<fr>(const fr*, fr*, const fr*, const size_t);
extern template void compute_efficient_interpolation<fr>(const fr*, fr*, const fr*, const size_t);

extern template grumpkin::fr evaluate<grumpkin::fr>(const grumpkin::fr*, const grumpkin::fr&, const size_t);
extern template grumpkin::fr evaluate<grumpkin::fr>(const std::vector<grumpkin::fr*>,
                                                    const grumpkin::fr&,
                                                    const size_t);
extern template void copy_polynomial<grumpkin::fr>(const grumpkin::fr*, grumpkin::fr*, size_t, size_t);
extern template void fft_inner_serial<grumpkin::fr>(std::vector<grumpkin::fr*>,
                                                    const size_t,
                                                    const std::vector<grumpkin::fr*>&);
extern template void fft_inner_parallel<grumpkin::fr>(std::vector<grumpkin::fr*>,
                                                      const EvaluationDomain<grumpkin::fr>&,
                                                      const grumpkin::fr&,
                                                      const std::vector<grumpkin::fr*>&);
extern template void fft<grumpkin::fr>(grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void fft<grumpkin::fr>(grumpkin::fr*, grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void fft<grumpkin::fr>(std::vector<grumpkin::fr*>, const EvaluationDomain<grumpkin::fr>&);
extern template void fft_with_constant<grumpkin::fr>(grumpkin::fr*,
                                                     const EvaluationDomain<grumpkin::fr>&,
                                                     const grumpkin::fr&);
extern template void coset_fft<grumpkin::fr>(grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void coset_fft<grumpkin::fr>(grumpkin::fr*, grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void coset_fft<grumpkin::fr>(std::vector<grumpkin::fr*>, const EvaluationDomain<grumpkin::fr>&);
extern template void coset_fft<grumpkin::fr>(grumpkin::fr*,
                                             const EvaluationDomain<grumpkin::fr>&,
                                             const EvaluationDomain<grumpkin::fr>&,
                                             const size_t);
extern template void coset_fft_with_constant<grumpkin::fr>(grumpkin::fr*,
                                                           const EvaluationDomain<grumpkin::fr>&,
                                                           const grumpkin::fr&);
extern template void coset_fft_with_generator_shift<grumpkin::fr>(grumpkin::fr*,
                                                                  const EvaluationDomain<grumpkin::fr>&,
                                                                  const grumpkin::fr&);
extern template void ifft<grumpkin::fr>(grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void ifft<grumpkin::fr>(grumpkin::fr*, grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void ifft<grumpkin::fr>(std::vector<grumpkin::fr*>, const EvaluationDomain<grumpkin::fr>&);
extern template void ifft_with_constant<grumpkin::fr>(grumpkin::fr*,
                                                      const EvaluationDomain<grumpkin::fr>&,
                                                      const grumpkin::fr&);
extern template void coset_ifft<grumpkin::fr>(grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&);
extern template void coset_ifft<grumpkin::fr>(std::vector<grumpkin::fr*>, const EvaluationDomain<grumpkin::fr>&);
extern template void partial_fft_serial_inner<grumpkin::fr>(grumpkin::fr*,
                                                            grumpkin::fr*,
                                                            const EvaluationDomain<grumpkin::fr>&,
                                                            const std::vector<grumpkin::fr*>&);
extern template void partial_fft_parellel_inner<grumpkin::fr>(
    grumpkin::fr*, const EvaluationDomain<grumpkin::fr>&, const std::vector<grumpkin::fr*>&, grumpkin::fr, bool);
extern template void partial_fft_serial<grumpkin::fr>(grumpkin::fr*,
                                                      grumpkin::fr*,
                                                      const EvaluationDomain<grumpkin::fr>&);
extern template void partial_fft<grumpkin::fr>(grumpkin::fr*,
                                               const EvaluationDomain<grumpkin::fr>&,
                                               grumpkin::fr,
                                               bool);
extern template void add<grumpkin::fr>(const grumpkin::fr*,
                                       const grumpkin::fr*,
                                       grumpkin::fr*,
                                       const EvaluationDomain<grumpkin::fr>&);
extern template void sub<grumpkin::fr>(const grumpkin::fr*,
                                       const grumpkin::fr*,
                                       grumpkin::fr*,
                                       const EvaluationDomain<grumpkin::fr>&);
extern template void mul<grumpkin::fr>(const grumpkin::fr*,
                                       const grumpkin::fr*,
                                       grumpkin::fr*,
                                       const EvaluationDomain<grumpkin::fr>&);
extern template void compute_lagrange_polynomial_fft<grumpkin::fr>(grumpkin::fr*,
                                                                   const EvaluationDomain<grumpkin::fr>&,
                                                                   const EvaluationDomain<grumpkin::fr>&);
extern template void divide_by_pseudo_vanishing_polynomial<grumpkin::fr>(std::vector<grumpkin::fr*>,
                                                                         const EvaluationDomain<grumpkin::fr>&,
                                                                         const EvaluationDomain<grumpkin::fr>&,
                                                                         const size_t);
extern template grumpkin::fr compute_kate_opening_coefficients<grumpkin::fr>(const grumpkin::fr*,
                                                                             grumpkin::fr*,
                                                                             const grumpkin::fr&,
                                                                             const size_t);
extern template LagrangeEvaluations<grumpkin::fr> get_lagrange_evaluations<grumpkin::fr>(
    const grumpkin::fr&, const EvaluationDomain<grumpkin::fr>&, const size_t);
extern template grumpkin::fr compute_barycentric_evaluation<grumpkin::fr>(const grumpkin::fr*,
                                                                          const size_t,
                                                                          const grumpkin::fr&,
                                                                          const EvaluationDomain<grumpkin::fr>&);
extern template void compress_fft<grumpkin::fr>(const grumpkin::fr*, grumpkin::fr*, const size_t, const size_t);
extern template grumpkin::fr evaluate_from_fft<grumpkin::fr>(const grumpkin::fr*,
                                                             const EvaluationDomain<grumpkin::fr>&,
                                                             const grumpkin::fr&,
                                                             const EvaluationDomain<grumpkin::fr>&);
extern template grumpkin::fr compute_sum<grumpkin::fr>(const grumpkin::fr*, const size_t);
extern template void compute_linear_polynomial_product<grumpkin::fr>(const grumpkin::fr*, grumpkin::fr*, const size_t);
extern template grumpkin::fr compute_linear_polynomial_product_evaluation<grumpkin::fr>(const grumpkin::fr*,
                                                                                        const grumpkin::fr,
                                                                                        const size_t);
extern template void fft_linear_polynomial_product<grumpkin::fr>(
    const grumpkin::fr* roots, grumpkin::fr*, const size_t n, const EvaluationDomain<grumpkin::fr>&, const bool);
extern template void compute_interpolation<grumpkin::fr>(const grumpkin::fr*,
                                                         grumpkin::fr*,
                                                         const grumpkin::fr*,
                                                         const size_t);
extern template void compute_efficient_interpolation<grumpkin::fr>(const grumpkin::fr*,
                                                                   grumpkin::fr*,
                                                                   const grumpkin::fr*,
                                                                   const size_t);

} // namespace polynomial_arithmetic
} // namespace barretenberg
