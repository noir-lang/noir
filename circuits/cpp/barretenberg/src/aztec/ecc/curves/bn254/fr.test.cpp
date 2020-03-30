#include "fr.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

TEST(fr, eq)
{
    fr a{ 0x01, 0x02, 0x03, 0x04 };
    fr b{ 0x01, 0x02, 0x03, 0x04 };
    fr c{ 0x01, 0x02, 0x03, 0x05 };
    fr d{ 0x01, 0x02, 0x04, 0x04 };
    fr e{ 0x01, 0x03, 0x03, 0x04 };
    fr f{ 0x02, 0x02, 0x03, 0x04 };
    EXPECT_EQ((a == b), true);
    EXPECT_EQ((a == c), false);
    EXPECT_EQ((a == d), false);
    EXPECT_EQ((a == e), false);
    EXPECT_EQ((a == f), false);
}

TEST(fr, is_zero)
{
    fr a = fr::zero();
    fr b = fr::zero();
    fr c = fr::zero();
    fr d = fr::zero();
    fr e = fr::zero();

    b.data[0] = 1;
    c.data[1] = 1;
    d.data[2] = 1;
    e.data[3] = 1;
    EXPECT_EQ(a.is_zero(), true);
    EXPECT_EQ(b.is_zero(), false);
    EXPECT_EQ(c.is_zero(), false);
    EXPECT_EQ(d.is_zero(), false);
    EXPECT_EQ(e.is_zero(), false);
}

TEST(fr, random_element)
{
    fr a = fr::random_element();
    fr b = fr::random_element();

    EXPECT_EQ((a == b), false);
    EXPECT_EQ(a.is_zero(), false);
    EXPECT_EQ(b.is_zero(), false);
}

TEST(fr, mul)
{
    fr a{ 0x192f9ddc938ea63, 0x1db93d61007ec4fe, 0xc89284ec31fa49c0, 0x2478d0ff12b04f0f };
    fr b{ 0x7aade4892631231c, 0x8e7515681fe70144, 0x98edb76e689b6fd8, 0x5d0886b15fc835fa };
    fr expected{ 0xab961ef46b4756b6, 0xbc6b636fc29678c8, 0xd247391ed6b5bd16, 0x12e8538b3bde6784 };
    fr result;
    result = a * b;
    EXPECT_EQ((result == expected), true);
}

TEST(fr, sqr)
{
    fr a{ 0x95f946723a1fc34f, 0x641ec0482fc40bb9, 0xb8d645bc49dd513d, 0x1c1bffd317599dbc };
    fr expected{ 0xc787f7d9e2c72714, 0xcf21cf53d8f65f67, 0x8db109903dac0008, 0x26ab4dd65f46be5f };
    fr result;
    result = a.sqr();
    EXPECT_EQ((result == expected), true);
}

TEST(fr, add)
{
    fr a{ 0x20565a572c565a66, 0x7bccd0f01f5f7bff, 0x63ec2beaad64711f, 0x624953caaf44a814 };
    fr b{ 0xa17307a2108adeea, 0x74629976c14c5e2b, 0x9ce6f072ab1740ee, 0x398c753702b2bef0 };
    fr expected{ 0x7de76c654ce1394f, 0xc7fb821e66f26999, 0x4882d6a6d6fa59b0, 0x6b717a8ed0c5c6db };
    fr result;
    result = a + b;
    EXPECT_EQ(result, expected.reduce_once());
}

TEST(fr, sub)
{
    fr a{ 0xcfbcfcf457cf2d38, 0x7b27af26ce62aa61, 0xf0378e90d48f2b92, 0x4734b22cb21ded };
    fr b{ 0x569fdb1db5198770, 0x446ddccef8347d52, 0xef215227182d22a, 0x8281b4fb109306 };
    fr expected{ 0xbcff176a92b5a5c9, 0x5eedbaa04fe79da0, 0x9995bf24e48db1c5, 0x3029017012d32b11 };
    fr result;
    result = a - b;
    EXPECT_EQ((result == expected), true);
}

TEST(fr, to_montgomery_form)
{
    fr result{ 0x01, 0x00, 0x00, 0x00 };
    fr expected = fr::one();
    result.self_to_montgomery_form();
    EXPECT_EQ((result == expected), true);
}

TEST(fr, from_montgomery_form)
{
    fr result = fr::one();
    fr expected{ 0x01, 0x00, 0x00, 0x00 };
    result.self_from_montgomery_form();
    EXPECT_EQ((result == expected), true);
}

TEST(fr, montgomery_consistency_check)
{
    fr a = fr::random_element();
    fr b = fr::random_element();
    fr aR;
    fr bR;
    fr aRR;
    fr bRR;
    fr bRRR;
    fr result_a;
    fr result_b;
    fr result_c;
    fr result_d;
    aR = a.to_montgomery_form();
    aRR = aR.to_montgomery_form();
    bR = b.to_montgomery_form();
    bRR = bR.to_montgomery_form();
    bRRR = bRR.to_montgomery_form();
    result_a = aRR * bRR; // abRRR
    result_b = aR * bRRR; // abRRR
    result_c = aR * bR;   // abR
    result_d = a * b;     // abR^-1
    EXPECT_EQ((result_a == result_b), true);
    result_a.self_from_montgomery_form(); // abRR
    result_a.self_from_montgomery_form(); // abR
    result_a.self_from_montgomery_form(); // ab
    result_c.self_from_montgomery_form(); // ab
    result_d.self_to_montgomery_form();   // ab
    EXPECT_EQ((result_a == result_c), true);
    EXPECT_EQ((result_a == result_d), true);
}

