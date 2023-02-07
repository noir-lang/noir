#include "polynomial_arithmetic.hpp"
#include <algorithm>
#include <common/mem.hpp>
#include <cstddef>
#include <gtest/gtest.h>
#include <utility>
#include "numeric/bitop/get_msb.hpp"
#include "numeric/random/engine.hpp"
#include "polynomial.hpp"

using namespace barretenberg;

TEST(polynomials, evaluation_domain)
{
    constexpr size_t n = 256;
    evaluation_domain domain = evaluation_domain(n);

    EXPECT_EQ(domain.size, 256UL);
    EXPECT_EQ(domain.log2_size, 8UL);
}

TEST(polynomials, domain_roots)
{
    constexpr size_t n = 256;
    evaluation_domain domain = evaluation_domain(n);

    fr result;
    fr expected;
    expected = fr::one();
    result = domain.root.pow(static_cast<uint64_t>(n));

    EXPECT_EQ((result == expected), true);
}

TEST(polynomials, evaluation_domain_roots)
{
    constexpr size_t n = 16;
    evaluation_domain domain(n);
    domain.compute_lookup_table();
    std::vector<fr*> root_table = domain.get_round_roots();
    std::vector<fr*> inverse_root_table = domain.get_inverse_round_roots();
    fr* roots = root_table[root_table.size() - 1];
    fr* inverse_roots = inverse_root_table[inverse_root_table.size() - 1];
    for (size_t i = 0; i < (n - 1) / 2; ++i) {
        EXPECT_EQ(roots[i] * domain.root, roots[i + 1]);
        EXPECT_EQ(inverse_roots[i] * domain.root_inverse, inverse_roots[i + 1]);
        EXPECT_EQ(roots[i] * inverse_roots[i], fr::one());
    }
}
TEST(polynomials, fft_with_small_degree)
{
    constexpr size_t n = 16;
    fr fft_transform[n];
    fr poly[n];

    for (size_t i = 0; i < n; ++i) {
        poly[i] = fr::random_element();
        fr::__copy(poly[i], fft_transform[i]);
    }

    evaluation_domain domain = evaluation_domain(n);
    domain.compute_lookup_table();
    polynomial_arithmetic::fft(fft_transform, domain);

    fr work_root;
    work_root = fr::one();
    fr expected;
    for (size_t i = 0; i < n; ++i) {
        expected = polynomial_arithmetic::evaluate(poly, work_root, n);
        EXPECT_EQ((fft_transform[i] == expected), true);
        work_root *= domain.root;
    }
}

TEST(polynomials, split_polynomial_fft)
{
    constexpr size_t n = 256;
    fr fft_transform[n];
    fr poly[n];

    for (size_t i = 0; i < n; ++i) {
        poly[i] = fr::random_element();
        fr::__copy(poly[i], fft_transform[i]);
    }

    constexpr size_t num_poly = 4;
    constexpr size_t n_poly = n / num_poly;
    fr fft_transform_[num_poly][n_poly];
    for (size_t i = 0; i < n; ++i) {
        fft_transform_[i / n_poly][i % n_poly] = poly[i];
    }

    evaluation_domain domain = evaluation_domain(n);
    domain.compute_lookup_table();
    polynomial_arithmetic::fft(fft_transform, domain);
    polynomial_arithmetic::fft({ fft_transform_[0], fft_transform_[1], fft_transform_[2], fft_transform_[3] }, domain);

    fr work_root;
    work_root = fr::one();
    fr expected;

    for (size_t i = 0; i < n; ++i) {
        expected = polynomial_arithmetic::evaluate(poly, work_root, n);
        EXPECT_EQ((fft_transform[i] == expected), true);
        EXPECT_EQ(fft_transform_[i / n_poly][i % n_poly], fft_transform[i]);
        work_root *= domain.root;
    }
}

TEST(polynomials, split_polynomial_evaluate)
{
    constexpr size_t n = 256;
    fr fft_transform[n];
    fr poly[n];

    for (size_t i = 0; i < n; ++i) {
        poly[i] = fr::random_element();
        fr::__copy(poly[i], fft_transform[i]);
    }

    constexpr size_t num_poly = 4;
    constexpr size_t n_poly = n / num_poly;
    fr fft_transform_[num_poly][n_poly];
    for (size_t i = 0; i < n; ++i) {
        fft_transform_[i / n_poly][i % n_poly] = poly[i];
    }

    fr z = fr::random_element();
    EXPECT_EQ(polynomial_arithmetic::evaluate(
                  { fft_transform_[0], fft_transform_[1], fft_transform_[2], fft_transform_[3] }, z, n),
              polynomial_arithmetic::evaluate(poly, z, n));
}

TEST(polynomials, basic_fft)
{
    constexpr size_t n = 1 << 14;
    fr* data = (fr*)aligned_alloc(32, sizeof(fr) * n * 2);
    fr* result = &data[0];
    fr* expected = &data[n];
    for (size_t i = 0; i < n; ++i) {
        result[i] = fr::random_element();
        fr::__copy(result[i], expected[i]);
    }

    evaluation_domain domain = evaluation_domain(n);
    domain.compute_lookup_table();
    polynomial_arithmetic::fft(result, domain);
    polynomial_arithmetic::ifft(result, domain);

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ((result[i] == expected[i]), true);
    }
    aligned_free(data);
}

TEST(polynomials, fft_ifft_consistency)
{
    constexpr size_t n = 256;
    fr result[n];
    fr expected[n];
    for (size_t i = 0; i < n; ++i) {
        result[i] = fr::random_element();
        fr::__copy(result[i], expected[i]);
    }

    evaluation_domain domain = evaluation_domain(n);
    domain.compute_lookup_table();
    polynomial_arithmetic::fft(result, domain);
    polynomial_arithmetic::ifft(result, domain);

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ((result[i] == expected[i]), true);
    }
}

