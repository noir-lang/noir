#include "univariate.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/gtest.h>

using namespace bb;

template <typename FF> class UnivariateTest : public testing::Test {
  public:
    template <size_t view_length> using UnivariateView = UnivariateView<FF, view_length>;
};

using FieldTypes = testing::Types<fr>;
TYPED_TEST_SUITE(UnivariateTest, FieldTypes);

TYPED_TEST(UnivariateTest, Constructors)
{
    fr a0 = fr::random_element();
    fr a1 = fr::random_element();
    fr a2 = fr::random_element();

    Univariate<fr, 3> uni({ a0, a1, a2 });

    EXPECT_EQ(uni.value_at(0), a0);
    EXPECT_EQ(uni.value_at(1), a1);
    EXPECT_EQ(uni.value_at(2), a2);
}

TYPED_TEST(UnivariateTest, Addition)
{
    Univariate<fr, 2> f1{ { 1, 2 } };
    Univariate<fr, 2> f2{ { 3, 4 } };
    // output should be {4, 6}
    Univariate<fr, 2> expected_result{ { 4, 6 } };
    auto f1f2 = f1 + f2;
    EXPECT_EQ(f1f2, expected_result);
}

TYPED_TEST(UnivariateTest, Multiplication)
{

    Univariate<fr, 3> f1 = Univariate<fr, 2>{ { 1, 2 } }.template extend_to<3>();
    Univariate<fr, 3> f2 = Univariate<fr, 2>{ { 3, 4 } }.template extend_to<3>();
    // output should be {3, 8, 15}
    Univariate<fr, 3> expected_result{ { 3, 8, 15 } };
    Univariate<fr, 3> f1f2 = f1 * f2;
    EXPECT_EQ(f1f2, expected_result);
}

TYPED_TEST(UnivariateTest, ConstructUnivariateViewFromUnivariate)
{

    Univariate<fr, 3> f{ { 1, 2, 3 } };
    UnivariateView<fr, 2> g(f);
    EXPECT_EQ(g.value_at(0), f.value_at(0));
    EXPECT_EQ(g.value_at(1), f.value_at(1));
}

TYPED_TEST(UnivariateTest, ConstructUnivariateFromUnivariateView)
{

    Univariate<fr, 3> f{ { 1, 2, 3 } };
    UnivariateView<fr, 2> g(f);
    Univariate<fr, 2> h(g);
    EXPECT_EQ(h.value_at(0), g.value_at(0));
    EXPECT_EQ(h.value_at(1), g.value_at(1));
}

TYPED_TEST(UnivariateTest, UnivariateViewAddition)
{
    Univariate<fr, 3> f1{ { 1, 2, 3 } };
    Univariate<fr, 3> f2{ { 3, 4, 3 } };

    UnivariateView<fr, 2> g1(f1);
    UnivariateView<fr, 2> g2(f2);

    Univariate<fr, 2> expected_result{ { 4, 6 } };
    Univariate<fr, 2> result = g1 + g2;
    EXPECT_EQ(result, expected_result);

    Univariate<fr, 2> result2 = result + g1;
    Univariate<fr, 2> expected_result2{ { 5, 8 } };
    EXPECT_EQ(result2, expected_result2);
}
TYPED_TEST(UnivariateTest, UnivariateViewSubtraction)
{
    Univariate<fr, 3> f1{ { 1, 2, 3 } };
    Univariate<fr, 3> f2{ { 3, 4, 3 } };

    UnivariateView<fr, 2> g1(f1);
    UnivariateView<fr, 2> g2(f2);

    Univariate<fr, 2> expected_result{ { -2, -2 } };
    Univariate<fr, 2> result = g1 - g2;
    EXPECT_EQ(result, expected_result);

    Univariate<fr, 2> result2 = result - g1;
    Univariate<fr, 2> expected_result2{ { -3, -4 } };
    EXPECT_EQ(result2, expected_result2);
}

TYPED_TEST(UnivariateTest, UnivariateViewMultiplication)
{
    Univariate<fr, 3> f1{ { 1, 2, 3 } };
    Univariate<fr, 3> f2{ { 3, 4, 3 } };

    UnivariateView<fr, 2> g1(f1);
    UnivariateView<fr, 2> g2(f2);

    Univariate<fr, 2> expected_result{ { 3, 8 } };
    Univariate<fr, 2> result = g1 * g2;
    EXPECT_EQ(result, expected_result);

    Univariate<fr, 2> result2 = result * g1;
    Univariate<fr, 2> expected_result2{ { 3, 16 } };
    EXPECT_EQ(result2, expected_result2);
}

TYPED_TEST(UnivariateTest, Serialization)
{
    const size_t LENGTH = 4;
    std::array<fr, LENGTH> evaluations;

    for (size_t i = 0; i < LENGTH; ++i) {
        evaluations[i] = fr::random_element();
    }

    // Instantiate a Univariate from the evaluations
    auto univariate = Univariate<fr, LENGTH>(evaluations);

    // Serialize univariate to buffer
    std::vector<uint8_t> buffer = univariate.to_buffer();

    // Deserialize
    auto deserialized_univariate = Univariate<fr, LENGTH>::serialize_from_buffer(&buffer[0]);

    for (size_t i = 0; i < LENGTH; ++i) {
        EXPECT_EQ(univariate.value_at(i), deserialized_univariate.value_at(i));
    }
}

TYPED_TEST(UnivariateTest, EvaluationCustomDomain)
{
    []() {
        auto poly = Univariate<fr, 3, 1>(std::array<fr, 2>{ 1, 2 });
        EXPECT_EQ(poly.evaluate(fr(5)), fr(5));
    }();

    []() {
        auto poly = Univariate<fr, 37, 32>(std::array<fr, 5>{ 1, 11, 111, 1111, 11111 });
        EXPECT_EQ(poly.evaluate(fr(2)), fr(294330751));
    }();
}
