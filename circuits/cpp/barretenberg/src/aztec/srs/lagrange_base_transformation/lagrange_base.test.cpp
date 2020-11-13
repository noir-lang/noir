#include <gtest/gtest.h>

#include "../../ecc/curves/bn254/g1.hpp"
#include "../../ecc/curves/bn254/g2.hpp"
#include "../../ecc/curves/bn254/fq12.hpp"
#include "../../ecc/curves/bn254/pairing.hpp"
#include "../../ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp"
#include "lagrange_base.hpp"
#include "../../polynomials/polynomial.hpp"
#include "../../plonk/reference_string/file_reference_string.hpp"
#include "srs/io.hpp"

using namespace barretenberg;

TEST(lagrange_base, verify_lagrange_base_transformation)
{
    constexpr size_t degree = 64;
    // step 1: create monomial base srs
    // step 2: create lagrange base srs

    // step 3: create polynomial
    // step 4: commit to poly over both reference strings
    // step 5: very correctness
    fr x = fr::random_element();

    std::array<fr, degree> monomial_powers;
    monomial_powers[0] = 1;
    for (size_t i = 1; i < degree; ++i) {
        monomial_powers[i] = monomial_powers[i - 1] * x;
    }

    std::array<g1::affine_element, degree * 2> monomial_srs;
    for (size_t i = 0; i < degree; ++i) {
        monomial_srs[i] = g1::element(g1::affine_one * monomial_powers[i]);
    }

    std::array<g1::affine_element, degree * 2> lagrange_base_srs;
    lagrange_base::transform_srs(&monomial_srs[0], &lagrange_base_srs[0], degree);

    barretenberg::evaluation_domain domain(degree);
    domain.compute_lookup_table();

    barretenberg::polynomial test_polynomial(degree);
    test_polynomial[0] = 1;
    for (size_t i = 1; i < degree; ++i) {
        test_polynomial[i] = 0;
    }
    barretenberg::polynomial lagrange_base_polynomial(test_polynomial);

    scalar_multiplication::pippenger_runtime_state state(degree);
    
    lagrange_base_polynomial.fft(domain);
    scalar_multiplication::generate_pippenger_point_table(&monomial_srs[0], &monomial_srs[0], degree);
    scalar_multiplication::generate_pippenger_point_table(&lagrange_base_srs[0], &lagrange_base_srs[0], degree);

    g1::element expected = scalar_multiplication::pippenger(&test_polynomial[0], &monomial_srs[0], degree, state);
    g1::element result = scalar_multiplication::pippenger(&lagrange_base_polynomial[0], &lagrange_base_srs[0], degree, state);
    expected = expected.normalize();
    result = result.normalize();
    
    EXPECT_EQ(result == expected, true);
}

TEST(lagrange_base, verify_lagrange_base_transformation_on_rand_poly)
{
    constexpr size_t degree = 64;
    // step 1: create monomial base srs
    // step 2: create lagrange base srs

    // step 3: create polynomial
    // step 4: commit to poly over both reference strings
    // step 5: very correctness
    fr x = fr::random_element();

    std::array<fr, degree> monomial_powers;
    monomial_powers[0] = 1;
    for (size_t i = 1; i < degree; ++i) {
        monomial_powers[i] = monomial_powers[i - 1] * x;
    }

    std::array<g1::affine_element, degree * 2> monomial_srs;
    for (size_t i = 0; i < degree; ++i) {
        monomial_srs[i] = g1::element(g1::affine_one * monomial_powers[i]);
    }

    std::array<g1::affine_element, degree * 2> lagrange_base_srs;
    lagrange_base::transform_srs(&monomial_srs[0], &lagrange_base_srs[0], degree);

    barretenberg::evaluation_domain domain(degree);
    domain.compute_lookup_table();
    
    barretenberg::polynomial test_polynomial(degree);
    for (size_t i = 0; i < degree; ++i) {
        test_polynomial[i] = fr::random_element();
    }
    barretenberg::polynomial lagrange_base_polynomial(test_polynomial);

    scalar_multiplication::pippenger_runtime_state state(degree);

    lagrange_base_polynomial.fft(domain);
    scalar_multiplication::generate_pippenger_point_table(&monomial_srs[0], &monomial_srs[0], degree);
    scalar_multiplication::generate_pippenger_point_table(&lagrange_base_srs[0], &lagrange_base_srs[0], degree);

    g1::element expected = scalar_multiplication::pippenger(&test_polynomial[0], &monomial_srs[0], degree, state);
    g1::element result = scalar_multiplication::pippenger(&lagrange_base_polynomial[0], &lagrange_base_srs[0], degree, state);
    expected = expected.normalize();
    result = result.normalize();

    EXPECT_EQ(result == expected, true);
}

TEST(lagrange_base, verify_lagrange_base_import_srs)
{
    constexpr size_t degree = 1 << 2;
    // step 1: create monomial base srs
    // step 2: create lagrange base srs

    // step 3: create polynomial
    // step 4: commit to poly over both reference strings
    // step 5: very correctness
    auto reference_string = std::make_shared<waffle::FileReferenceString>(degree, "../srs_db");

    std::vector<g1::affine_element> monomial_srs(degree * 2);
    std::vector<g1::affine_element> lagrange_base_srs(degree * 2);
    lagrange_base::transform_srs(reference_string->get_monomials(), &lagrange_base_srs[0], degree);

    barretenberg::evaluation_domain domain(degree);
    domain.compute_lookup_table();
    barretenberg::polynomial test_polynomial(degree);
    for (size_t i = 0; i < degree; ++i) {
        test_polynomial[i] = fr::random_element();
        monomial_srs[i] = reference_string->get_monomials()[i];
    }
    barretenberg::polynomial lagrange_base_polynomial(test_polynomial);

    scalar_multiplication::pippenger_runtime_state state(degree);

    lagrange_base_polynomial.fft(domain);
    scalar_multiplication::generate_pippenger_point_table(&monomial_srs[0], &monomial_srs[0], degree);
    scalar_multiplication::generate_pippenger_point_table(&lagrange_base_srs[0], &lagrange_base_srs[0], degree);

    g1::element expected =
        scalar_multiplication::pippenger(&test_polynomial[0], &monomial_srs[0], degree, state);
    g1::element result =
        scalar_multiplication::pippenger(&lagrange_base_polynomial[0], &lagrange_base_srs[0], degree, state);
    expected = expected.normalize();
    result = result.normalize();

    EXPECT_EQ(result == expected, true);
}