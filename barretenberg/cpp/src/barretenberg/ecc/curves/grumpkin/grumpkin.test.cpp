#include "grumpkin.hpp"
#include <chrono>
#include <gtest/gtest.h>

using namespace bb;

TEST(grumpkin, CheckB)
{
    auto b = grumpkin::g1::curve_b;
    fr seventeen = 17;
    EXPECT_EQ(seventeen, -b);
}

TEST(grumpkin, RandomElement)
{
    grumpkin::g1::element result = grumpkin::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(grumpkin, RandomAffineElement)
{
    grumpkin::g1::affine_element result = grumpkin::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(grumpkin, Eq)
{
    grumpkin::g1::element a = grumpkin::g1::element::random_element();
    grumpkin::g1::element b = a.normalize();

    EXPECT_EQ(a == b, true);
    EXPECT_EQ(a == a, true);

    b.self_set_infinity();

    EXPECT_EQ(a == b, false);
    grumpkin::g1::element c = grumpkin::g1::element::random_element();

    EXPECT_EQ(a == c, false);

    a.self_set_infinity();

    EXPECT_EQ(a == b, true);
}

TEST(grumpkin, CheckGroupModulus)
{
    // grumpkin::g1::affine_element expected = grumpkin::g1::affine_one;
    grumpkin::fr exponent = -grumpkin::fr(1);
    grumpkin::g1::element result = grumpkin::g1::one * exponent;
    result += grumpkin::g1::one;
    result += grumpkin::g1::one;
    EXPECT_EQ(result.on_curve(), true);
    EXPECT_EQ(result == grumpkin::g1::one, true);
}

TEST(grumpkin, AddExceptionTestInfinity)
{
    grumpkin::g1::element lhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element rhs;
    grumpkin::g1::element result;

    rhs = -lhs;

    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    grumpkin::g1::element rhs_b;
    rhs_b = rhs;
    rhs_b.self_set_infinity();

    result = lhs + rhs_b;

    EXPECT_EQ(lhs == result, true);

    lhs.self_set_infinity();
    result = lhs + rhs;

    EXPECT_EQ(rhs == result, true);
}

TEST(grumpkin, AddExceptionTestDbl)
{
    grumpkin::g1::element lhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element rhs;
    rhs = lhs;

    grumpkin::g1::element result;
    grumpkin::g1::element expected;

    result = lhs + rhs;
    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, AddDblConsistency)
{
    grumpkin::g1::element a = grumpkin::g1::element::random_element();
    grumpkin::g1::element b = grumpkin::g1::element::random_element();

    grumpkin::g1::element c;
    grumpkin::g1::element d;
    grumpkin::g1::element add_result;
    grumpkin::g1::element dbl_result;

    c = a + b;
    b = -b;
    d = a + b;

    add_result = c + d;
    dbl_result = a.dbl();

    EXPECT_EQ(add_result == dbl_result, true);
}

TEST(grumpkin, AddDblConsistencyRepeated)
{
    grumpkin::g1::element a = grumpkin::g1::element::random_element();
    grumpkin::g1::element b;
    grumpkin::g1::element c;
    grumpkin::g1::element d;
    grumpkin::g1::element e;

    grumpkin::g1::element result;
    grumpkin::g1::element expected;

    b = a.dbl(); // b = 2a
    c = b.dbl(); // c = 4a

    d = a + b;      // d = 3a
    e = a + c;      // e = 5a
    result = d + e; // result = 8a

    expected = c.dbl(); // expected = 8a

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, MixedAddExceptionTestInfinity)
{
    grumpkin::g1::element lhs = grumpkin::g1::one;
    grumpkin::g1::affine_element rhs = grumpkin::g1::element::random_element();
    grumpkin::fq::__copy(rhs.x, lhs.x);
    lhs.y = -rhs.y;

    grumpkin::g1::element result;
    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    lhs.self_set_infinity();
    result = lhs + rhs;
    grumpkin::g1::element rhs_c;
    rhs_c = grumpkin::g1::element(rhs);

    EXPECT_EQ(rhs_c == result, true);
}

TEST(grumpkin, MixedAddExceptionTestDbl)
{
    grumpkin::g1::affine_element rhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element lhs;
    lhs = grumpkin::g1::element(rhs);

    grumpkin::g1::element result;
    grumpkin::g1::element expected;
    result = lhs + rhs;

    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, AddMixedAddConsistencyCheck)
{
    grumpkin::g1::affine_element rhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element lhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element rhs_b;
    rhs_b = grumpkin::g1::element(rhs);

    grumpkin::g1::element add_result;
    grumpkin::g1::element mixed_add_result;
    add_result = lhs + rhs_b;
    mixed_add_result = lhs + rhs;

    EXPECT_EQ(add_result == mixed_add_result, true);
}

TEST(grumpkin, OnCurve)
{
    for (size_t i = 0; i < 100; ++i) {
        grumpkin::g1::element test = grumpkin::g1::element::random_element();
        EXPECT_EQ(test.on_curve(), true);
        grumpkin::g1::affine_element affine_test = grumpkin::g1::element::random_element();
        EXPECT_EQ(affine_test.on_curve(), true);
    }
}
TEST(grumpkin, BatchNormalize)
{
    size_t num_points = 2;
    std::vector<grumpkin::g1::element> points(num_points);
    std::vector<grumpkin::g1::element> normalized(num_points);
    for (size_t i = 0; i < num_points; ++i) {
        grumpkin::g1::element a = grumpkin::g1::element::random_element();
        grumpkin::g1::element b = grumpkin::g1::element::random_element();
        points[i] = a + b;
        normalized[i] = points[i];
    }
    grumpkin::g1::element::batch_normalize(&normalized[0], num_points);

    for (size_t i = 0; i < num_points; ++i) {
        grumpkin::fq zz;
        grumpkin::fq zzz;
        grumpkin::fq result_x;
        grumpkin::fq result_y;
        zz = points[i].z.sqr();
        zzz = points[i].z * zz;
        result_x = normalized[i].x * zz;
        result_y = normalized[i].y * zzz;

        EXPECT_EQ((result_x == points[i].x), true);
        EXPECT_EQ((result_y == points[i].y), true);
    }
}

TEST(grumpkin, GroupExponentiationZeroAndOne)
{
    grumpkin::g1::affine_element result = grumpkin::g1::one * grumpkin::fr::zero();

    EXPECT_EQ(result.is_point_at_infinity(), true);

    result = grumpkin::g1::one * grumpkin::fr::one();

    EXPECT_EQ(result == grumpkin::g1::affine_one, true);
}

TEST(grumpkin, GroupExponentiationConsistencyCheck)
{
    grumpkin::fr a = grumpkin::fr::random_element();
    grumpkin::fr b = grumpkin::fr::random_element();

    grumpkin::fr c;
    c = a * b;

    grumpkin::g1::affine_element input = grumpkin::g1::affine_one;
    grumpkin::g1::affine_element result = input * a;
    result = result * b;

    grumpkin::g1::affine_element expected = input * c;

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, DeriveGenerators)
{
    constexpr size_t num_generators = 128;
    auto result = grumpkin::g1::derive_generators("test generators", num_generators);
    const auto is_unique = [&result](const grumpkin::g1::affine_element& y, const size_t j) {
        for (size_t i = 0; i < result.size(); ++i) {
            if ((i != j) && result[i] == y) {
                return false;
            }
        }
        return true;
    };

    for (size_t k = 0; k < num_generators; ++k) {
        EXPECT_EQ(is_unique(result[k], k), true);
        EXPECT_EQ(result[k].on_curve(), true);
    }
}

TEST(grumpkin, BatchMul)
{
    constexpr size_t num_points = 1024;

    std::vector<grumpkin::g1::element> points;
    for (size_t i = 0; i < num_points; ++i) {
        points.emplace_back(grumpkin::g1::element::random_element());
    }
    grumpkin::g1::element::batch_normalize(&points[0], num_points);

    std::vector<grumpkin::g1::affine_element> affine_points;
    for (size_t i = 0; i < num_points; ++i) {
        affine_points.emplace_back(points[i]);
    }
    const grumpkin::fr exponent = grumpkin::fr::random_element();

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();

    std::vector<grumpkin::g1::element> expected;
    expected.reserve(num_points);
    for (const auto& point : points) {
        expected.emplace_back((point * exponent).normalize());
    }

    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "regular mul operations: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();

    const auto result = grumpkin::g1::element::batch_mul_with_endomorphism(affine_points, exponent);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "batched mul operations: " << diff.count() << "ms" << std::endl;

    for (size_t i = 0; i < num_points; ++i) {
        EXPECT_EQ(result[i].x, expected[i].x);
        EXPECT_EQ(result[i].y, expected[i].y);
    }
}
// Checks for "bad points" in terms of sharing a y-coordinate as explained here:
// https://github.com/AztecProtocol/aztec2-internal/issues/437
TEST(grumpkin, BadPoints)
{
    auto beta = grumpkin::fr::cube_root_of_unity();
    auto beta_sqr = beta * beta;
    bool res = true;
    grumpkin::fr c(1);
    for (size_t i = 0; i < 256; i++) {
        auto val = c / (grumpkin::fr(1) + c);
        if (val == beta || val == beta_sqr) {
            res = false;
        }
        c *= grumpkin::fr(4);
    }
    EXPECT_TRUE(res);
}