TEST(polynomials, split_polynomial_fft_ifft_consistency)
{
    constexpr size_t n = 256;
    constexpr size_t num_poly = 4;
    fr result[num_poly][n];
    fr expected[num_poly][n];
    for (size_t j = 0; j < num_poly; j++) {
        for (size_t i = 0; i < n; ++i) {
            result[j][i] = fr::random_element();
            fr::__copy(result[j][i], expected[j][i]);
        }
    }

    evaluation_domain domain = evaluation_domain(num_poly * n);
    domain.compute_lookup_table();

    std::vector<fr*> coeffs_vec;
    for (size_t j = 0; j < num_poly; j++) {
        coeffs_vec.push_back(result[j]);
    }
    polynomial_arithmetic::fft(coeffs_vec, domain);
    polynomial_arithmetic::ifft(coeffs_vec, domain);

    for (size_t j = 0; j < num_poly; j++) {
        for (size_t i = 0; i < n; ++i) {
            EXPECT_EQ((result[j][i] == expected[j][i]), true);
        }
    }
}

TEST(polynomials, fft_coset_ifft_consistency)
{
    constexpr size_t n = 256;
    fr result[n];
    fr expected[n];
    for (size_t i = 0; i < n; ++i) {
        result[i] = fr::random_element();
        fr::__copy(result[i], expected[i]);
    }

    evaluation_domain domain = evaluation_domain(n);
    domain.compute_lookup_table();
    fr T0;
    T0 = domain.generator * domain.generator_inverse;
    EXPECT_EQ((T0 == fr::one()), true);

    polynomial_arithmetic::coset_fft(result, domain);
    polynomial_arithmetic::coset_ifft(result, domain);

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ((result[i] == expected[i]), true);
    }
}

TEST(polynomials, split_polynomial_fft_coset_ifft_consistency)
{
    constexpr size_t n = 256;
    constexpr size_t num_poly = 4;
    fr result[num_poly][n];
    fr expected[num_poly][n];
    for (size_t j = 0; j < num_poly; j++) {
        for (size_t i = 0; i < n; ++i) {
            result[j][i] = fr::random_element();
            fr::__copy(result[j][i], expected[j][i]);
        }
    }

    evaluation_domain domain = evaluation_domain(num_poly * n);
    domain.compute_lookup_table();

    std::vector<fr*> coeffs_vec;
    for (size_t j = 0; j < num_poly; j++) {
        coeffs_vec.push_back(result[j]);
    }
    polynomial_arithmetic::coset_fft(coeffs_vec, domain);
    polynomial_arithmetic::coset_ifft(coeffs_vec, domain);

    for (size_t j = 0; j < num_poly; j++) {
        for (size_t i = 0; i < n; ++i) {
            EXPECT_EQ((result[j][i] == expected[j][i]), true);
        }
    }
}

TEST(polynomials, fft_coset_ifft_cross_consistency)
{
    constexpr size_t n = 2;
    fr expected[n];
    fr poly_a[4 * n];
    fr poly_b[4 * n];
    fr poly_c[4 * n];

    for (size_t i = 0; i < n; ++i) {
        poly_a[i] = fr::random_element();
        fr::__copy(poly_a[i], poly_b[i]);
        fr::__copy(poly_a[i], poly_c[i]);
        expected[i] = poly_a[i] + poly_c[i];
        expected[i] += poly_b[i];
    }

    for (size_t i = n; i < 4 * n; ++i) {
        poly_a[i] = fr::zero();
        poly_b[i] = fr::zero();
        poly_c[i] = fr::zero();
    }
    evaluation_domain small_domain = evaluation_domain(n);
    evaluation_domain mid_domain = evaluation_domain(2 * n);
    evaluation_domain large_domain = evaluation_domain(4 * n);
    small_domain.compute_lookup_table();
    mid_domain.compute_lookup_table();
    large_domain.compute_lookup_table();
    polynomial_arithmetic::coset_fft(poly_a, small_domain);
    polynomial_arithmetic::coset_fft(poly_b, mid_domain);
    polynomial_arithmetic::coset_fft(poly_c, large_domain);

    for (size_t i = 0; i < n; ++i) {
        poly_a[i] = poly_a[i] + poly_c[4 * i];
        poly_a[i] = poly_a[i] + poly_b[2 * i];
    }

    polynomial_arithmetic::coset_ifft(poly_a, small_domain);

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ((poly_a[i] == expected[i]), true);
    }
}

/**
 * @brief Test function compute_lagrange_polynomial_fft() on medium domain (size 2 * n)
 */
