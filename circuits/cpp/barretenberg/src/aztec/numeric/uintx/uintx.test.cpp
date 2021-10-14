#include "../random/engine.hpp"
#include "./uintx.hpp"
#include <gtest/gtest.h>

namespace {
auto& engine = numeric::random::get_debug_engine();
} // namespace

TEST(uintx, get_bit)
{
    constexpr uint256_t lo{ 0b0110011001110010011001100111001001100110011100100110011001110011,
                            0b1001011101101010101010100100101101101001001010010101110101010111,
                            0b0101010010010101111100001011011010101010110101110110110111010101,
                            0b0101011010101010100010001000101011010101010101010010000100000000 };

    constexpr uint256_t hi{ 0b0110011001110010011001100111001001100110011100100110011001110011,
                            0b1001011101101010101010100100101101101001001010010101110101010111,
                            0b0101010010010101111100001011011010101010110101110110110111010101,
                            0b0101011010101010100010001000101011010101010101010010000100000000 };

    constexpr uint1024_t a(uint512_t(lo, hi), uint512_t(lo, hi));
    uint1024_t res(0);
    for (size_t i = 0; i < 1024; ++i) {
        res += a.get_bit(i) ? (uint1024_t(1) << i) : 0;
    }

    EXPECT_EQ(a, res);
}

TEST(uintx, mul)
{
    uint1024_t a = engine.get_random_uint1024();
    uint1024_t b = engine.get_random_uint1024();

    uint1024_t c = (a + b) * (a + b);
    uint1024_t d = (a * a) + (b * b) + (a * b) + (a * b);
    EXPECT_EQ(c, d);
}

TEST(uintx, div_and_mod)
{
    for (size_t i = 0; i < 256; ++i) {
        uint1024_t a = engine.get_random_uint1024();
        uint1024_t b = engine.get_random_uint1024();

        uint1024_t q = a / b;
        uint1024_t r = a % b;

        uint1024_t c = q * b + r;
        EXPECT_EQ(c, a);
    }

    uint1024_t b = engine.get_random_uint1024();
    uint1024_t a = 0;

    uint1024_t q = a / b;
    uint1024_t r = a % b;

    EXPECT_EQ(q, uint1024_t(0));
    EXPECT_EQ(r, uint1024_t(0));

    b = a;
    q = a / b;
    r = a % b;

    EXPECT_EQ(q, uint1024_t(0));
    EXPECT_EQ(r, uint1024_t(0));
}

// We should not be depending on ecc in numeric.
TEST(uintx, DISABLED_mulmod)
{
    /*
        barretenberg::fq a = barretenberg::fq::random_element();
        barretenberg::fq b = barretenberg::fq::random_element();
        // barretenberg::fq a_converted = a.from_montgomery_form();
        // barretenberg::fq b_converted = b.from_montgomery_form();
        uint256_t a_uint =
            uint256_t(a); // { a_converted.data[0], a_converted.data[1], a_converted.data[2], a_converted.data[3] };
        uint256_t b_uint =
            uint256_t(b); // { b_converted.data[0], b_converted.data[1], b_converted.data[2], b_converted.data[3] };
        uint256_t modulus_uint{ barretenberg::Bn254FqParams::modulus_0,
                                barretenberg::Bn254FqParams::modulus_1,
                                barretenberg::Bn254FqParams::modulus_2,
                                barretenberg::Bn254FqParams::modulus_3 };
        uint1024_t a_uintx = uint1024_t(uint512_t(a_uint));
        uint1024_t b_uintx = uint1024_t(uint512_t(b_uint));
        uint1024_t modulus_uintx = uint1024_t(uint512_t(modulus_uint));

        const auto [quotient, remainder] = (a_uintx * b_uintx).divmod(modulus_uintx);

        // barretenberg::fq expected_a = a_converted.to_montgomery_form();
        // barretenberg::fq expected_b = b_converted.to_montgomery_form();
        barretenberg::fq expected = (a * b).from_montgomery_form();

        EXPECT_EQ(remainder.lo.lo.data[0], expected.data[0]);
        EXPECT_EQ(remainder.lo.lo.data[1], expected.data[1]);
        EXPECT_EQ(remainder.lo.lo.data[2], expected.data[2]);
        EXPECT_EQ(remainder.lo.lo.data[3], expected.data[3]);

        const auto rhs = (quotient * modulus_uintx) + remainder;
        const auto lhs = a_uintx * b_uintx;
        EXPECT_EQ(lhs, rhs);
    */
}

