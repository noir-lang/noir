#include <gtest/gtest.h>
#include <common/mem.hpp>
#include "polynomial_arithmetic.hpp"

using namespace barretenberg;

TEST(polynomials, evaluation_domain)
{
    size_t n = 256;
    evaluation_domain domain = evaluation_domain(n);

    EXPECT_EQ(domain.size, 256UL);
    EXPECT_EQ(domain.log2_size, 8UL);
}

TEST(polynomials, domain_roots)
{
    size_t n = 256;
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
    size_t n = 16;
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

TEST(polynomials, basic_fft)
{
    size_t n = 1 << 14;
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
    size_t n = 256;
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

TEST(polynomials, fft_coset_ifft_consistency)
{
    size_t n = 256;
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

TEST(polynomials, fft_coset_ifft_cross_consistency)
{
    size_t n = 2;
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

TEST(polynomials, compute_lagrange_polynomial_fft)
{
    size_t n = 256;
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

TEST(polynomials, divide_by_pseudo_vanishing_polynomial)
{
    size_t n = 256;
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
    evaluation_domain mid_domain = evaluation_domain(4 * n);
    small_domain.compute_lookup_table();
    mid_domain.compute_lookup_table();

    polynomial_arithmetic::ifft(a, small_domain);
    polynomial_arithmetic::ifft(b, small_domain);
    polynomial_arithmetic::ifft(c, small_domain);

    polynomial_arithmetic::coset_fft(a, mid_domain);
    polynomial_arithmetic::coset_fft(b, mid_domain);
    polynomial_arithmetic::coset_fft(c, mid_domain);

    fr result[mid_domain.size];
    for (size_t i = 0; i < mid_domain.size; ++i) {
        result[i] = a[i] * b[i];
        result[i] += c[i];
    }

    polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(&result[0], small_domain, mid_domain);

    polynomial_arithmetic::coset_ifft(result, mid_domain);

    for (size_t i = n + 1; i < mid_domain.size; ++i) {

        EXPECT_EQ((result[i] == fr::zero()), true);
    }
}

TEST(polynomials, compute_kate_opening_coefficients)
{
    // generate random polynomial F(X) = coeffs
    size_t n = 256;
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

    polynomial_arithmetic::lagrange_evaluations evals = polynomial_arithmetic::get_lagrange_evaluations(z, domain);

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
    EXPECT_EQ((evals.l_1 == l_1_expected), true);
    EXPECT_EQ((evals.l_n_minus_1 == l_n_minus_1_expected), true);
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