TEST(polynomials, compute_lagrange_polynomial_fft)
{
    constexpr size_t n = 256;
    evaluation_domain small_domain = evaluation_domain(n);
    evaluation_domain mid_domain = evaluation_domain(2 * n);
    small_domain.compute_lookup_table();
    mid_domain.compute_lookup_table();
    fr l_1_coefficients[2 * n];
    fr scratch_memory[2 * n + 4];
    for (size_t i = 0; i < 2 * n; ++i) {
        l_1_coefficients[i] = fr::zero();
        scratch_memory[i] = fr::zero();
    }
    polynomial_arithmetic::compute_lagrange_polynomial_fft(l_1_coefficients, small_domain, mid_domain);

    polynomial_arithmetic::copy_polynomial(l_1_coefficients, scratch_memory, 2 * n, 2 * n);

    polynomial_arithmetic::coset_ifft(l_1_coefficients, mid_domain);

    fr z = fr::random_element();
    fr shifted_z;
    shifted_z = z * small_domain.root;
    shifted_z *= small_domain.root;

    fr eval;
    fr shifted_eval;

    eval = polynomial_arithmetic::evaluate(l_1_coefficients, shifted_z, small_domain.size);
    polynomial_arithmetic::fft(l_1_coefficients, small_domain);

    fr::__copy(scratch_memory[0], scratch_memory[2 * n]);
    fr::__copy(scratch_memory[1], scratch_memory[2 * n + 1]);
    fr::__copy(scratch_memory[2], scratch_memory[2 * n + 2]);
    fr::__copy(scratch_memory[3], scratch_memory[2 * n + 3]);
    fr* l_n_minus_one_coefficients = &scratch_memory[4];
    polynomial_arithmetic::coset_ifft(l_n_minus_one_coefficients, mid_domain);

    shifted_eval = polynomial_arithmetic::evaluate(l_n_minus_one_coefficients, z, small_domain.size);
    EXPECT_EQ((eval == shifted_eval), true);

    polynomial_arithmetic::fft(l_n_minus_one_coefficients, small_domain);

    EXPECT_EQ((l_1_coefficients[0] == fr::one()), true);

    for (size_t i = 1; i < n; ++i) {
        EXPECT_EQ((l_1_coefficients[i] == fr::zero()), true);
    }

    EXPECT_EQ(l_n_minus_one_coefficients[n - 2] == fr::one(), true);

    for (size_t i = 0; i < n; ++i) {
        if (i == (n - 2)) {
            continue;
        }
        EXPECT_EQ((l_n_minus_one_coefficients[i] == fr::zero()), true);
    }
}

/**
 * @brief Test function compute_lagrange_polynomial_fft() on large domain (size 4 * n)
 * @details Compute L_1 in monomial form by 1) compute_lagrange_polynomial_fft() then
 * 2) coset_ifft. Evaluate L_1 at the shifted random point z*ω^2. Show that this gives
 * the same result as 1) manually shifting coset FFT of L_1, then 2) calling
 * coset_ifft and evaluating the result (L_{n-1} monomial) at z.
 * Finally, verify that L_1 and L_{n-1} have indeed been computed correctly by checking
 * that they evaluate to one at the correct location and are zero elsewhere.
 */
TEST(polynomials, compute_lagrange_polynomial_fft_large_domain)
{
    constexpr size_t n = 256; // size of small_domain
    constexpr size_t M = 4;   // size of large_domain == M * n
    evaluation_domain small_domain = evaluation_domain(n);
    evaluation_domain large_domain = evaluation_domain(M * n);
    small_domain.compute_lookup_table();
    large_domain.compute_lookup_table();

    fr l_1_coefficients[M * n];
    // Scratch memory needs additional space (M*2) to allow for 'shift' later on
    fr scratch_memory[M * n + (M * 2)];
    for (size_t i = 0; i < M * n; ++i) {
        l_1_coefficients[i] = fr::zero();
        scratch_memory[i] = fr::zero();
    }
    // Compute FFT on target domain
    polynomial_arithmetic::compute_lagrange_polynomial_fft(l_1_coefficients, small_domain, large_domain);

    // Copy L_1 FFT into scratch space and shift it to get FFT of L_{n-1}
    polynomial_arithmetic::copy_polynomial(l_1_coefficients, scratch_memory, M * n, M * n);
    // Manually 'shift' L_1 FFT in scratch memory by M*2
    for (size_t i = 0; i < M * 2; ++i) {
        fr::__copy(scratch_memory[i], scratch_memory[M * n + i]);
    }
    fr* l_n_minus_one_coefficients = &scratch_memory[M * 2];

    // Recover monomial forms of L_1 and L_{n-1} (from manually shifted L_1 FFT)
    polynomial_arithmetic::coset_ifft(l_1_coefficients, large_domain);
    polynomial_arithmetic::coset_ifft(l_n_minus_one_coefficients, large_domain);

    // Compute shifted random eval point z*ω^2
    fr z = fr::random_element();
    fr shifted_z;                      // z*ω^2
    shifted_z = z * small_domain.root; // z*ω
    shifted_z *= small_domain.root;    // z*ω^2

    // Compute L_1(z_shifted) and L_{n-1}(z)
    fr eval = polynomial_arithmetic::evaluate(l_1_coefficients, shifted_z, small_domain.size);
    fr shifted_eval = polynomial_arithmetic::evaluate(l_n_minus_one_coefficients, z, small_domain.size);

    // Check L_1(z_shifted) = L_{n-1}(z)
    EXPECT_EQ((eval == shifted_eval), true);

    // Compute evaluation forms of L_1 and L_{n-1} and check that they have
    // a one in the right place and zeros elsewhere
    polynomial_arithmetic::fft(l_1_coefficients, small_domain);
    polynomial_arithmetic::fft(l_n_minus_one_coefficients, small_domain);

    EXPECT_EQ((l_1_coefficients[0] == fr::one()), true);

    for (size_t i = 1; i < n; ++i) {
        EXPECT_EQ((l_1_coefficients[i] == fr::zero()), true);
    }

    EXPECT_EQ(l_n_minus_one_coefficients[n - 2] == fr::one(), true);

    for (size_t i = 0; i < n; ++i) {
        if (i == (n - 2)) {
            continue;
        }
        EXPECT_EQ((l_n_minus_one_coefficients[i] == fr::zero()), true);
    }
}

