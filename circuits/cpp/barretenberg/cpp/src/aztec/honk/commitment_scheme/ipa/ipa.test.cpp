#include "ipa.hpp"
#include <common/mem.hpp>
#include <gtest/gtest.h>
#include "./polynomials/polynomial_arithmetic.hpp"
#include "./polynomials/polynomial.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
using namespace barretenberg;

TEST(honk_commitment_scheme, ipa_commit)
{
    constexpr size_t n = 1024;
    std::vector<barretenberg::fr> scalars(n);
    std::vector<barretenberg::g1::affine_element> points(n);

    for (size_t i = 0; i < n; i++) {
        scalars[i] = barretenberg::fr::random_element();
        points[i] = barretenberg::g1::affine_element(barretenberg::g1::element::random_element());
    }

    barretenberg::g1::element expected = points[0] * scalars[0];
    for (size_t i = 1; i < n; i++) {
        expected += points[i] * scalars[i];
    }

    InnerProductArgument<barretenberg::fr, barretenberg::fq, barretenberg::g1> newIpa;
    barretenberg::g1::element result = newIpa.commit(scalars, n, points);
    EXPECT_EQ(expected.normalize(), result.normalize());
}

TEST(honk_commitment_scheme, ipa_open)
{
    // generate a random polynomial coeff, degree needs to be a power of two
    size_t n = 1024;
    std::vector<barretenberg::fr> coeffs(n);
    for (size_t i = 0; i < n; ++i) {
        coeffs[i] = barretenberg::fr::random_element();
    }
    // generate random evaluation point x and the evaluation
    auto x = barretenberg::fr::random_element();
    auto eval = polynomial_arithmetic::evaluate(&coeffs[0], x, n);
    // generate G_vec for testing, bypassing srs
    std::vector<barretenberg::g1::affine_element> G_vec(n);
    for (size_t i = 0; i < n; ++i) {
        auto scalar = fr::random_element();
        G_vec[i] = barretenberg::g1::affine_element(barretenberg::g1::one * scalar);
    }
    InnerProductArgument<barretenberg::fr, barretenberg::fq, barretenberg::g1> newIpa;
    auto C = newIpa.commit(coeffs, n, G_vec);
    InnerProductArgument<barretenberg::fr, barretenberg::fq, barretenberg::g1>::IpaPubInput pub_input;
    pub_input.commitment = C;
    pub_input.challenge_point = x;
    pub_input.evaluation = eval;
    pub_input.poly_degree = n;
    auto aux_scalar = fr::random_element();
    pub_input.aux_generator = barretenberg::g1::one * aux_scalar;
    const size_t log_n = static_cast<size_t>(numeric::get_msb(n));
    pub_input.round_challenges = std::vector<barretenberg::fr>(log_n);
    for (size_t i = 0; i < log_n; i++) {
        pub_input.round_challenges[i] = barretenberg::fr::random_element();
    }
    auto proof = newIpa.ipa_prove(pub_input, coeffs, G_vec);
    auto result =
        InnerProductArgument<barretenberg::fr, barretenberg::fq, barretenberg::g1>::ipa_verify(proof, pub_input, G_vec);
    EXPECT_TRUE(result);
}
