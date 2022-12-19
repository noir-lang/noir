#include "./barycentric_data.hpp"
#include "./univariate.hpp"
#include <common/mem.hpp>
#include <gtest/gtest.h>
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>
#include <span>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk::sumcheck;
namespace test_univariate {

template <typename Fr> class UnivariateTests : public testing::Test {
    template <size_t view_length> using UnivariateView = UnivariateView<Fr, view_length>;

    template <size_t length> Univariate<Fr, length> random_univariate()
    {
        auto output = Univariate<Fr, length>();
        for (size_t i = 0; i != length; ++i) {
            output.value_at(i) = Fr::random_element();
        }
        return output;
    };

  public:
    static void test_constructors()
    {
        Fr a0 = Fr::random_element();
        Fr a1 = Fr::random_element();
        Fr a2 = Fr::random_element();

        Univariate<Fr, 3> uni({ a0, a1, a2 });

        EXPECT_EQ(uni.value_at(0), a0);
        EXPECT_EQ(uni.value_at(1), a1);
        EXPECT_EQ(uni.value_at(2), a2);
    }

    static void test_addition()
    {
        Univariate<Fr, 2> f1{ { 1, 2 } };
        Univariate<Fr, 2> f2{ { 3, 4 } };
        // output should be {4, 6}
        Univariate<Fr, 2> expected_result{ { 4, 6 } };
        auto f1f2 = f1 + f2;
        EXPECT_EQ(f1f2, expected_result);
    }

    static void test_barycentric_data()
    {
        const size_t domain_size = 2;
        const size_t num_evals = 3;
        auto barycentric = BarycentricData<Fr, domain_size, num_evals>();
        std::array<Fr, 3> expected_big_domain{ { 0, 1, 2 } };
        std::array<Fr, 2> expected_denominators{ { -1, 1 } };
        std::array<Fr, 3> expected_full_numerator_values{ { 0, 0, 2 } };
        EXPECT_EQ(barycentric.big_domain, expected_big_domain);
        EXPECT_EQ(barycentric.lagrange_denominators, expected_denominators);
        EXPECT_EQ(barycentric.full_numerator_values, expected_full_numerator_values);

        // e1(X) = 1*(1-X) + 2*X = 1 + X
        Univariate<Fr, 2> e1{ { 1, 2 } };
        Fr u = Fr::random_element();
        Fr calculated_val_at_u = barycentric.evaluate(e1, u);
        EXPECT_EQ(u + 1, calculated_val_at_u);

        Univariate<Fr, 3> ext1 = barycentric.extend(e1);
        Univariate<Fr, 3> expected{ { 1, 2, 3 } };
        EXPECT_EQ(ext1, expected);
    }

    static void test_barycentric_data_extend()
    {
        const size_t domain_size = 5;
        const size_t num_evals = 6;
        auto barycentric = BarycentricData<Fr, domain_size, num_evals>();

        // Note: we are able to represent a degree 4 polynomial with 5 points thus this
        // extension will succeed. It would fail for values on a polynomial of degree > 4.
        Univariate<Fr, domain_size> e1{ { 1, 3, 25, 109, 321 } }; // X^4 + X^3 + 1

        Univariate<Fr, num_evals> ext1 = barycentric.extend(e1);

        Univariate<Fr, num_evals> expected{ { 1, 3, 25, 109, 321, 751 } };

        EXPECT_EQ(ext1, expected);
    }

    static void test_multiplication()
    {
        auto barycentric = BarycentricData<Fr, 2, 3>();
        Univariate<Fr, 3> f1 = barycentric.extend(Univariate<Fr, 2>{ { 1, 2 } });
        Univariate<Fr, 3> f2 = barycentric.extend(Univariate<Fr, 2>{ { 3, 4 } });
        // output should be {3, 8, 15}
        Univariate<Fr, 3> expected_result{ { 3, 8, 15 } };
        Univariate<Fr, 3> f1f2 = f1 * f2;
        EXPECT_EQ(f1f2, expected_result);
    }

    static void test_construct_univariate_view_from_univariate()
    {
        Univariate<Fr, 3> f{ { 1, 2, 3 } };
        UnivariateView<2> g(f);
        EXPECT_EQ(g.value_at(0), f.value_at(0));
        EXPECT_EQ(g.value_at(1), f.value_at(1));
    }

    static void test_construct_univariate_from_univariate_view()
    {
        Univariate<Fr, 3> f{ { 1, 2, 3 } };
        UnivariateView<2> g(f);
        Univariate<Fr, 2> h(g);
        EXPECT_EQ(h.value_at(0), g.value_at(0));
        EXPECT_EQ(h.value_at(1), g.value_at(1));
    }

    static void test_univariate_view_addition()
    {
        Univariate<Fr, 3> f1{ { 1, 2, 3 } };
        Univariate<Fr, 3> f2{ { 3, 4, 3 } };

        UnivariateView<2> g1(f1);
        UnivariateView<2> g2(f2);

        Univariate<Fr, 2> expected_result{ { 4, 6 } };
        Univariate<Fr, 2> result = g1 + g2;
        EXPECT_EQ(result, expected_result);

        Univariate<Fr, 2> result2 = result + g1;
        Univariate<Fr, 2> expected_result2{ { 5, 8 } };
        EXPECT_EQ(result2, expected_result2);
    }
    static void test_univariate_view_subtraction()
    {
        Univariate<Fr, 3> f1{ { 1, 2, 3 } };
        Univariate<Fr, 3> f2{ { 3, 4, 3 } };

        UnivariateView<2> g1(f1);
        UnivariateView<2> g2(f2);

        Univariate<Fr, 2> expected_result{ { -2, -2 } };
        Univariate<Fr, 2> result = g1 - g2;
        EXPECT_EQ(result, expected_result);

        Univariate<Fr, 2> result2 = result - g1;
        Univariate<Fr, 2> expected_result2{ { -3, -4 } };
        EXPECT_EQ(result2, expected_result2);
    }

    static void test_univariate_view_multiplication()
    {
        Univariate<Fr, 3> f1{ { 1, 2, 3 } };
        Univariate<Fr, 3> f2{ { 3, 4, 3 } };

        UnivariateView<2> g1(f1);
        UnivariateView<2> g2(f2);

        Univariate<Fr, 2> expected_result{ { 3, 8 } };
        Univariate<Fr, 2> result = g1 * g2;
        EXPECT_EQ(result, expected_result);

        Univariate<Fr, 2> result2 = result * g1;
        Univariate<Fr, 2> expected_result2{ { 3, 16 } };
        EXPECT_EQ(result2, expected_result2);
    }
};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(UnivariateTests, FieldTypes);

TYPED_TEST(UnivariateTests, Constructors)
{
    TestFixture::test_constructors();
}
TYPED_TEST(UnivariateTests, Addition)
{
    TestFixture::test_addition();
}
TYPED_TEST(UnivariateTests, UnivariateToView)
{
    TestFixture::test_construct_univariate_from_univariate_view();
}
TYPED_TEST(UnivariateTests, ViewToUnivariate)
{
    TestFixture::test_construct_univariate_view_from_univariate();
}
TYPED_TEST(UnivariateTests, ViewAddition)
{
    TestFixture::test_univariate_view_addition();
}
TYPED_TEST(UnivariateTests, ViewMultiplication)
{
    TestFixture::test_univariate_view_subtraction();
}
TYPED_TEST(UnivariateTests, ViewSubtraction)
{
    TestFixture::test_univariate_view_multiplication();
}
TYPED_TEST(UnivariateTests, BarycentricData)
{
    TestFixture::test_barycentric_data();
}
TYPED_TEST(UnivariateTests, BarycentricDataExtend)
{
    TestFixture::test_barycentric_data_extend();
}
TYPED_TEST(UnivariateTests, Multiplication)
{
    TestFixture::test_multiplication();
}
} // namespace test_univariate