TEST(polynomials, divide_by_pseudo_vanishing_polynomial)
{
    constexpr size_t n = 256;
    constexpr size_t n_large = 4 * n;
    fr a[4 * n];
    fr b[4 * n];
    fr c[4 * n];

    fr T0;
    for (size_t i = 0; i < n; ++i) {
        a[i] = fr::random_element();
        b[i] = fr::random_element();
        c[i] = a[i] * b[i];
        c[i].self_neg();
        T0 = a[i] * b[i];
        T0 += c[i];
    }
    for (size_t i = n; i < 4 * n; ++i) {
        a[i] = fr::zero();
        b[i] = fr::zero();
        c[i] = fr::zero();
    }

    // make the final evaluation not vanish
    // c[n-1].one();
    evaluation_domain small_domain = evaluation_domain(n);
    evaluation_domain large_domain = evaluation_domain(n_large);
    small_domain.compute_lookup_table();
    large_domain.compute_lookup_table();

    polynomial_arithmetic::ifft(a, small_domain);
    polynomial_arithmetic::ifft(b, small_domain);
    polynomial_arithmetic::ifft(c, small_domain);

    polynomial_arithmetic::coset_fft(a, large_domain);
    polynomial_arithmetic::coset_fft(b, large_domain);
    polynomial_arithmetic::coset_fft(c, large_domain);

    fr result[n_large];
    for (size_t i = 0; i < large_domain.size; ++i) {
        result[i] = a[i] * b[i];
        result[i] += c[i];
    }

    polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial({ &result[0] }, small_domain, large_domain, 1);

    polynomial_arithmetic::coset_ifft(result, large_domain);

    for (size_t i = n + 1; i < large_domain.size; ++i) {

        EXPECT_EQ((result[i] == fr::zero()), true);
    }
}

TEST(polynomials, compute_kate_opening_coefficients)
{
    // generate random polynomial F(X) = coeffs
    constexpr size_t n = 256;
    fr* coeffs = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * 2 * n));
    fr* W = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * 2 * n));
    for (size_t i = 0; i < n; ++i) {
        coeffs[i] = fr::random_element();
        coeffs[i + n] = fr::zero();
    }
    polynomial_arithmetic::copy_polynomial(coeffs, W, 2 * n, 2 * n);

    // generate random evaluation point z
    fr z = fr::random_element();

    // compute opening polynomial W(X), and evaluation f = F(z)
    fr f = polynomial_arithmetic::compute_kate_opening_coefficients(W, W, z, n);

    // validate that W(X)(X - z) = F(X) - F(z)
    // compute (X - z) in coefficient form
    fr* multiplicand = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * 2 * n));
    multiplicand[0] = -z;
    multiplicand[1] = fr::one();
    for (size_t i = 2; i < 2 * n; ++i) {
        multiplicand[i] = fr::zero();
    }

    // set F(X) = F(X) - F(z)
    coeffs[0] -= f;

    // compute fft of polynomials
    evaluation_domain domain = evaluation_domain(2 * n);
    domain.compute_lookup_table();
    polynomial_arithmetic::coset_fft(coeffs, domain);
    polynomial_arithmetic::coset_fft(W, domain);
    polynomial_arithmetic::coset_fft(multiplicand, domain);

    // validate that, at each evaluation, W(X)(X - z) = F(X) - F(z)
    fr result;
    for (size_t i = 0; i < domain.size; ++i) {
        result = W[i] * multiplicand[i];
        EXPECT_EQ((result == coeffs[i]), true);
    }

    aligned_free(coeffs);
    aligned_free(W);
    aligned_free(multiplicand);
}

TEST(polynomials, get_lagrange_evaluations)
{
    constexpr size_t n = 16;

    evaluation_domain domain = evaluation_domain(n);
    domain.compute_lookup_table();
    fr z = fr::random_element();

    polynomial_arithmetic::lagrange_evaluations evals = polynomial_arithmetic::get_lagrange_evaluations(z, domain, 1);

    fr vanishing_poly[2 * n];
    fr l_1_poly[n];
    fr l_n_minus_1_poly[n];

    for (size_t i = 0; i < n; ++i) {
        l_1_poly[i] = fr::zero();
        l_n_minus_1_poly[i] = fr::zero();
        vanishing_poly[i] = fr::zero();
    }
    l_1_poly[0] = fr::one();
    l_n_minus_1_poly[n - 2] = fr::one();

    fr n_mont{ n, 0, 0, 0 };
    n_mont.self_to_montgomery_form();
    vanishing_poly[n - 1] = n_mont * domain.root;

    polynomial_arithmetic::ifft(l_1_poly, domain);
    polynomial_arithmetic::ifft(l_n_minus_1_poly, domain);
    polynomial_arithmetic::ifft(vanishing_poly, domain);

    fr l_1_expected;
    fr l_n_minus_1_expected;
    fr vanishing_poly_expected;
    l_1_expected = polynomial_arithmetic::evaluate(l_1_poly, z, n);
    l_n_minus_1_expected = polynomial_arithmetic::evaluate(l_n_minus_1_poly, z, n);
    vanishing_poly_expected = polynomial_arithmetic::evaluate(vanishing_poly, z, n);
    EXPECT_EQ((evals.l_start == l_1_expected), true);
    EXPECT_EQ((evals.l_end == l_n_minus_1_expected), true);
    EXPECT_EQ((evals.vanishing_poly == vanishing_poly_expected), true);
}

