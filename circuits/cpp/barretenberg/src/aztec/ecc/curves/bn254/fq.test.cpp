#include "fq.hpp"
#include "pseudorandom.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

TEST(fq, eq)
{
    constexpr fq a{ 0x01, 0x02, 0x03, 0x04 };
    constexpr fq b{ 0x01, 0x02, 0x03, 0x04 };
    constexpr fq c{ 0x01, 0x02, 0x03, 0x05 };
    constexpr fq d{ 0x01, 0x02, 0x04, 0x04 };
    constexpr fq e{ 0x01, 0x03, 0x03, 0x04 };
    constexpr fq f{ 0x02, 0x02, 0x03, 0x04 };
    static_assert(a == b);
    static_assert(!(a == c));
    static_assert(!(a == d));
    static_assert(!(a == e));
    static_assert(!(a == f));

    fq a_var;
    fq b_var;
    fq c_var;
    fq d_var;
    fq e_var;
    fq f_var;
    memcpy((void*)a_var.data, (void*)a.data, 32);
    memcpy((void*)b_var.data, (void*)b.data, 32);
    memcpy((void*)c_var.data, (void*)c.data, 32);
    memcpy((void*)d_var.data, (void*)d.data, 32);
    memcpy((void*)e_var.data, (void*)e.data, 32);
    memcpy((void*)f_var.data, (void*)f.data, 32);

    EXPECT_EQ(a_var == a_var, true);
    EXPECT_EQ(a_var == b_var, true);
    EXPECT_EQ(a_var == c_var, false);
    EXPECT_EQ(a_var == d_var, false);
    EXPECT_EQ(a_var == e_var, false);
    EXPECT_EQ(a_var == f_var, false);
}

