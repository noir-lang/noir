#include "secp256k1.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <gtest/gtest.h>

using namespace bb;
namespace {
auto& engine = numeric::get_debug_randomness();

constexpr uint256_t test_fq_mod(secp256k1::Secp256k1FqParams::modulus_0,
                                secp256k1::Secp256k1FqParams::modulus_1,
                                secp256k1::Secp256k1FqParams::modulus_2,
                                secp256k1::Secp256k1FqParams::modulus_3);

uint256_t get_fq_element()
{
    uint256_t res = engine.get_random_uint256();
    while (res >= test_fq_mod) {
        res -= test_fq_mod;
    }
    return res;
}
} // namespace

TEST(secp256k1, TestAdd)
{
    const size_t n = 100;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        uint256_t b_raw = get_fq_element();

        secp256k1::fq a{ a_raw.data[0], a_raw.data[1], a_raw.data[2], a_raw.data[3] };
        secp256k1::fq b{ b_raw.data[0], b_raw.data[1], b_raw.data[2], b_raw.data[3] };

        secp256k1::fq c = a + b;

        uint256_t expected = a_raw + b_raw;
        if (expected < a_raw) {
            expected -= test_fq_mod;
        }
        uint256_t result{ c.data[0], c.data[1], c.data[2], c.data[3] };
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256k1, TestSub)
{
    const size_t n = 100;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        uint256_t b_raw = get_fq_element();

        secp256k1::fq a{ a_raw.data[0], a_raw.data[1], a_raw.data[2], a_raw.data[3] };
        secp256k1::fq b{ b_raw.data[0], b_raw.data[1], b_raw.data[2], b_raw.data[3] };

        secp256k1::fq c = a - b;

        uint256_t expected = a_raw - b_raw;
        if (expected > a_raw) {
            expected += test_fq_mod;
        }
        uint256_t result{ c.data[0], c.data[1], c.data[2], c.data[3] };
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256k1, TestToMontgomeryForm)
{
    const size_t n = 10;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        secp256k1::fq montgomery_result(a_raw);

        uint512_t R = uint512_t(0, 1);
        uint512_t aR = uint512_t(a_raw) * R;
        uint256_t expected = (aR % uint512_t(test_fq_mod)).lo;

        uint256_t result{
            montgomery_result.data[0], montgomery_result.data[1], montgomery_result.data[2], montgomery_result.data[3]
        };
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256k1, TestFromMontgomeryForm)
{
    const size_t n = 100;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        secp256k1::fq b(a_raw);
        uint256_t c(b);
        EXPECT_EQ(a_raw, c);
    }
}

TEST(secp256k1, TestMul)
{
    const size_t n = 10;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();
        uint256_t b_raw = get_fq_element();

        secp256k1::fq a(a_raw);
        secp256k1::fq b(b_raw);
        secp256k1::fq c = (a * b);

        uint1024_t a_1024((uint512_t(a_raw)));
        uint1024_t b_1024((uint512_t(b_raw)));
        uint1024_t c_1024 = a_1024 * b_1024;
        uint1024_t cmod = c_1024 % uint1024_t(uint512_t(test_fq_mod));
        uint256_t expected = cmod.lo.lo;
        uint256_t result(c);
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256k1, TestSqr)
{
    const size_t n = 10;
    for (size_t i = 0; i < n; ++i) {
        uint256_t a_raw = get_fq_element();

        secp256k1::fq a(a_raw);
        secp256k1::fq c = a.sqr();

        uint512_t c_raw = uint512_t(a_raw) * uint512_t(a_raw);
        c_raw = c_raw % uint512_t(test_fq_mod);
        uint256_t expected = c_raw.lo;
        uint256_t result(c);
        EXPECT_EQ(result, expected);
    }
}

TEST(secp256k1, TestArithmetic)
{
    secp256k1::fq a = secp256k1::fq::random_element();
    secp256k1::fq b = secp256k1::fq::random_element();

    secp256k1::fq c = (a + b) * (a - b);
    secp256k1::fq d = a.sqr() - b.sqr();
    EXPECT_EQ(c, d);
}

TEST(secp256k1, GeneratorOnCurve)
{
    secp256k1::g1::element result = secp256k1::g1::one;
    EXPECT_EQ(result.on_curve(), true);
}