TEST(polynomials, barycentric_weight_evaluations)
{
    constexpr size_t n = 16;

    evaluation_domain domain(n);

    std::vector<fr> poly(n);
    std::vector<fr> barycentric_poly(n);

    for (size_t i = 0; i < n / 2; ++i) {
        poly[i] = fr::random_element();
        barycentric_poly[i] = poly[i];
    }
    for (size_t i = n / 2; i < n; ++i) {
        poly[i] = fr::zero();
        barycentric_poly[i] = poly[i];
    }
    fr evaluation_point = fr{ 2, 0, 0, 0 }.to_montgomery_form();

    fr result =
        polynomial_arithmetic::compute_barycentric_evaluation(&barycentric_poly[0], n / 2, evaluation_point, domain);

    domain.compute_lookup_table();

    polynomial_arithmetic::ifft(&poly[0], domain);

    fr expected = polynomial_arithmetic::evaluate(&poly[0], evaluation_point, n);

    EXPECT_EQ((result == expected), true);
}

TEST(polynomials, divide_by_vanishing_polynomial)
{
    // generate mock polys A(X), B(X), C(X)
    // A(X)B(X) - C(X) = 0 mod Z_H'(X)
    // A(X)B(X) - C(X) = 0 mod Z_H(X)

    constexpr size_t n = 16;

    polynomial A(2 * n);
    polynomial B(2 * n);
    polynomial C(2 * n);

    for (size_t i = 0; i < 13; ++i) {
        A[i] = fr::random_element();
        B[i] = fr::random_element();
        C[i] = A[i] * B[i];
    }
    for (size_t i = 13; i < 16; ++i) {
        A[i] = 1;
        B[i] = 2;
        C[i] = 3;
    }

    evaluation_domain small_domain(n);
    evaluation_domain large_domain(2 * n);

    small_domain.compute_lookup_table();
    large_domain.compute_lookup_table();

    A.ifft(small_domain);
    B.ifft(small_domain);
    C.ifft(small_domain);

    fr z = fr::random_element();
    fr a_eval = A.evaluate(z, n);
    fr b_eval = B.evaluate(z, n);
    fr c_eval = C.evaluate(z, n);

    A.coset_fft(large_domain);
    B.coset_fft(large_domain);
    C.coset_fft(large_domain);

    // compute A(X) * B(X) - C(X)
    polynomial R(2 * n);

    polynomial_arithmetic::mul(&A[0], &B[0], &R[0], large_domain);
    polynomial_arithmetic::sub(&R[0], &C[0], &R[0], large_domain);

    polynomial R_copy(2 * n);
    R_copy = R;

    polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial({ &R[0] }, small_domain, large_domain, 3);
    R.coset_ifft(large_domain);

    fr r_eval = R.evaluate(z, 2 * n);

    fr Z_H_eval = (z.pow(16) - 1) / ((z - small_domain.root_inverse) * (z - small_domain.root_inverse.sqr()) *
                                     (z - small_domain.root_inverse * small_domain.root_inverse.sqr()));

    fr lhs = a_eval * b_eval - c_eval;
    fr rhs = r_eval * Z_H_eval;
    EXPECT_EQ(lhs, rhs);

    polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial({ &R_copy[0] }, small_domain, large_domain, 0);
    R_copy.coset_ifft(large_domain);

    r_eval = R_copy.evaluate(z, 2 * n);
    fr Z_H_vanishing_eval = (z.pow(16) - 1);
    rhs = r_eval * Z_H_vanishing_eval;
    EXPECT_EQ((lhs == rhs), false);
}

TEST(polynomials, partial_fft_serial)
{
    constexpr size_t n = 2;
    fr poly_eval[4 * n];
    fr poly_partial_fft[4 * n];

    evaluation_domain large_domain = evaluation_domain(4 * n);
    large_domain.compute_lookup_table();

    for (size_t i = 0; i < 4 * n; ++i) {
        poly_eval[i] = fr::random_element();
        poly_partial_fft[i] = 0;
    }

    polynomial_arithmetic::partial_fft_serial(poly_eval, poly_partial_fft, large_domain);

    fr eval_point = fr::random_element();
    fr expected = polynomial_arithmetic::compute_barycentric_evaluation(poly_eval, 4 * n, eval_point, large_domain);

    fr inner_poly_eval = 0;
    fr x_pow_4n = eval_point.pow(static_cast<uint64_t>(4 * n));
    fr x_pow_4 = eval_point.pow(4);
    fr x_pow_3 = eval_point.pow(3);
    fr x_pow_2 = eval_point.pow(2);
    fr root = large_domain.root;
    fr root_pow = 1;
    fr result = 0;

    for (size_t i = 0; i < n; ++i) {
        inner_poly_eval = poly_partial_fft[i] + poly_partial_fft[n + i] * eval_point +
                          poly_partial_fft[2 * n + i] * x_pow_2 + poly_partial_fft[3 * n + i] * x_pow_3;
        root_pow = root.pow(static_cast<uint64_t>(4 * i));
        result += (inner_poly_eval / (x_pow_4 - root_pow));
    }
    result *= (x_pow_4n - 1);
    result /= large_domain.size;

    EXPECT_EQ(result, expected);
}

TEST(polynomials, partial_fft_parallel)
{
    constexpr size_t n = 2;
    fr poly_eval[4 * n];

    evaluation_domain large_domain = evaluation_domain(4 * n);
    large_domain.compute_lookup_table();

    for (size_t i = 0; i < 4 * n; ++i) {
        poly_eval[i] = fr::random_element();
    }

    fr eval_point = fr::random_element();
    fr expected = polynomial_arithmetic::compute_barycentric_evaluation(poly_eval, 4 * n, eval_point, large_domain);

    polynomial_arithmetic::partial_fft(poly_eval, large_domain);

    fr inner_poly_eval = 0;
    fr x_pow_4n = eval_point.pow(static_cast<uint64_t>(4 * n));
    fr x_pow_4 = eval_point.pow(4);
    fr x_pow_3 = eval_point.pow(3);
    fr x_pow_2 = eval_point.pow(2);
    fr root = large_domain.root;
    fr root_pow = 1;
    fr result = 0;

    for (size_t i = 0; i < n; ++i) {
        inner_poly_eval = poly_eval[i] + poly_eval[n + i] * eval_point + poly_eval[2 * n + i] * x_pow_2 +
                          poly_eval[3 * n + i] * x_pow_3;
        root_pow = root.pow(static_cast<uint64_t>(4 * i));
        result += (inner_poly_eval / (x_pow_4 - root_pow));
    }
    result *= (x_pow_4n - 1);
    result /= large_domain.size;

    EXPECT_EQ(result, expected);
}

