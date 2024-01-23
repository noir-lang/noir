#include "secp256r1.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <gtest/gtest.h>

using namespace bb;
namespace {
auto& engine = numeric::get_debug_randomness();

constexpr uint256_t test_fq_mod(secp256r1::Secp256r1FqParams::modulus_0,
                                secp256r1::Secp256r1FqParams::modulus_1,
                                secp256r1::Secp256r1FqParams::modulus_2,
                                secp256r1::Secp256r1FqParams::modulus_3);

uint256_t get_fq_element()
{
    uint256_t res = engine.get_random_uint256();
    while (res >= test_fq_mod) {
        res -= test_fq_mod;
    }
    return res;
}
} // namespace

TEST(secp256r1, TestAdd)
{
    const size_t n = 100;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        uint256_t b_raw = get_fq_element();

        secp256r1::fq a{ a_raw.data[0], a_raw.data[1], a_raw.data[2], a_raw.data[3] };
        secp256r1::fq b{ b_raw.data[0], b_raw.data[1], b_raw.data[2], b_raw.data[3] };

        secp256r1::fq c = a + b;

        uint256_t expected = a_raw + b_raw;
        if (expected < a_raw) {
            expected -= test_fq_mod;
        }
        uint256_t result{ c.data[0], c.data[1], c.data[2], c.data[3] };
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256r1, TestSub)
{
    const size_t n = 100;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        uint256_t b_raw = get_fq_element();

        secp256r1::fq a{ a_raw.data[0], a_raw.data[1], a_raw.data[2], a_raw.data[3] };
        secp256r1::fq b{ b_raw.data[0], b_raw.data[1], b_raw.data[2], b_raw.data[3] };

        secp256r1::fq c = a - b;

        uint256_t expected = a_raw - b_raw;
        if (expected > a_raw) {
            expected += test_fq_mod;
        }
        uint256_t result{ c.data[0], c.data[1], c.data[2], c.data[3] };
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256r1, TestToMontgomeryForm)
{
    const size_t n = 10;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        secp256r1::fq montgomery_result(a_raw);

        uint512_t R = uint512_t(0, 1);
        uint512_t aR = uint512_t(a_raw) * R;
        uint256_t expected = (aR % uint512_t(test_fq_mod)).lo;

        uint256_t result{
            montgomery_result.data[0], montgomery_result.data[1], montgomery_result.data[2], montgomery_result.data[3]
        };
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256r1, TestFromMontgomeryForm)
{
    const size_t n = 100;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        secp256r1::fq b(a_raw);
        uint256_t c(b);
        EXPECT_EQ(a_raw, c);
    }
}

TEST(secp256r1, TestMul)
{
    const size_t n = 10;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        uint256_t b_raw = get_fq_element();

        secp256r1::fq a(a_raw);
        secp256r1::fq b(b_raw);
        secp256r1::fq c = (a * b);

        uint1024_t a_1024((uint512_t(a_raw)));
        uint1024_t b_1024((uint512_t(b_raw)));
        uint1024_t c_1024 = a_1024 * b_1024;
        uint1024_t cmod = c_1024 % uint1024_t(uint512_t(test_fq_mod));
        uint256_t expected = cmod.lo.lo;
        uint256_t result(c);
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256r1, TestSqr)
{
    const size_t n = 10;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();

        secp256r1::fq a(a_raw);
        secp256r1::fq c = a.sqr();

        uint512_t c_raw = uint512_t(a_raw) * uint512_t(a_raw);
        c_raw = c_raw % uint512_t(test_fq_mod);
        uint256_t expected = c_raw.lo;
        uint256_t result(c);
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256r1, TestArithmetic)
{
    secp256r1::fq a = secp256r1::fq::random_element();
    secp256r1::fq b = secp256r1::fq::random_element();

    secp256r1::fq c = (a + b) * (a - b);
    secp256r1::fq d = a.sqr() - b.sqr();
    EXPECT_EQ(c, d);
}

TEST(secp256r1, GeneratorOnCurve)
{
    secp256r1::g1::element result = secp256r1::g1::one;
    EXPECT_EQ(result.on_curve(), true);
}