TEST(secp256k1, RandomElement)
{
    secp256k1::g1::element result = secp256k1::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(secp256k1, RandomAffineElement)
{
    secp256k1::g1::affine_element result = secp256k1::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(secp256k1, Eq)
{
    secp256k1::g1::element a = secp256k1::g1::element::random_element();
    secp256k1::g1::element b = a.normalize();

    EXPECT_EQ(a == b, true);
    EXPECT_EQ(a == a, true);

    b.self_set_infinity();

    EXPECT_EQ(a == b, false);
    secp256k1::g1::element c = secp256k1::g1::element::random_element();

    EXPECT_EQ(a == c, false);

    a.self_set_infinity();

    EXPECT_EQ(a == b, true);
}

TEST(secp256k1, CheckGroupModulus)
{
    // secp256k1::g1::affine_element expected = secp256k1::g1::affine_one;
    secp256k1::fr exponent = -secp256k1::fr(1);
    secp256k1::g1::element result = secp256k1::g1::one * exponent;
    result += secp256k1::g1::one;
    result += secp256k1::g1::one;
    EXPECT_EQ(result.on_curve(), true);
    EXPECT_EQ(result == secp256k1::g1::one, true);
}

TEST(secp256k1, AddExceptionTestInfinity)
{
    secp256k1::g1::element lhs = secp256k1::g1::element::random_element();
    secp256k1::g1::element rhs;
    secp256k1::g1::element result;

    rhs = -lhs;

    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    secp256k1::g1::element rhs_b;
    rhs_b = rhs;
    rhs_b.self_set_infinity();

    result = lhs + rhs_b;

    EXPECT_EQ(lhs == result, true);

    lhs.self_set_infinity();
    result = lhs + rhs;

    EXPECT_EQ(rhs == result, true);
}

TEST(secp256k1, AddExceptionTestDbl)
{
    secp256k1::g1::element lhs = secp256k1::g1::element::random_element();
    secp256k1::g1::element rhs;
    rhs = lhs;

    secp256k1::g1::element result;
    secp256k1::g1::element expected;

    result = lhs + rhs;
    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(secp256k1, AddDblConsistency)
{
    secp256k1::g1::element a = secp256k1::g1::element::random_element();
    secp256k1::g1::element b = secp256k1::g1::element::random_element();

    secp256k1::g1::element c;
    secp256k1::g1::element d;
    secp256k1::g1::element add_result;
    secp256k1::g1::element dbl_result;

    c = a + b;
    b = -b;
    d = a + b;

    add_result = c + d;
    dbl_result = a.dbl();

    EXPECT_EQ(add_result == dbl_result, true);
}

TEST(secp256k1, AddDblConsistencyRepeated)
{
    secp256k1::g1::element a = secp256k1::g1::element::random_element();
    secp256k1::g1::element b;
    secp256k1::g1::element c;
    secp256k1::g1::element d;
    secp256k1::g1::element e;

    secp256k1::g1::element result;
    secp256k1::g1::element expected;

    b = a.dbl(); // b = 2a
    c = b.dbl(); // c = 4a

    d = a + b;      // d = 3a
    e = a + c;      // e = 5a
    result = d + e; // result = 8a

    expected = c.dbl(); // expected = 8a

    EXPECT_EQ(result == expected, true);
}

TEST(secp256k1, MixedAddExceptionTestInfinity)
{
    secp256k1::g1::element lhs = secp256k1::g1::one;
    secp256k1::g1::affine_element rhs = secp256k1::g1::element::random_element();
    secp256k1::fq::__copy(rhs.x, lhs.x);
    lhs.y = -rhs.y;

    secp256k1::g1::element result;
    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    lhs.self_set_infinity();
    result = lhs + rhs;
    secp256k1::g1::element rhs_c;
    rhs_c = secp256k1::g1::element(rhs);

    EXPECT_EQ(rhs_c == result, true);
}

TEST(secp256k1, MixedAddExceptionTestDbl)
{
    secp256k1::g1::affine_element rhs = secp256k1::g1::element::random_element();
    secp256k1::g1::element lhs;
    lhs = secp256k1::g1::element(rhs);

    secp256k1::g1::element result;
    secp256k1::g1::element expected;
    result = lhs + rhs;

    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(secp256k1, AddMixedAddConsistencyCheck)
{
    secp256k1::g1::affine_element rhs = secp256k1::g1::element::random_element();
    secp256k1::g1::element lhs = secp256k1::g1::element::random_element();
    secp256k1::g1::element rhs_b;
    rhs_b = secp256k1::g1::element(rhs);

    secp256k1::g1::element add_result;
    secp256k1::g1::element mixed_add_result;
    add_result = lhs + rhs_b;
    mixed_add_result = lhs + rhs;

    EXPECT_EQ(add_result == mixed_add_result, true);
}

TEST(secp256k1, OnCurve)
{
    for (size_t i = 0; i < 100; ++i) {
        secp256k1::g1::element test = secp256k1::g1::element::random_element();
        EXPECT_EQ(test.on_curve(), true);
        secp256k1::g1::affine_element affine_test = secp256k1::g1::element::random_element();
        EXPECT_EQ(affine_test.on_curve(), true);
    }
}
TEST(secp256k1, BatchNormalize)
{
    size_t num_points = 2;
    std::vector<secp256k1::g1::element> points(num_points);
    std::vector<secp256k1::g1::element> normalized(num_points);
    for (size_t i = 0; i < num_points; ++i) {
        secp256k1::g1::element a = secp256k1::g1::element::random_element();
        secp256k1::g1::element b = secp256k1::g1::element::random_element();
        points[i] = a + b;
        normalized[i] = points[i];
    }
    secp256k1::g1::element::batch_normalize(&normalized[0], num_points);

    for (size_t i = 0; i < num_points; ++i) {
        secp256k1::fq zz;
        secp256k1::fq zzz;
        secp256k1::fq result_x;
        secp256k1::fq result_y;
        zz = points[i].z.sqr();
        zzz = points[i].z * zz;
        result_x = normalized[i].x * zz;
        result_y = normalized[i].y * zzz;

        EXPECT_EQ((result_x == points[i].x), true);
        EXPECT_EQ((result_y == points[i].y), true);
    }
}

TEST(secp256k1, GroupExponentiationZeroAndOne)
{
    secp256k1::g1::affine_element result = secp256k1::g1::one * secp256k1::fr::zero();

    EXPECT_EQ(result.is_point_at_infinity(), true);

    result = secp256k1::g1::one * secp256k1::fr::one();

    EXPECT_EQ(result == secp256k1::g1::affine_one, true);
}

TEST(secp256k1, GroupExponentiationConsistencyCheck)
{
    secp256k1::fr a = secp256k1::fr::random_element();
    secp256k1::fr b = secp256k1::fr::random_element();

    secp256k1::fr c;
    c = a * b;

    secp256k1::g1::affine_element input = secp256k1::g1::affine_one;
    secp256k1::g1::affine_element result = input * a;
    result = result * b;

    secp256k1::g1::affine_element expected = input * c;

    EXPECT_EQ(result == expected, true);
}

TEST(secp256k1, DeriveGenerators)
{
    constexpr size_t num_generators = 128;
    auto result = secp256k1::g1::derive_generators("test generators", num_generators);

    const auto is_unique = [&result](const secp256k1::g1::affine_element& y, const size_t j) {
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

TEST(secp256k1, GetEndomorphismScalars)
{
    for (size_t i = 0; i < 2048; i++) {
        secp256k1::fr k = secp256k1::fr::random_element();
        secp256k1::fr k1 = 0;
        secp256k1::fr k2 = 0;

        secp256k1::fr::split_into_endomorphism_scalars(k, k1, k2);
        bool k1_neg = false;
        bool k2_neg = false;

        if (k2.uint256_t_no_montgomery_conversion().get_msb() > 200) {
            k2 = -k2;
            k2_neg = true;
        }

        EXPECT_LT(k1.uint256_t_no_montgomery_conversion().get_msb(), 129ULL);
        EXPECT_LT(k2.uint256_t_no_montgomery_conversion().get_msb(), 129ULL);

        if (k1_neg) {
            k1 = -k1;
        }
        if (k2_neg) {
            k2 = -k2;
        }

        k1.self_to_montgomery_form();
        k2.self_to_montgomery_form();

        secp256k1::fr beta = secp256k1::fr::cube_root_of_unity();
        secp256k1::fr expected = k1 - k2 * beta;

        expected.self_from_montgomery_form();
        EXPECT_EQ(k, expected);
    }
}

TEST(secp256k1, TestEndomorphismScalars)
{
    secp256k1::fr k = secp256k1::fr::random_element();
    secp256k1::fr k1 = 0;
    secp256k1::fr k2 = 0;

    secp256k1::fr::split_into_endomorphism_scalars(k, k1, k2);
    bool k1_neg = false;
    bool k2_neg = false;

    if (k1.uint256_t_no_montgomery_conversion().get_msb() > 200) {
        k1 = -k1;
        k1_neg = true;
    }
    if (k2.uint256_t_no_montgomery_conversion().get_msb() > 200) {
        k2 = -k2;
        k2_neg = true;
    }

    EXPECT_LT(k1.uint256_t_no_montgomery_conversion().get_msb(), 129ULL);
    EXPECT_LT(k2.uint256_t_no_montgomery_conversion().get_msb(), 129ULL);

    if (k1_neg) {
        k1 = -k1;
    }
    if (k2_neg) {
        k2 = -k2;
    }
    k1.self_to_montgomery_form();
    k2.self_to_montgomery_form();
    static const uint256_t secp256k1_const_lambda{
        0xDF02967C1B23BD72ULL, 0x122E22EA20816678UL, 0xA5261C028812645AULL, 0x5363AD4CC05C30E0ULL
    };

    secp256k1::fr expected = k1 - k2 * secp256k1_const_lambda;

    expected.self_from_montgomery_form();
    EXPECT_EQ(k, expected);
}

TEST(secp256k1, NegAndSelfNeg0CmpRegression)
{
    secp256k1::fq a = 0;
    secp256k1::fq a_neg = -a;
    EXPECT_EQ((a == a_neg), true);
    a = 0;
    a_neg = 0;
    a_neg.self_neg();
    EXPECT_EQ((a == a_neg), true);
}

TEST(secp256k1, MontgomeryMulBigBug)
{
    secp256k1::fq a(uint256_t{ 0xfffffffe630dc02f, 0xffffffffffffffff, 0xffffffffffffffff, 0xffffffffffffffff });
    secp256k1::fq a_sqr = a.sqr();
    secp256k1::fq expected(uint256_t{ 0x60381e557e100000, 0x0, 0x0, 0x0 });
    EXPECT_EQ((a_sqr == expected), true);
}
