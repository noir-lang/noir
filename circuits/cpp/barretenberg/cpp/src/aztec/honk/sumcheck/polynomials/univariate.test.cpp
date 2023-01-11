#include "univariate.hpp"
#include "barycentric_data.hpp"
#include <ecc/curves/bn254/fr.hpp>

#include <gtest/gtest.h>
#include <numeric/random/engine.hpp>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk::sumcheck;
namespace test_univariate {

template <typename FF> class UnivariateTest : public testing::Test {
  public:
    template <size_t view_length> using UnivariateView = UnivariateView<FF, view_length>;

    // IMPROVEMENT(Cody) this is not used anywhere? Move to memeber function of U/snivariate?
    template <size_t length> Univariate<FF, length> random_univariate()
    {
        auto output = Univariate<FF, length>();
        for (size_t i = 0; i != length; ++i) {
            output.value_at(i) = FF::random_element();
        }
        return output;
    };
};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(UnivariateTest, FieldTypes);

#define UNIVARIATE_TESTS_ALIASES using FF = TypeParam;
// IMPROVEMENT: Can't make alias for Univariate<FF, _> for some reason.
// Might be convenient to solve boilerplate or repeated type aliasing
// using this or some other means.

TYPED_TEST(UnivariateTest, Constructors)
{
    UNIVARIATE_TESTS_ALIASES

    FF a0 = FF::random_element();
    FF a1 = FF::random_element();
    FF a2 = FF::random_element();

    Univariate<FF, 3> uni({ a0, a1, a2 });

    EXPECT_EQ(uni.value_at(0), a0);
    EXPECT_EQ(uni.value_at(1), a1);
    EXPECT_EQ(uni.value_at(2), a2);
}

TYPED_TEST(UnivariateTest, Addition)
{
    UNIVARIATE_TESTS_ALIASES

    Univariate<FF, 2> f1{ { 1, 2 } };
    Univariate<FF, 2> f2{ { 3, 4 } };
    // output should be {4, 6}
    Univariate<FF, 2> expected_result{ { 4, 6 } };
    auto f1f2 = f1 + f2;
    EXPECT_EQ(f1f2, expected_result);
}

TYPED_TEST(UnivariateTest, BarycentricData2to3)
{
    UNIVARIATE_TESTS_ALIASES

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

TYPED_TEST(UnivariateTest, BarycentricData5to6)
{
    UNIVARIATE_TESTS_ALIASES

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

TYPED_TEST(UnivariateTest, Multiplication)
{
    UNIVARIATE_TESTS_ALIASES

    auto barycentric = BarycentricData<FF, 2, 3>();
    Univariate<FF, 3> f1 = barycentric.extend(Univariate<FF, 2>{ { 1, 2 } });
    Univariate<FF, 3> f2 = barycentric.extend(Univariate<FF, 2>{ { 3, 4 } });
    // output should be {3, 8, 15}
    Univariate<FF, 3> expected_result{ { 3, 8, 15 } };
    Univariate<FF, 3> f1f2 = f1 * f2;
    EXPECT_EQ(f1f2, expected_result);
}

TYPED_TEST(UnivariateTest, ConstructUnivariateViewFromUnivariate)
{
    UNIVARIATE_TESTS_ALIASES

    Univariate<FF, 3> f{ { 1, 2, 3 } };
    UnivariateView<FF, 2> g(f);
    EXPECT_EQ(g.value_at(0), f.value_at(0));
    EXPECT_EQ(g.value_at(1), f.value_at(1));
}

TYPED_TEST(UnivariateTest, ConstructUnivariateFromUnivariateView)
{
    UNIVARIATE_TESTS_ALIASES

    Univariate<FF, 3> f{ { 1, 2, 3 } };
    UnivariateView<FF, 2> g(f);
    Univariate<FF, 2> h(g);
    EXPECT_EQ(h.value_at(0), g.value_at(0));
    EXPECT_EQ(h.value_at(1), g.value_at(1));
}

TYPED_TEST(UnivariateTest, UnivariateViewAddition)
{
    UNIVARIATE_TESTS_ALIASES

    Univariate<FF, 3> f1{ { 1, 2, 3 } };
    Univariate<FF, 3> f2{ { 3, 4, 3 } };

    UnivariateView<FF, 2> g1(f1);
    UnivariateView<FF, 2> g2(f2);

    Univariate<FF, 2> expected_result{ { 4, 6 } };
    Univariate<FF, 2> result = g1 + g2;
    EXPECT_EQ(result, expected_result);

    Univariate<FF, 2> result2 = result + g1;
    Univariate<FF, 2> expected_result2{ { 5, 8 } };
    EXPECT_EQ(result2, expected_result2);
}
TYPED_TEST(UnivariateTest, UnivariateViewSubtraction)
{
    UNIVARIATE_TESTS_ALIASES

    Univariate<FF, 3> f1{ { 1, 2, 3 } };
    Univariate<FF, 3> f2{ { 3, 4, 3 } };

    UnivariateView<FF, 2> g1(f1);
    UnivariateView<FF, 2> g2(f2);

    Univariate<FF, 2> expected_result{ { -2, -2 } };
    Univariate<FF, 2> result = g1 - g2;
    EXPECT_EQ(result, expected_result);

    Univariate<FF, 2> result2 = result - g1;
    Univariate<FF, 2> expected_result2{ { -3, -4 } };
    EXPECT_EQ(result2, expected_result2);
}

TYPED_TEST(UnivariateTest, UnivariateViewMultiplication)
{
    UNIVARIATE_TESTS_ALIASES

    Univariate<FF, 3> f1{ { 1, 2, 3 } };
    Univariate<FF, 3> f2{ { 3, 4, 3 } };

    UnivariateView<FF, 2> g1(f1);
    UnivariateView<FF, 2> g2(f2);

    Univariate<FF, 2> expected_result{ { 3, 8 } };
    Univariate<FF, 2> result = g1 * g2;
    EXPECT_EQ(result, expected_result);

    Univariate<FF, 2> result2 = result * g1;
    Univariate<FF, 2> expected_result2{ { 3, 16 } };
    EXPECT_EQ(result2, expected_result2);
}

TYPED_TEST(UnivariateTest, Serialization)
{
    UNIVARIATE_TESTS_ALIASES

    const size_t LENGTH = 4;
    std::array<FF, LENGTH> evaluations;

    for (size_t i = 0; i < LENGTH; ++i) {
        evaluations[i] = FF::random_element();
    }

    // Instantiate a Univariate from the evaluations
    auto univariate = Univariate<FF, LENGTH>(evaluations);

    // Serialize univariate to buffer
    std::vector<uint8_t> buffer = univariate.to_buffer();

    // Deserialize
    auto deserialized_univariate = Univariate<FF, LENGTH>::serialize_from_buffer(&buffer[0]);

    for (size_t i = 0; i < LENGTH; ++i) {
        EXPECT_EQ(univariate.value_at(i), deserialized_univariate.value_at(i));
    }
}

} // namespace test_univariate