TEST(fr, add_mul_consistency)
{
    fr multiplicand = { 0x09, 0, 0, 0 };
    multiplicand.self_to_montgomery_form();

    fr a = fr::random_element();
    fr result;
    result = a + a;   // 2
    result += result; // 4
    result += result; // 8
    result += a;      // 9

    fr expected;
    expected = a * multiplicand;

    EXPECT_EQ((result == expected), true);
}

TEST(fr, sub_mul_consistency)
{
    fr multiplicand = { 0x05, 0, 0, 0 };
    multiplicand.self_to_montgomery_form();

    fr a = fr::random_element();
    fr result;
    result = a + a;   // 2
    result += result; // 4
    result += result; // 8
    result -= a;      // 7
    result -= a;      // 6
    result -= a;      // 5

    fr expected;
    expected = a * multiplicand;

    EXPECT_EQ((result == expected), true);
}

TEST(fr, lambda)
{
    fr x = fr::random_element();

    fr lambda_x = { x.data[0], x.data[1], x.data[2], x.data[3] };
    lambda_x = lambda_x * fr::beta();

    // compute x^3
    fr x_cubed;
    x_cubed = x * x;
    x_cubed *= x;

    // compute lambda_x^3
    fr lambda_x_cubed;
    lambda_x_cubed = lambda_x * lambda_x;
    lambda_x_cubed *= lambda_x;

    EXPECT_EQ((x_cubed == lambda_x_cubed), true);
}

TEST(fr, invert)
{
    fr input = fr::random_element();
    fr inverse = input.invert();
    fr result = input * inverse;

    EXPECT_EQ((result == fr::one()), true);
}

TEST(fr, invert_one_is_one)
{
    fr result = fr::one();
    result = result.invert();
    EXPECT_EQ((result == fr::one()), true);
}

TEST(fr, sqrt)
{
    fr input = fr::one();
    fr root = input.sqrt();
    fr result = root.sqr();
    EXPECT_EQ(result, input);
}

TEST(fr, sqrt_random)
{
    size_t n = 1;
    for (size_t i = 0; i < n; ++i) {
        fr input = fr::random_element().sqr();
        fr root_test = input.sqrt().sqr();
        EXPECT_EQ(root_test, input);
    }
}

TEST(fr, one_and_zero)
{
    fr result;
    result = fr::one() - fr::one();
    EXPECT_EQ((result == fr::zero()), true);
}

TEST(fr, copy)
{
    fr result = fr::random_element();
    fr expected;
    fr::__copy(result, expected);
    EXPECT_EQ((result == expected), true);
}

TEST(fr, neg)
{
    fr a = fr::random_element();
    fr b;
    b = -a;
    fr result;
    result = a + b;
    EXPECT_EQ((result == fr::zero()), true);
}

TEST(fr, split_into_endomorphism_scalars)
{
    fr k = fr::random_element();
    fr k1 = { 0, 0, 0, 0 };
    fr k2 = { 0, 0, 0, 0 };

    fr::split_into_endomorphism_scalars(k, k1, k2);

    fr result{ 0, 0, 0, 0 };

    k1.self_to_montgomery_form();
    k2.self_to_montgomery_form();

    result = k2 * fr::beta();
    result = k1 - result;

    result.self_from_montgomery_form();
    EXPECT_EQ(result, k);
}

TEST(fr, split_into_endomorphism_scalars_simple)
{

    fr input = { 1, 0, 0, 0 };
    fr k = { 0, 0, 0, 0 };
    fr k1 = { 0, 0, 0, 0 };
    fr k2 = { 0, 0, 0, 0 };
    fr::__copy(input, k);

    fr::split_into_endomorphism_scalars(k, k1, k2);

    fr result{ 0, 0, 0, 0 };
    k1.self_to_montgomery_form();
    k2.self_to_montgomery_form();

    result = k2 * fr::beta();
    result = k1 - result;

    result.self_from_montgomery_form();
    for (size_t i = 0; i < 4; ++i) {
        EXPECT_EQ(result.data[i], k.data[i]);
    }
}

TEST(fr, batch_invert)
{
    size_t n = 10;
    fr coeffs[n];
    fr inverses[n];
    fr one = fr::one();
    for (size_t i = 0; i < n; ++i) {
        coeffs[i] = fr::random_element();
        fr::__copy(coeffs[i], inverses[i]);
    }
    fr::batch_invert(inverses, n);

    for (size_t i = 0; i < n; ++i) {
        coeffs[i] *= inverses[i];
        coeffs[i] -= one;
    }

    for (size_t i = 0; i < n; ++i) {
        EXPECT_EQ(coeffs[i].data[0], 0UL);
        EXPECT_EQ(coeffs[i].data[1], 0UL);
        EXPECT_EQ(coeffs[i].data[2], 0UL);
        EXPECT_EQ(coeffs[i].data[3], 0UL);
    }
}

TEST(fr, multiplicative_generator)
{
    EXPECT_EQ(fr::multiplicative_generator(), fr(5));
}

TEST(fr, uint256_conversions)
{
    constexpr uint256_t a{ 0x1111, 0x2222, 0x3333, 0x4444 };

    constexpr fr b(a);
    constexpr uint256_t c = b;

    static_assert(a == c);
    EXPECT_EQ(a, c);
}