TEST(fq, is_zero)
{
    fq a = fq::zero();
    fq b = fq::zero();
    fq c = fq::zero();
    fq d = fq::zero();
    fq e = fq::zero();

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

TEST(fq, random_element)
{
    fq a = fq::random_element();
    fq b = fq::random_element();

    EXPECT_EQ(a == b, false);
    EXPECT_EQ(a.is_zero(), false);
    EXPECT_EQ(a.is_zero(), false);
}

TEST(fq, mul_check_against_constants)
{
    // test against some randomly generated test data
    constexpr fq a{ 0x2523b6fa3956f038, 0x158aa08ecdd9ec1d, 0xf48216a4c74738d4, 0x2514cc93d6f0a1bf };
    constexpr fq a_copy{ 0x2523b6fa3956f038, 0x158aa08ecdd9ec1d, 0xf48216a4c74738d4, 0x2514cc93d6f0a1bf };
    constexpr fq b{ 0xb68aee5e4c8fc17c, 0xc5193de7f401d5e8, 0xb8777d4dde671db3, 0xe513e75c087b0bb };
    constexpr fq b_copy = { 0xb68aee5e4c8fc17c, 0xc5193de7f401d5e8, 0xb8777d4dde671db3, 0xe513e75c087b0bb };
    constexpr fq const_expected{ 0x7ed4174114b521c4, 0x58f5bd1d4279fdc2, 0x6a73ac09ee843d41, 0x687a76ae9b3425c };
    constexpr fq const_result = a * b;

    static_assert(const_result == const_expected);
    static_assert(a == a_copy);
    static_assert(b == b_copy);

    fq c;
    fq d;
    memcpy((void*)c.data, (void*)a.data, 32);
    memcpy((void*)d.data, (void*)b.data, 32);
    EXPECT_EQ(c * d, const_expected);
}

// validate that zero-value limbs don't cause any problems
TEST(fq, mul_short_integers)
{
    constexpr fq a{ 0xa, 0, 0, 0 };
    constexpr fq b{ 0xb, 0, 0, 0 };
    constexpr fq const_expected = { 0x65991a6dc2f3a183, 0xe3ba1f83394a2d08, 0x8401df65a169db3f, 0x1727099643607bba };
    constexpr fq const_result = a * b;
    static_assert(const_result == const_expected);

    fq c;
    fq d;
    memcpy((void*)c.data, (void*)a.data, 32);
    memcpy((void*)d.data, (void*)b.data, 32);
    EXPECT_EQ(c * d, const_expected);
}

TEST(fq, mul_sqr_consistency)
{
    fq a = fq::random_element();
    fq b = fq::random_element();
    fq t1;
    fq t2;
    fq mul_result;
    fq sqr_result;
    t1 = a - b;
    t2 = a + b;
    mul_result = t1 * t2;
    t1 = a.sqr();
    t2 = b.sqr();
    sqr_result = t1 - t2;
    EXPECT_EQ(mul_result, sqr_result);
}

TEST(fq, sqr_check_against_constants)
{
    constexpr fq a{ 0x329596aa978981e8, 0x8542e6e254c2a5d0, 0xc5b687d82eadb178, 0x2d242aaf48f56b8a };
    constexpr fq expected{ 0xbf4fb34e120b8b12, 0xf64d70efbf848328, 0xefbb6a533f2e7d89, 0x1de50f941425e4aa };
    constexpr fq result = a.sqr();
    static_assert(result == expected);

    fq b;
    memcpy((void*)b.data, (void*)a.data, 32);
    fq c = b.sqr();
    EXPECT_EQ(result, c);
}

TEST(fq, add_check_against_constants)
{
    constexpr fq a{ 0x7d2e20e82f73d3e8, 0x8e50616a7a9d419d, 0xcdc833531508914b, 0xd510253a2ce62c };
    constexpr fq b{ 0x2829438b071fd14e, 0xb03ef3f9ff9274e, 0x605b671f6dc7b209, 0x8701f9d971fbc9 };
    constexpr fq const_expected{ 0xa55764733693a536, 0x995450aa1a9668eb, 0x2e239a7282d04354, 0x15c121f139ee1f6 };
    constexpr fq const_result = a + b;
    static_assert(const_result == const_expected);

    fq c;
    fq d;
    memcpy((void*)c.data, (void*)a.data, 32);
    memcpy((void*)d.data, (void*)b.data, 32);
    EXPECT_EQ(c + d, const_expected);
}

TEST(fq, sub_check_against_constants)
{
    constexpr fq a{ 0xd68d01812313fb7c, 0x2965d7ae7c6070a5, 0x08ef9af6d6ba9a48, 0x0cb8fe2108914f53 };
    constexpr fq b{ 0x2cd2a2a37e9bf14a, 0xebc86ef589c530f6, 0x75124885b362b8fe, 0x1394324205c7a41d };
    constexpr fq const_expected{ 0xe5daeaf47cf50779, 0xd51ed34a5b0d0a3c, 0x4c2d9827a4d939a6, 0x29891a51e3fb4b5f };
    constexpr fq const_result = a - b;
    static_assert(const_result == const_expected);

    fq c;
    fq d;
    memcpy((void*)c.data, (void*)a.data, 32);
    memcpy((void*)d.data, (void*)b.data, 32);
    EXPECT_EQ(c - d, const_expected);
}

TEST(fq, coarse_equivalence_checks)
{
    fq a = get_pseudorandom_fq();
    fq b = get_pseudorandom_fq();

    fq c = (a * b) + a - b;

    fq d = a * b + a - b;

    EXPECT_EQ(c, d);
}

TEST(fq, to_montgomery_form)
{
    fq result = fq{ 0x01, 0x00, 0x00, 0x00 }.to_montgomery_form();
    fq expected = fq::one();
    EXPECT_EQ(result, expected);
}

TEST(fq, from_montgomery_form)
{
    constexpr fq t0 = fq::one();
    constexpr fq result = t0.from_montgomery_form();
    constexpr fq expected{ 0x01, 0x00, 0x00, 0x00 };
    EXPECT_EQ(result, expected);
}

TEST(fq, montgomery_consistency_check)
{
    fq a = fq::random_element();
    fq b = fq::random_element();
    fq aR;
    fq bR;
    fq aRR;
    fq bRR;
    fq bRRR;
    fq result_a;
    fq result_b;
    fq result_c;
    fq result_d;
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

TEST(fq, add_mul_consistency)
{
    fq multiplicand = { 0x09, 0, 0, 0 };
    multiplicand.self_to_montgomery_form();

    fq a = fq::random_element();
    fq result;
    result = a + a;   // 2
    result += result; // 4
    result += result; // 8
    result += a;      // 9

    fq expected;
    expected = a * multiplicand;

    EXPECT_EQ((result == expected), true);
}

TEST(fq, sub_mul_consistency)
{
    fq multiplicand = { 0x05, 0, 0, 0 };
    multiplicand.self_to_montgomery_form();

    fq a = fq::random_element();
    fq result;
    result = a + a;   // 2
    result += result; // 4
    result += result; // 8
    result -= a;      // 7
    result -= a;      // 6
    result -= a;      // 5

    fq expected;
    expected = a * multiplicand;

    EXPECT_EQ((result == expected), true);
}

TEST(fq, beta)
{
    fq x = fq::random_element();

    fq beta_x = { x.data[0], x.data[1], x.data[2], x.data[3] };
    beta_x = beta_x * fq::beta();

    // compute x^3
    fq x_cubed;
    x_cubed = x * x;
    x_cubed *= x;

    // compute beta_x^3
    fq beta_x_cubed;
    beta_x_cubed = beta_x * beta_x;
    beta_x_cubed *= beta_x;

    EXPECT_EQ((x_cubed == beta_x_cubed), true);
}

TEST(fq, invert)
{
    fq input = fq::random_element();
    fq inverse = input.invert();
    fq result = input * inverse;
    result = result.reduce_once();
    result = result.reduce_once();
    EXPECT_EQ(result, fq::one());
}

TEST(fq, invert_one_is_one)
{
    fq result = fq::one();
    result = result.invert();
    EXPECT_EQ((result == fq::one()), true);
}

TEST(fq, sqrt)
{
    fq input = fq::one();
    auto [is_sqr, root] = input.sqrt();
    fq result = root.sqr();
    EXPECT_EQ(result, input);
}

TEST(fq, sqrt_random)
{
    for (size_t i = 0; i < 1; ++i) {
        fq input = fq::random_element().sqr();
        auto [is_sqr, root] = input.sqrt();
        fq root_test = root.sqr();
        EXPECT_EQ(root_test, input);
    }
}

TEST(fq, one_and_zero)
{
    fq result;
    result = fq::one() - fq::one();
    EXPECT_EQ((result == fq::zero()), true);
}

TEST(fq, copy)
{
    fq result = fq::random_element();
    fq expected;
    fq::__copy(result, expected);
    EXPECT_EQ((result == expected), true);
}

TEST(fq, neg)
{
    fq a = fq::random_element();
    fq b;
    b = -a;
    fq result;
    result = a + b;
    EXPECT_EQ((result == fq::zero()), true);
}

TEST(fq, split_into_endomorphism_scalars)
{
    fq k = fq::random_element();
    fq k1 = 0;
    fq k2 = 0;

    fq::split_into_endomorphism_scalars(k, k1, k2);

    // std::cout << "endo scalars = " << k1 << k2 << std::endl;
    fq result = 0;

    k1.self_to_montgomery_form();
    k2.self_to_montgomery_form();

    result = k2 * fq::beta();
    result = k1 - result;

    result.self_from_montgomery_form();
    EXPECT_EQ(result, k);
}

TEST(fq, split_into_endomorphism_scalars_simple)
{

    fq input = { 1, 0, 0, 0 };
    fq k = { 0, 0, 0, 0 };
    fq k1 = { 0, 0, 0, 0 };
    fq k2 = { 0, 0, 0, 0 };
    fq::__copy(input, k);

    fq::split_into_endomorphism_scalars(k, k1, k2);

    fq result{ 0, 0, 0, 0 };
    k1.self_to_montgomery_form();
    k2.self_to_montgomery_form();

    result = k2 * fq::beta();
    result = k1 - result;

    result.self_from_montgomery_form();
    for (size_t i = 0; i < 4; ++i) {
        EXPECT_EQ(result.data[i], k.data[i]);
    }
}

TEST(fq, serialize_to_buffer)
{
    uint8_t buffer[32];
    fq a = { 0x1234567876543210, 0x2345678987654321, 0x3456789a98765432, 0x006789abcba98765 };
    a = a.to_montgomery_form();

    fq::serialize_to_buffer(a, &buffer[0]);

    EXPECT_EQ(buffer[31], 0x10);
    EXPECT_EQ(buffer[30], 0x32);
    EXPECT_EQ(buffer[29], 0x54);
    EXPECT_EQ(buffer[28], 0x76);
    EXPECT_EQ(buffer[27], 0x78);
    EXPECT_EQ(buffer[26], 0x56);
    EXPECT_EQ(buffer[25], 0x34);
    EXPECT_EQ(buffer[24], 0x12);

    EXPECT_EQ(buffer[23], 0x21);
    EXPECT_EQ(buffer[22], 0x43);
    EXPECT_EQ(buffer[21], 0x65);
    EXPECT_EQ(buffer[20], 0x87);
    EXPECT_EQ(buffer[19], 0x89);
    EXPECT_EQ(buffer[18], 0x67);
    EXPECT_EQ(buffer[17], 0x45);
    EXPECT_EQ(buffer[16], 0x23);

    EXPECT_EQ(buffer[15], 0x32);
    EXPECT_EQ(buffer[14], 0x54);
    EXPECT_EQ(buffer[13], 0x76);
    EXPECT_EQ(buffer[12], 0x98);
    EXPECT_EQ(buffer[11], 0x9a);
    EXPECT_EQ(buffer[10], 0x78);
    EXPECT_EQ(buffer[9], 0x56);
    EXPECT_EQ(buffer[8], 0x34);

    EXPECT_EQ(buffer[7], 0x65);
    EXPECT_EQ(buffer[6], 0x87);
    EXPECT_EQ(buffer[5], 0xa9);
    EXPECT_EQ(buffer[4], 0xcb);
    EXPECT_EQ(buffer[3], 0xab);
    EXPECT_EQ(buffer[2], 0x89);
    EXPECT_EQ(buffer[1], 0x67);
    EXPECT_EQ(buffer[0], 0x00);
}

TEST(fq, serialize_from_buffer)
{
    uint8_t buffer[32];
    fq expected = { 0x1234567876543210, 0x2345678987654321, 0x3456789a98765432, 0x006789abcba98765 };

    fq::serialize_to_buffer(expected, &buffer[0]);

    fq result = fq::serialize_from_buffer(&buffer[0]);

    EXPECT_EQ((result == expected), true);
}

TEST(fq, multiplicative_generator)
{
    EXPECT_EQ(fq::multiplicative_generator(), fq(3));
}

TEST(fq, r_inv)
{
    uint256_t prime_256{
        Bn254FqParams::modulus_0, Bn254FqParams::modulus_1, Bn254FqParams::modulus_2, Bn254FqParams::modulus_3
    };
    uint512_t r{ 0, 1 };
    // -(1/q) mod r
    uint512_t q{ -prime_256, 0 };
    uint256_t q_inv = q.invmod(r).lo;
    uint64_t expected = (q_inv).data[0];
    uint64_t result = Bn254FqParams::r_inv;
    EXPECT_EQ(result, expected);
}

// TEST to check we don't have 0^0=0
TEST(fq, pow_regression_check)
{
    fq zero = fq::zero();
    fq one = fq::one();
    EXPECT_EQ(zero.pow(uint256_t(0)), one);
}
//   438268ca91d42ad f1e7025a7b654e1f f8d9d72e0438b995 8c422ec208ac8a6e