TEST(polynomials, partial_coset_fft_output)
{
    constexpr size_t n = 64;
    fr poly_coset_fft[4 * n];
    fr poly_coset_fft_copy[4 * n];

    evaluation_domain large_domain = evaluation_domain(4 * n);
    large_domain.compute_lookup_table();
    evaluation_domain small_domain = evaluation_domain(n);
    small_domain.compute_lookup_table();

    for (size_t i = 0; i < 4 * n; ++i) {
        poly_coset_fft[i] = fr::random_element();
        poly_coset_fft_copy[i] = poly_coset_fft[i];
    }

    // Compute R_{i,s} = \sum_{k=0}^{3} Y_{i + kn} . ω^{(i + kn)(s + 1)}
    polynomial_arithmetic::partial_fft(poly_coset_fft_copy, large_domain);

    // Compute R'_{i,s} = (g^{s - 3} / (4 * ω^{4i})) . R_{i,s}
    fr constant = (large_domain.generator_inverse.pow(4) * large_domain.four_inverse);
    polynomial_arithmetic::partial_fft(poly_coset_fft, large_domain, constant, true);

    for (size_t i = 0; i < n; ++i) {
        fr current_root = small_domain.root_inverse.pow(i);
        fr multiplicand = constant * current_root;
        for (size_t s = 0; s < 4; ++s) {
            multiplicand *= large_domain.generator;
            EXPECT_EQ(poly_coset_fft_copy[(3 - s) * n + i] * multiplicand, poly_coset_fft[(3 - s) * n + i]);
        }
    }
}

TEST(polynomials, partial_coset_fft)
{
    constexpr size_t n = 64;
    fr poly_coset_fft[4 * n];

    evaluation_domain large_domain = evaluation_domain(4 * n);
    large_domain.compute_lookup_table();
    evaluation_domain small_domain = evaluation_domain(n);
    small_domain.compute_lookup_table();

    for (size_t i = 0; i < n; ++i) {
        poly_coset_fft[i] = fr::random_element();
        poly_coset_fft[i + n] = 0;
        poly_coset_fft[i + 2 * n] = 0;
        poly_coset_fft[i + 3 * n] = 0;
    }

    polynomial_arithmetic::coset_fft(poly_coset_fft, large_domain);

    fr zeta = fr::random_element();
    fr expected = polynomial_arithmetic::evaluate_from_fft(poly_coset_fft, large_domain, zeta, small_domain);

    // Compute R'_{i,s} = (g^{s - 3} / (4 * ω^{4i})) . R_{i,s}
    fr constant = (large_domain.generator_inverse.pow(4) * large_domain.four_inverse);
    polynomial_arithmetic::partial_fft(poly_coset_fft, large_domain, constant, true);

    fr zeta_by_g_four = (zeta * large_domain.generator_inverse).pow(4);
    fr numerator = zeta_by_g_four.pow(n) - 1;
    fr result = 0;

    for (size_t i = 0; i < n; ++i) {
        fr current_root = small_domain.root_inverse.pow(i);
        fr internal_term = 0;
        fr multiplicand = 1;
        fr denominator = (zeta_by_g_four * current_root - 1);
        for (size_t s = 0; s < 4; ++s) {
            internal_term += (poly_coset_fft[s * n + i] * multiplicand);
            multiplicand *= zeta;
        }
        result += (internal_term / denominator);
    }
    result *= (numerator / n);

    EXPECT_EQ(result, expected);
}

TEST(polynomials, partial_coset_fft_evaluation)
{
    constexpr size_t n = 64;
    fr poly_coset_fft[4 * n];

    evaluation_domain large_domain = evaluation_domain(4 * n);
    large_domain.compute_lookup_table();
    evaluation_domain small_domain = evaluation_domain(n);
    small_domain.compute_lookup_table();

    for (size_t i = 0; i < 4 * n; ++i) {
        poly_coset_fft[i] = fr::random_element();
    }

    fr zeta = fr::random_element();
    fr expected = polynomial_arithmetic::compute_barycentric_evaluation(
        poly_coset_fft, 4 * n, zeta * large_domain.generator_inverse, large_domain);

    // Compute R'_{i,s} = (g^{s - 3} / (4 * ω^{4i})) . R_{i,s}
    fr constant = (large_domain.generator_inverse.pow(4) * large_domain.four_inverse);
    polynomial_arithmetic::partial_fft(poly_coset_fft, large_domain, constant, true);

    fr zeta_by_g_four = (zeta * large_domain.generator_inverse).pow(4);

    fr result = 0, multiplicand = 1;
    for (size_t s = 0; s < 4; ++s) {
        fr local_eval = polynomial_arithmetic::compute_barycentric_evaluation(
            &poly_coset_fft[s * n], n, zeta_by_g_four, small_domain);
        result += (local_eval * multiplicand);
        multiplicand *= zeta;
    }

    EXPECT_EQ(result, expected);
}

