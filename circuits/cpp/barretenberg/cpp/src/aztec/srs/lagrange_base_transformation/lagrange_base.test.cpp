#include <common/test.hpp>

#include <ecc/curves/bn254/g1.hpp>
#include <ecc/curves/bn254/g2.hpp>
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/pippenger.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial.hpp>

#include "lagrange_base.hpp"
#include "../io.hpp"

#include <chrono>

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
    g1::element result =
        scalar_multiplication::pippenger(&lagrange_base_polynomial[0], &lagrange_base_srs[0], degree, state);
    expected = expected.normalize();
    result = result.normalize();

    EXPECT_EQ(result, expected);
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
    g1::element result =
        scalar_multiplication::pippenger(&lagrange_base_polynomial[0], &lagrange_base_srs[0], degree, state);
    expected = expected.normalize();
    result = result.normalize();

    EXPECT_EQ(result, expected);
}

TEST(lagrange_base, verify_lagrange_base_import_srs)
{
    constexpr size_t degree = 1 << 4;
    // step 1: create monomial base srs
    // step 2: create lagrange base srs

    // step 3: create polynomial
    // step 4: commit to poly over both reference strings
    // step 5: very correctness

    std::array<g1::affine_element, degree * 2> monomial_srs;
    barretenberg::io::read_transcript_g1(&monomial_srs[0], degree, "../srs_db/ignition");

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
    g1::element result =
        scalar_multiplication::pippenger(&lagrange_base_polynomial[0], &lagrange_base_srs[0], degree, state);
    expected = expected.normalize();
    result = result.normalize();

    EXPECT_EQ(result, expected);
}

void test_lagrange_transcripts_helper(const size_t degree,
                                      const std::string monomial_path,
                                      const std::string lagrange_path)
{
    auto lagrange_reference_string = scalar_multiplication::Pippenger(lagrange_path, degree, true);
    auto monomial_reference_string = scalar_multiplication::Pippenger(monomial_path, degree, false);

    barretenberg::evaluation_domain domain(degree);
    domain.compute_lookup_table();

    barretenberg::polynomial test_polynomial(degree);
    for (size_t i = 0; i < degree; ++i) {
        test_polynomial[i] = fr::random_element();
    }
    barretenberg::polynomial lagrange_base_polynomial(test_polynomial);
    lagrange_base_polynomial.fft(domain);

    g1::affine_element expected = monomial_reference_string.pippenger_unsafe(&test_polynomial[0], 0, degree);
    g1::affine_element result = lagrange_reference_string.pippenger_unsafe(&lagrange_base_polynomial[0], 0, degree);

    EXPECT_EQ(result, expected);

    g2::affine_element lagrange_g2x;
    g2::affine_element monomial_g2x;
    barretenberg::io::read_transcript_g2(lagrange_g2x, lagrange_path, true);
    barretenberg::io::read_transcript_g2(monomial_g2x, monomial_path, false);

    EXPECT_EQ(lagrange_g2x, monomial_g2x);
}

/**
 * The following tests ensure the correctness of the Lagrange transcripts downloaded using
 * `download_ignition_lagrange.sh`. These need to be run only once after you download the transcripts (IFF you want to
 * verify their correctness). Also, the `num_files` is set to 16 which checks transcripts upto size 2^16. You can change
 * it to 24 if you wish to check all the transcripts (Warning: it'll a lot of time for size > 2^{20}).
 */
HEAVY_TEST(lagrange_base, test_local_lagrange_transcripts)
{
    // Setup monomial srs
    // TODO: Not sure if this is a test that should be run every time
    // We can check a single one to see
    const size_t num_files = 4;

    for (size_t i = 0; i < num_files; i++) {
        const size_t degree = static_cast<size_t>(1 << (i + 1));
        auto begin = std::chrono::steady_clock::now();
        test_lagrange_transcripts_helper(degree, "../srs_db/ignition", "../srs_db/lagrange");
        auto end = std::chrono::steady_clock::now();

        std::cout << "Verified Lagrange transcript of size " << degree << " in "
                  << std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count() << " ms" << std::endl;
    }
}