TEST(secp256r1, RandomElement)
{
    secp256r1::g1::element result = secp256r1::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(secp256r1, RandomAffineElement)
{
    secp256r1::g1::affine_element result = secp256r1::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(secp256r1, Eq)
{
    secp256r1::g1::element a = secp256r1::g1::element::random_element();
    secp256r1::g1::element b = a.normalize();

    EXPECT_EQ(a == b, true);
    EXPECT_EQ(a == a, true);

    b.self_set_infinity();

    EXPECT_EQ(a == b, false);
    secp256r1::g1::element c = secp256r1::g1::element::random_element();

    EXPECT_EQ(a == c, false);

    a.self_set_infinity();

    EXPECT_EQ(a == b, true);
}

TEST(secp256r1, CheckGroupModulus)
{
    // secp256r1::g1::affine_element expected = secp256r1::g1::affine_one;
    secp256r1::fr exponent = -secp256r1::fr(1);
    secp256r1::g1::element result = secp256r1::g1::one * exponent;
    result += secp256r1::g1::one;
    result += secp256r1::g1::one;
    EXPECT_EQ(result.on_curve(), true);
    EXPECT_EQ(result == secp256r1::g1::one, true);
}

TEST(secp256r1, AddExceptionTestInfinity)
{
    secp256r1::g1::element lhs = secp256r1::g1::element::random_element();
    secp256r1::g1::element rhs;
    secp256r1::g1::element result;

    rhs = -lhs;

    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    secp256r1::g1::element rhs_b;
    rhs_b = rhs;
    rhs_b.self_set_infinity();

    result = lhs + rhs_b;

    EXPECT_EQ(lhs == result, true);

    lhs.self_set_infinity();
    result = lhs + rhs;

    EXPECT_EQ(rhs == result, true);
}

TEST(secp256r1, AddExceptionTestDbl)
{
    secp256r1::g1::element lhs = secp256r1::g1::element::random_element();
    secp256r1::g1::element rhs;
    rhs = lhs;

    secp256r1::g1::element result;
    secp256r1::g1::element expected;

    result = lhs + rhs;
    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(secp256r1, AddDblConsistency)
{
    secp256r1::g1::element a = secp256r1::g1::one; // P
    secp256r1::g1::element b = a.dbl();            // 2P

    secp256r1::g1::element c = b.dbl(); // 4P
    c = c.dbl();                        // 8P

    secp256r1::g1::element d = a + b; // 3P
    d = d + b;                        // 5P
    d = d + a;                        // 6P
    d = d + a;                        // 7P
    d = d + a;                        // 8P
    EXPECT_EQ(c, d);
}

TEST(secp256r1, AddDblConsistencyRepeated)
{
    secp256r1::g1::element a = secp256r1::g1::element::random_element();
    secp256r1::g1::element b;
    secp256r1::g1::element c;
    secp256r1::g1::element d;
    secp256r1::g1::element e;

    secp256r1::g1::element result;
    secp256r1::g1::element expected;

    b = a.dbl(); // b = 2a
    c = b.dbl(); // c = 4a

    d = a + b;      // d = 3a
    e = a + c;      // e = 5a
    result = d + e; // result = 8a

    expected = c.dbl(); // expected = 8a

    EXPECT_EQ(result == expected, true);
}

TEST(secp256r1, MixedAddExceptionTestInfinity)
{
    secp256r1::g1::element lhs = secp256r1::g1::one;
    secp256r1::g1::affine_element rhs = secp256r1::g1::element::random_element();
    secp256r1::fq::__copy(rhs.x, lhs.x);
    lhs.y = -rhs.y;

    secp256r1::g1::element result;
    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    lhs.self_set_infinity();
    result = lhs + rhs;
    secp256r1::g1::element rhs_c;
    rhs_c = secp256r1::g1::element(rhs);

    EXPECT_EQ(rhs_c == result, true);
}

TEST(secp256r1, MixedAddExceptionTestDbl)
{
    secp256r1::g1::affine_element rhs = secp256r1::g1::element::random_element();
    secp256r1::g1::element lhs;
    lhs = secp256r1::g1::element(rhs);

    secp256r1::g1::element result;
    secp256r1::g1::element expected;
    result = lhs + rhs;

    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(secp256r1, AddMixedAddConsistencyCheck)
{
    secp256r1::g1::affine_element rhs = secp256r1::g1::element::random_element();
    secp256r1::g1::element lhs = secp256r1::g1::element::random_element();
    secp256r1::g1::element rhs_b;
    rhs_b = secp256r1::g1::element(rhs);

    secp256r1::g1::element add_result;
    secp256r1::g1::element mixed_add_result;
    add_result = lhs + rhs_b;
    mixed_add_result = lhs + rhs;

    EXPECT_EQ(add_result == mixed_add_result, true);
}

TEST(secp256r1, OnCurve)
{
    for (size_t i = 0; i < 100; ++i) {
        secp256r1::g1::element test = secp256r1::g1::element::random_element();
        EXPECT_EQ(test.on_curve(), true);
        secp256r1::g1::affine_element affine_test = secp256r1::g1::element::random_element();
        EXPECT_EQ(affine_test.on_curve(), true);
    }
}
TEST(secp256r1, BatchNormalize)
{
    size_t num_points = 2;
    std::vector<secp256r1::g1::element> points(num_points);
    std::vector<secp256r1::g1::element> normalized(num_points);
    for (size_t i = 0; i < num_points; ++i) {
        secp256r1::g1::element a = secp256r1::g1::element::random_element();
        secp256r1::g1::element b = secp256r1::g1::element::random_element();
        points[i] = a + b;
        normalized[i] = points[i];
    }
    secp256r1::g1::element::batch_normalize(&normalized[0], num_points);

    for (size_t i = 0; i < num_points; ++i) {
        secp256r1::fq zz;
        secp256r1::fq zzz;
        secp256r1::fq result_x;
        secp256r1::fq result_y;
        zz = points[i].z.sqr();
        zzz = points[i].z * zz;
        result_x = normalized[i].x * zz;
        result_y = normalized[i].y * zzz;

        EXPECT_EQ((result_x == points[i].x), true);
        EXPECT_EQ((result_y == points[i].y), true);
    }
}

TEST(secp256r1, GroupExponentiationZeroAndOne)
{
    secp256r1::g1::affine_element result = secp256r1::g1::one * secp256r1::fr::zero();

    EXPECT_EQ(result.is_point_at_infinity(), true);
    secp256r1::g1::element pif = secp256r1::g1::one * secp256r1::fr::zero();

    EXPECT_EQ(result.is_point_at_infinity(), true);
    EXPECT_NE(pif, secp256r1::g1::one);

    result = secp256r1::g1::one * secp256r1::fr::one();

    EXPECT_EQ(result == secp256r1::g1::affine_one, true);
}

TEST(secp256r1, GroupExponentiationConsistencyCheck)
{
    secp256r1::fr a = secp256r1::fr::random_element();
    secp256r1::fr b = secp256r1::fr::random_element();

    secp256r1::fr c;
    c = a * b;

    secp256r1::g1::affine_element input = secp256r1::g1::affine_one;
    secp256r1::g1::affine_element result = input * a;
    result = result * b;

    secp256r1::g1::affine_element expected = input * c;

    EXPECT_EQ(result == expected, true);
}

/**
 * @brief We had an issue where we added field elements and subtracted a prime depending on the 2²⁵⁶ overflow. This
 * was incorrect. Sometimes we need to subtract the prime twice. The same is true for subtractions
 *
 */
TEST(secp256r1, AdditionSubtractionRegressionCheck)
{
    secp256r1::fq fq1(uint256_t{ 0xfffffe0000000200, 0x200fffff9ff, 0xfffffbfffffffe00, 0xfffffbff00000400 });
    secp256r1::fq fq2(uint256_t{ 0xfffffe0000000200, 0x200fffff9ff, 0xfffffbfffffffe00, 0xfffffbff00000400 });
    secp256r1::fq fq3(0);
    secp256r1::fq fq4(0);
    fq1 += secp256r1::fq(secp256r1::fq::modulus_minus_two);
    fq1 += secp256r1::fq(2);

    fq3 -= fq1;
    fq4 -= fq2;
    EXPECT_EQ(fq1 + fq1, fq2 + fq2);
    EXPECT_EQ(fq3, fq4);
}

/* TODO (#LARGE_MODULUS_AFFINE_POINT_COMPRESSION): Rewrite this test after designing point compression for p>2^255
TEST(secp256r1, derive_generators)
{
    constexpr size_t num_generators = 128;
    auto result = secp256r1::g1::derive_generators<num_generators>();

    const auto is_unique = [&result](const secp256r1::g1::affine_element& y, const size_t j) {
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
TEST(secp256r1, check_compression_constructor)
{
    secp256r1::g1::affine_element el(uint256_t(10));
    std::cout << "Affine element: " << el << std::endl;
}**/

TEST(secp256r1, MontgomeryMulBigBug)
{
    secp256r1::fr a;
    a.data[0] = 0xC5BF4F6AFF993D09;
    a.data[1] = 0xA3361BDA67E62E0E;
    a.data[2] = 0xAAAAAAAAAAAAAAAA;
    a.data[3] = 0xFFFFFFFFE38E38E3;
    secp256r1::fr a_sqr = a.sqr();
    secp256r1::fr expected(uint256_t{ 0x57abc6aa0349c084, 0x65b21b232a4cb7a5, 0x5ba781948b0fcd6e, 0xd6e9e0644bda12f7 });
    EXPECT_EQ((a_sqr == expected), true);
}