TEST(polynomials, linear_poly_product)
{
    constexpr size_t n = 64;
    fr roots[n];

    fr z = fr::random_element();
    fr expected = 1;
    for (size_t i = 0; i < n; ++i) {
        roots[i] = fr::random_element();
        expected *= (z - roots[i]);
    }

    fr dest[n + 1];
    polynomial_arithmetic::compute_linear_polynomial_product(roots, dest, n);
    fr result = polynomial_arithmetic::evaluate(dest, z, n + 1);

    EXPECT_EQ(result, expected);
}

TEST(polynomials, fft_linear_poly_product)
{
    constexpr size_t n = 60;
    fr roots[n];

    fr z = fr::random_element();
    fr expected = 1;
    for (size_t i = 0; i < n; ++i) {
        roots[i] = fr::random_element();
        expected *= (z - roots[i]);
    }

    constexpr size_t log2_n = static_cast<size_t>(numeric::get_msb(n));
    constexpr size_t N = static_cast<size_t>(1 << (log2_n + 1));
    evaluation_domain domain = evaluation_domain(N);
    domain.compute_lookup_table();

    fr dest[N];
    polynomial_arithmetic::fft_linear_polynomial_product(roots, dest, n, domain);
    fr result = polynomial_arithmetic::compute_barycentric_evaluation(dest, N, z, domain);

    fr dest_coset[N];
    fr z_by_g = z * domain.generator_inverse;
    polynomial_arithmetic::fft_linear_polynomial_product(roots, dest_coset, n, domain, true);
    fr result1 = polynomial_arithmetic::compute_barycentric_evaluation(dest_coset, N, z_by_g, domain);

    fr coeffs[n + 1];
    polynomial_arithmetic::compute_linear_polynomial_product(roots, coeffs, n);
    fr result2 = polynomial_arithmetic::evaluate(coeffs, z, n + 1);

    EXPECT_EQ(result, expected);
    EXPECT_EQ(result1, expected);
    EXPECT_EQ(result2, expected);
}

TEST(polynomials, compute_interpolation)
{
    constexpr size_t n = 100;
    fr src[n], poly[n], x[n];

    for (size_t i = 0; i < n; ++i) {
        poly[i] = fr::random_element();
    }

    for (size_t i = 0; i < n; ++i) {
        x[i] = fr::random_element();
        src[i] = polynomial_arithmetic::evaluate(poly, x[i], n);
    }
    polynomial_arithmetic::compute_interpolation(src, src, x, n);

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ(src[i], poly[i]);
    }
}

TEST(polynomials, compute_efficient_interpolation)
{
    constexpr size_t n = 250;
    fr src[n], poly[n], x[n];

    for (size_t i = 0; i < n; ++i) {
        poly[i] = fr::random_element();
    }

    for (size_t i = 0; i < n; ++i) {
        x[i] = fr::random_element();
        src[i] = polynomial_arithmetic::evaluate(poly, x[i], n);
    }
    polynomial_arithmetic::compute_efficient_interpolation(src, src, x, n);

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ(src[i], poly[i]);
    }
}

TEST(polynomials, interpolation_constructor_single)
{
    auto root = std::array{ fr(3) };
    auto eval = std::array{ fr(4) };
    polynomial t(root, eval);
    ASSERT_EQ(t.size(), 1);
    ASSERT_EQ(t[0], eval[0]);
}

TEST(polynomials, interpolation_constructor)
{
    constexpr size_t N = 32;
    std::array<fr, N> roots;
    std::array<fr, N> evaluations;
    for (size_t i = 0; i < N; ++i) {
        roots[i] = fr::random_element();
        evaluations[i] = fr::random_element();
    }

    auto roots_copy(roots);
    auto evaluations_copy(evaluations);

    polynomial interpolated(roots, evaluations);

    ASSERT_EQ(interpolated.size(), N);
    ASSERT_EQ(roots, roots_copy);
    ASSERT_EQ(evaluations, evaluations_copy);

    for (size_t i = 0; i < N; ++i) {
        fr eval = interpolated.evaluate(roots[i]);
        ASSERT_EQ(eval, evaluations[i]);
    }
}

TEST(polynomials, evaluate_mle)
{
    auto test_case = [](size_t N) {
        auto& engine = numeric::random::get_debug_engine();
        const size_t m = numeric::get_msb(N);
        EXPECT_EQ(N, 1 << m);
        polynomial poly(N);
        for (size_t i = 1; i < N - 1; ++i) {
            poly[i] = fr::random_element(&engine);
        }
        poly[N - 1] = fr::zero();

        EXPECT_TRUE(poly[0].is_zero());

        // sample u = (u₀,…,uₘ₋₁)
        std::vector<fr> u(m);
        for (size_t l = 0; l < m; ++l) {
            u[l] = fr::random_element(&engine);
        }

        std::vector<fr> lagrange_evals(N, fr(1));
        for (size_t i = 0; i < N; ++i) {
            auto& coef = lagrange_evals[i];
            for (size_t l = 0; l < m; ++l) {
                size_t mask = (1 << l);
                if ((i & mask) == 0) {
                    coef *= (fr(1) - u[l]);
                } else {
                    coef *= u[l];
                }
            }
        }

        // check eval by computing scalar product between
        // lagrange evaluations and coefficients
        fr real_eval(0);
        for (size_t i = 0; i < N; ++i) {
            real_eval += poly[i] * lagrange_evals[i];
        }
        fr computed_eval = poly.evaluate_mle(u);
        EXPECT_EQ(real_eval, computed_eval);

        // also check shifted eval
        fr real_eval_shift(0);
        for (size_t i = 1; i < N; ++i) {
            real_eval_shift += poly[i] * lagrange_evals[i - 1];
        }
        fr computed_eval_shift = poly.evaluate_mle(u, true);
        EXPECT_EQ(real_eval_shift, computed_eval_shift);
    };
    test_case(32);
    test_case(4);
    test_case(2);
}

