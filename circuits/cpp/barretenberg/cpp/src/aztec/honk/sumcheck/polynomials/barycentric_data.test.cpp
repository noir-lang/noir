#include "barycentric_data.hpp"
#include <ecc/curves/bn254/fr.hpp>

#include <gtest/gtest.h>
#include <numeric/random/engine.hpp>

using namespace honk::sumcheck;
namespace test_sumcheck_polynomials {

template <class FF> class BarycentricDataTests : public testing::Test {};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(BarycentricDataTests, FieldTypes);

#define BARYCENTIC_DATA_TESTS_TYPE_ALIASES using FF = TypeParam;

TYPED_TEST(BarycentricDataTests, Extend)
{
    BARYCENTIC_DATA_TESTS_TYPE_ALIASES
    const size_t domain_size(2);
    const size_t num_evals(10);
    auto f = Univariate<FF, domain_size>({ 1, 2 });
    auto expected_result = Univariate<FF, num_evals>({ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 });
    BarycentricData<FF, domain_size, num_evals> barycentric_data;
    auto result = barycentric_data.extend(f);
    EXPECT_EQ(result, expected_result);
}

TYPED_TEST(BarycentricDataTests, Evaluate)
{
    BARYCENTIC_DATA_TESTS_TYPE_ALIASES
    const size_t domain_size(2);
    const size_t num_evals(10);
    auto f = Univariate<FF, domain_size>({ 1, 2 });
    FF u = 5;
    FF expected_result = 6;
    BarycentricData<FF, domain_size, num_evals> barycentric_data;
    auto result = barycentric_data.evaluate(f, u);
    EXPECT_EQ(result, expected_result);
}

TYPED_TEST(BarycentricDataTests, BarycentricData2to3)
{
    BARYCENTIC_DATA_TESTS_TYPE_ALIASES

    const size_t domain_size = 2;
    const size_t num_evals = 3;
    auto barycentric = BarycentricData<FF, domain_size, num_evals>();
    std::array<FF, 3> expected_big_domain{ { 0, 1, 2 } };
    std::array<FF, 2> expected_denominators{ { -1, 1 } };
    std::array<FF, 3> expected_full_numerator_values{ { 0, 0, 2 } };
    EXPECT_EQ(barycentric.big_domain, expected_big_domain);
    EXPECT_EQ(barycentric.lagrange_denominators, expected_denominators);
    EXPECT_EQ(barycentric.full_numerator_values, expected_full_numerator_values);

    // e1(X) = 1*(1-X) + 2*X = 1 + X
    Univariate<FF, 2> e1{ { 1, 2 } };
    FF u = FF::random_element();
    FF calculated_val_at_u = barycentric.evaluate(e1, u);
    EXPECT_EQ(u + 1, calculated_val_at_u);

    Univariate<FF, 3> ext1 = barycentric.extend(e1);
    Univariate<FF, 3> expected{ { 1, 2, 3 } };
    EXPECT_EQ(ext1, expected);
}

TYPED_TEST(BarycentricDataTests, BarycentricData5to6)
{
    BARYCENTIC_DATA_TESTS_TYPE_ALIASES

    const size_t domain_size = 5;
    const size_t num_evals = 6;
    auto barycentric = BarycentricData<FF, domain_size, num_evals>();

    // Note: we are able to represent a degree 4 polynomial with 5 points thus this
    // extension will succeed. It would fail for values on a polynomial of degree > 4.
    Univariate<FF, domain_size> e1{ { 1, 3, 25, 109, 321 } }; // X^4 + X^3 + 1

    Univariate<FF, num_evals> ext1 = barycentric.extend(e1);

    Univariate<FF, num_evals> expected{ { 1, 3, 25, 109, 321, 751 } };

    EXPECT_EQ(ext1, expected);
}

} // namespace test_sumcheck_polynomials