TEST(uintx, sub)
{
    uint1024_t a = engine.get_random_uint1024();
    uint1024_t b = engine.get_random_uint1024();

    uint1024_t c = (a - b) * (a + b);
    uint1024_t d = (a * a) - (b * b);

    EXPECT_EQ(c, d);

    uint1024_t e = 0;
    e = e - 1;

    EXPECT_EQ(e.lo.lo.data[0], UINT64_MAX);
    EXPECT_EQ(e.lo.lo.data[1], UINT64_MAX);
    EXPECT_EQ(e.lo.lo.data[2], UINT64_MAX);
    EXPECT_EQ(e.lo.lo.data[3], UINT64_MAX);
    EXPECT_EQ(e.lo.hi.data[0], UINT64_MAX);
    EXPECT_EQ(e.lo.hi.data[1], UINT64_MAX);
    EXPECT_EQ(e.lo.hi.data[2], UINT64_MAX);
    EXPECT_EQ(e.lo.hi.data[3], UINT64_MAX);
    EXPECT_EQ(e.hi.lo.data[0], UINT64_MAX);
    EXPECT_EQ(e.hi.lo.data[1], UINT64_MAX);
    EXPECT_EQ(e.hi.lo.data[2], UINT64_MAX);
    EXPECT_EQ(e.hi.lo.data[3], UINT64_MAX);
    EXPECT_EQ(e.hi.hi.data[0], UINT64_MAX);
    EXPECT_EQ(e.hi.hi.data[1], UINT64_MAX);
    EXPECT_EQ(e.hi.hi.data[2], UINT64_MAX);
    EXPECT_EQ(e.hi.hi.data[3], UINT64_MAX);
}

TEST(uintx, and)
{
    uint1024_t a = engine.get_random_uint1024();
    uint1024_t b = engine.get_random_uint1024();

    uint1024_t c = a & b;

    EXPECT_EQ(c.lo, a.lo & b.lo);
    EXPECT_EQ(c.hi, a.hi & b.hi);
}

TEST(uintx, or)
{
    uint1024_t a = engine.get_random_uint1024();
    uint1024_t b = engine.get_random_uint1024();

    uint1024_t c = a | b;

    EXPECT_EQ(c.lo, a.lo | b.lo);
    EXPECT_EQ(c.hi, a.hi | b.hi);
}

TEST(uintx, xor)
{
    uint1024_t a = engine.get_random_uint1024();
    uint1024_t b = engine.get_random_uint1024();

    uint1024_t c = a ^ b;

    EXPECT_EQ(c.lo, a.lo ^ b.lo);
    EXPECT_EQ(c.hi, a.hi ^ b.hi);
}

TEST(uintx, bit_not)
{
    uint1024_t a = engine.get_random_uint1024();

    uint1024_t c = ~a;

    EXPECT_EQ(c.lo, ~a.lo);
    EXPECT_EQ(c.hi, ~a.hi);
}

TEST(uintx, logic_not)
{
    uint1024_t a(1);

    bool b = !a;

    EXPECT_EQ(b, false);

    uint1024_t c(0);

    EXPECT_EQ(!c, true);
}

TEST(uintx, not_equal)
{
    uint1024_t a(1);
    uint1024_t b(1);
    EXPECT_EQ(a != b, false);

    a = uint1024_t(0);
    EXPECT_EQ(a != b, true);
}

// We should not be depending on ecc in numeric.
TEST(uintx, DISABLED_invmod)
{
    /*
    uint256_t prime_lo = prime_256;
    uint1024_t prime = uint1024_t(uint512_t(prime_lo));
    uint256_t target_lo = engine.get_random_uint256();
    uint1024_t target = uint1024_t(uint512_t(target_lo));
    uint256_t inverse = uint256_t(uint512_t(target.invmod(prime)));

    uint256_t expected = uint256_t(fr(target_lo).invert());
    EXPECT_EQ(inverse, expected);
    */
}

TEST(uintx, DISABLED_r_inv)
{
    /*
    uint512_t r{ 0, 1 };
    // -(1/q) mod r
    uint512_t q{ -prime_256, 0 };
    uint256_t q_inv = q.invmod(r).lo;
    uint64_t result = q_inv.data[0];
    EXPECT_EQ(result, Bn254FrParams::r_inv);
    */
}