TEST(polynomials, factor_roots)
{
    constexpr size_t N = 32;

    auto test_case = [](size_t NUM_ZERO_ROOTS, size_t NUM_NON_ZERO_ROOTS) {
        const size_t NUM_ROOTS = NUM_NON_ZERO_ROOTS + NUM_ZERO_ROOTS;

        polynomial poly(N);
        for (size_t i = NUM_ZERO_ROOTS; i < N; ++i) {
            poly[i] = fr::random_element();
        }

        // sample a root r, and compute p(r)/r^N for each non-zero root r
        std::vector<fr> non_zero_roots(NUM_NON_ZERO_ROOTS);
        std::vector<fr> non_zero_evaluations(NUM_NON_ZERO_ROOTS);
        for (size_t i = 0; i < NUM_NON_ZERO_ROOTS; ++i) {
            const auto root = fr::random_element();
            non_zero_roots[i] = root;
            const auto root_pow = root.pow(NUM_ZERO_ROOTS);
            non_zero_evaluations[i] = poly.evaluate(root) / root_pow;
        }

        std::vector<fr> roots(NUM_ROOTS);
        for (size_t i = 0; i < NUM_ZERO_ROOTS; ++i) {
            roots[i] = fr::zero();
        }
        for (size_t i = 0; i < NUM_NON_ZERO_ROOTS; ++i) {
            roots[NUM_ZERO_ROOTS + i] = non_zero_roots[i];
        }

        if (NUM_NON_ZERO_ROOTS > 0) {
            polynomial interpolated(non_zero_roots, non_zero_evaluations);
            EXPECT_EQ(interpolated.size(), NUM_NON_ZERO_ROOTS);
            for (size_t i = 0; i < NUM_NON_ZERO_ROOTS; ++i) {
                poly[NUM_ZERO_ROOTS + i] -= interpolated[i];
            }
        }

        // Sanity check that all roots are actually roots
        for (size_t i = 0; i < NUM_ROOTS; ++i) {
            EXPECT_EQ(poly.evaluate(roots[i]), fr::zero()) << i;
        }

        polynomial quotient(poly);
        quotient.factor_roots(roots);

        // check that (t-r)q(t) == p(t)
        fr t = fr::random_element();
        fr roots_eval = polynomial_arithmetic::compute_linear_polynomial_product_evaluation(roots.data(), t, NUM_ROOTS);
        fr q_t = quotient.evaluate(t, N - NUM_ROOTS);
        fr p_t = poly.evaluate(t, N);
        EXPECT_EQ(roots_eval * q_t, p_t);

        for (size_t i = N - NUM_ROOTS; i < N; ++i) {
            EXPECT_EQ(quotient.coefficients_[i], fr::zero());
        }
        if (NUM_ROOTS == 0) {
            EXPECT_EQ(poly, quotient);
        }
        if (NUM_ROOTS == 1) {
            polynomial quotient_single(poly);
            quotient_single.factor_roots(roots[0]);
            EXPECT_EQ(quotient_single, quotient);
        }
    };
    test_case(0, 0);
    test_case(0, 1);
    test_case(1, 0);
    test_case(1, 1);
    test_case(2, 0);
    test_case(0, 2);
    test_case(3, 6);
}

TEST(polynomials, move_construct_and_assign)
{
    // construct a poly with some arbitrary data
    size_t num_coeffs = 64;
    polynomial polynomial_a(num_coeffs);
    for (auto& coeff : polynomial_a) {
        coeff = fr::random_element();
    }

    // construct a new poly from the original via the move constructor
    polynomial polynomial_b(std::move(polynomial_a));

    // verifiy that source poly is appropriately destroyed
    EXPECT_EQ(polynomial_a.coefficients_, nullptr);
    EXPECT_EQ(polynomial_a.size(), 0);
    EXPECT_EQ(polynomial_a.mapped_, false);

    // construct another poly; this will also use the move constructor!
    auto polynomial_c = std::move(polynomial_b);

    // verifiy that source poly is appropriately destroyed
    EXPECT_EQ(polynomial_b.coefficients_, nullptr);
    EXPECT_EQ(polynomial_b.size(), 0);
    EXPECT_EQ(polynomial_b.mapped_, false);

    // define a poly with some arbitrary coefficients
    polynomial polynomial_d(num_coeffs);
    for (auto& coeff : polynomial_d) {
        coeff = fr::random_element();
    }

    // reset its data using move assignment
    polynomial_d = std::move(polynomial_c);

    // verifiy that source poly is appropriately destroyed
    EXPECT_EQ(polynomial_c.coefficients_, nullptr);
    EXPECT_EQ(polynomial_c.size(), 0);
    EXPECT_EQ(polynomial_c.mapped_, false);
}

TEST(polynomials, default_construct_then_assign)
{
    // construct an arbitrary but non-empty polynomial
    size_t num_coeffs = 64;
    polynomial interesting_poly(num_coeffs);
    for (auto& coeff : interesting_poly) {
        coeff = fr::random_element();
    }

    // construct an empty poly via the default constructor
    polynomial poly;

    EXPECT_EQ(poly.is_empty(), true);

    // fill the empty poly using the assignment operator
    poly = interesting_poly;

    // coefficients and size should be equal in value
    for (size_t i = 0; i < num_coeffs; ++i) {
        EXPECT_EQ(poly[i], interesting_poly[i]);
    }
    EXPECT_EQ(poly.size(), interesting_poly.size());
}