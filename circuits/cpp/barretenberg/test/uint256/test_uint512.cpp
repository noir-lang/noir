#include <barretenberg/curves/bn254/fr.hpp>
#include <barretenberg/uint256/uint512.hpp>
#include <gtest/gtest.h>

#include "../test_helpers.hpp"

#include <random>

using namespace barretenberg;

TEST(uint512, get_bit)
{
    constexpr uint256_t lo{ 0b0110011001110010011001100111001001100110011100100110011001110011,
                            0b1001011101101010101010100100101101101001001010010101110101010111,
                            0b0101010010010101111100001011011010101010110101110110110111010101,
                            0b0101011010101010100010001000101011010101010101010010000100000000 };

    constexpr uint256_t hi{ 0b0110011001110010011001100111001001100110011100100110011001110011,
                            0b1001011101101010101010100100101101101001001010010101110101010111,
                            0b0101010010010101111100001011011010101010110101110110110111010101,
                            0b0101011010101010100010001000101011010101010101010010000100000000 };

    constexpr uint512_t a(lo, hi);
    uint512_t res(0);
    for (size_t i = 0; i < 512; ++i) {
        res += a.get_bit(i) ? (uint512_t(1) << i) : 0;
    }

    EXPECT_EQ(a, res);
}

TEST(uint512, add)
{
    constexpr uint512_t a{ { 1, 2, 3, 4 }, { 1, 2, 3, 4 } };
    constexpr uint512_t b{ { 5, 6, 7, 8 }, { 5, 6, 7, 8 } };

    constexpr uint512_t c = a + b;
    uint512_t d = a;
    d += b;
    EXPECT_EQ(c.lo.data[0], 6ULL);
    EXPECT_EQ(c.lo.data[1], 8ULL);
    EXPECT_EQ(c.lo.data[2], 10ULL);
    EXPECT_EQ(c.lo.data[3], 12ULL);
    EXPECT_EQ(d.lo.data[0], 6ULL);
    EXPECT_EQ(d.lo.data[1], 8ULL);
    EXPECT_EQ(d.lo.data[2], 10ULL);
    EXPECT_EQ(d.lo.data[3], 12ULL);
    EXPECT_EQ(c.hi.data[0], 6ULL);
    EXPECT_EQ(c.hi.data[1], 8ULL);
    EXPECT_EQ(c.hi.data[2], 10ULL);
    EXPECT_EQ(c.hi.data[3], 12ULL);
    EXPECT_EQ(d.hi.data[0], 6ULL);
    EXPECT_EQ(d.hi.data[1], 8ULL);
    EXPECT_EQ(d.hi.data[2], 10ULL);
    EXPECT_EQ(d.hi.data[3], 12ULL);
}

TEST(uint512, get_msb)
{
    uint512_t a{ { 0, 0, 1, 1 }, { 0, 0, 1, 1 } };
    uint512_t b{ { 1, 0, 1, 0 }, { 1, 0, 1, 0 } };
    uint512_t c{ { 0, 1, 0, 0 }, { 0, 1, 0, 0 } };
    uint512_t d{ { 1, 0, 0, 0 }, { 1, 0, 0, 0 } };
    uint512_t e{ { 1, 0, 0, 0 }, { 0, 0, 0, 0 } };

    EXPECT_EQ(a.get_msb(), 256ULL + 192ULL);
    EXPECT_EQ(b.get_msb(), 256ULL + 128ULL);
    EXPECT_EQ(c.get_msb(), 256ULL + 64ULL);
    EXPECT_EQ(d.get_msb(), 256ULL);
    EXPECT_EQ(e.get_msb(), 0ULL);
}

TEST(uint512, mul)
{
    uint512_t a = test_helpers::get_pseudorandom_uint512();
    uint512_t b = test_helpers::get_pseudorandom_uint512();

    uint512_t c = (a + b) * (a + b);
    uint512_t d = (a * a) + (b * b) + (a * b) + (a * b);
    EXPECT_EQ(c, d);
}

TEST(uint512, div_and_mod)
{
    for (size_t i = 0; i < 256; ++i) {
        uint512_t a = test_helpers::get_pseudorandom_uint512();
        uint512_t b = test_helpers::get_pseudorandom_uint512();

        b.hi.data[3] = (i > 0) ? 0 : b.hi.data[3];
        b.hi.data[2] = (i > 1) ? 0 : b.hi.data[2];
        b.hi.data[1] = (i > 2) ? 0 : b.hi.data[1];
        uint512_t q = a / b;
        uint512_t r = a % b;

        uint512_t c = q * b + r;
        EXPECT_EQ(c, a);
    }

    uint512_t a = test_helpers::get_pseudorandom_uint512();
    uint512_t b = 0;

    uint512_t q = a / b;
    uint512_t r = a % b;

    EXPECT_EQ(q, uint512_t(0));
    EXPECT_EQ(r, uint512_t(0));

    b = a;
    q = a / b;
    r = a % b;

    EXPECT_EQ(q, uint512_t(1));
    EXPECT_EQ(r, uint512_t(0));
}

TEST(uint512, sub)
{
    uint512_t a = test_helpers::get_pseudorandom_uint512();
    uint512_t b = test_helpers::get_pseudorandom_uint512();

    uint512_t c = (a - b) * (a + b);
    uint512_t d = (a * a) - (b * b);

    EXPECT_EQ(c, d);

    uint512_t e = 0;
    e = e - 1;

    EXPECT_EQ(e.lo.data[0], UINT64_MAX);
    EXPECT_EQ(e.lo.data[1], UINT64_MAX);
    EXPECT_EQ(e.lo.data[2], UINT64_MAX);
    EXPECT_EQ(e.lo.data[3], UINT64_MAX);
    EXPECT_EQ(e.hi.data[0], UINT64_MAX);
    EXPECT_EQ(e.hi.data[1], UINT64_MAX);
    EXPECT_EQ(e.hi.data[2], UINT64_MAX);
    EXPECT_EQ(e.hi.data[3], UINT64_MAX);
}

TEST(uint512, right_shift)
{
    constexpr uint512_t a{ { 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd },
                           { 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd } };

    constexpr uint512_t b = a >> 512;
    EXPECT_EQ(b, uint512_t(0));

    constexpr uint512_t c = a >> 0;
    EXPECT_EQ(a, c);

    constexpr uint512_t d = a >> 64;
    EXPECT_EQ(d.hi, uint256_t(0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd, 0));
    EXPECT_EQ(d.lo, uint256_t(0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd, 0xaaaaaaaaaaaaaaaa));

    constexpr uint512_t e = a >> 123;
    constexpr uint512_t f = e * (uint256_t{ 0, 1ULL << 59ULL, 0, 0 });
    EXPECT_EQ(f.lo, uint256_t(0, 0xb800000000000000, 0xcccccccccccccccc, 0xdddddddddddddddd));
}

// TEST(uint512, left_shift)
// {
//     uint512_t a{ 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd };

//     uint512_t b = a << 256;
//     EXPECT_EQ(b, uint512_t(0));

//     uint512_t c = a << 0;
//     EXPECT_EQ(a, c);

//     uint512_t d = a << 64;
//     EXPECT_EQ(d, uint512_t(0, 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc));

//     uint512_t e = a << 123;
//     e = e >> 123;
//     EXPECT_EQ(e, uint512_t(0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xc, 0));
// }

TEST(uint512, and)
{
    uint512_t a = test_helpers::get_pseudorandom_uint512();
    uint512_t b = test_helpers::get_pseudorandom_uint512();

    uint512_t c = a & b;

    EXPECT_EQ(c.lo, a.lo & b.lo);
    EXPECT_EQ(c.hi, a.hi & b.hi);
}

TEST(uint512, or)
{
    uint512_t a = test_helpers::get_pseudorandom_uint512();
    uint512_t b = test_helpers::get_pseudorandom_uint512();

    uint512_t c = a | b;

    EXPECT_EQ(c.lo, a.lo | b.lo);
    EXPECT_EQ(c.hi, a.hi | b.hi);
}

TEST(uint512, xor)
{
    uint512_t a = test_helpers::get_pseudorandom_uint512();
    uint512_t b = test_helpers::get_pseudorandom_uint512();

    uint512_t c = a ^ b;

    EXPECT_EQ(c.lo, a.lo ^ b.lo);
    EXPECT_EQ(c.hi, a.hi ^ b.hi);
}

TEST(uint512, bit_not)
{
    uint512_t a = test_helpers::get_pseudorandom_uint512();

    uint512_t c = ~a;

    EXPECT_EQ(c.lo, ~a.lo);
    EXPECT_EQ(c.hi, ~a.hi);
}

TEST(uint512, logic_not)
{
    uint512_t a(1);

    bool b = !a;

    EXPECT_EQ(b, false);

    uint512_t c(0);

    EXPECT_EQ(!c, true);
}

TEST(uint512, equality)
{
    uint512_t a(1);
    uint512_t b(1);
    EXPECT_EQ(a == b, true);

    a = uint512_t{ { 0, 1, 0, 0 }, { 0, 0, 0, 0 } };
    EXPECT_EQ(a == b, false);

    a = uint512_t{ { 0, 0, 1, 0 }, { 1, 0, 0, 0 } };
    EXPECT_EQ(a == b, false);

    a = uint512_t{ { 1, 0, 0, 0 }, { 1, 0, 0, 0 } };
    EXPECT_EQ(a == b, false);
}

TEST(uint512, not_equal)
{
    uint512_t a(1);
    uint512_t b(1);
    EXPECT_EQ(a != b, false);

    a = uint512_t(0);
    EXPECT_EQ(a != b, true);
}

TEST(uint512, greater_than)
{
    uint512_t a{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                 { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX } };
    uint512_t b{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                 { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX } };
    EXPECT_EQ(a > b, false);

    b.hi = uint256_t{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX - 1 };
    EXPECT_EQ(a > b, true);

    b.hi = uint256_t{ UINT64_MAX, UINT64_MAX, UINT64_MAX - 1, UINT64_MAX };
    EXPECT_EQ(a > b, true);

    b.hi = uint256_t{ UINT64_MAX, UINT64_MAX - 1, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a > b, true);

    b.hi = uint256_t{ UINT64_MAX - 1, UINT64_MAX, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a > b, true);
}

TEST(uint512, greater_than_or_equal)
{
    uint512_t a{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                 { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX - 1 } };
    uint512_t b{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                 { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX } };
    EXPECT_EQ(a >= b, false);

    b = uint512_t{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                   { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX - 1 } };
    EXPECT_EQ(a > b, false);
    EXPECT_EQ(a >= b, true);

    b = uint512_t{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                   { UINT64_MAX, UINT64_MAX, UINT64_MAX - 1, UINT64_MAX } };
    EXPECT_EQ(a >= b, false);

    a = uint512_t{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                   { UINT64_MAX, UINT64_MAX - 1, UINT64_MAX - 1, UINT64_MAX } };
    EXPECT_EQ(a >= b, false);

    b = uint512_t{ { UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX },
                   { UINT64_MAX - 1, UINT64_MAX, UINT64_MAX, UINT64_MAX } };
    EXPECT_EQ(a >= b, false);
}

TEST(uint512, invmod)
{
    uint256_t prime_lo = fr::modulus;
    // uint256_t prime_lo(fr::modulus.data[0],
    //                    fr::modulus.data[1],
    //                    fr::modulus.data[2],
    //                    fr::modulus.data[3]);
    uint512_t prime(prime_lo, uint256_t(0));
    uint256_t target_lo = test_helpers::get_pseudorandom_uint256();
    uint512_t inverse = uint512_t(target_lo, uint256_t(0)).invmod(prime);

    uint512_t expected = uint256_t(fr(target_lo).invert());
    EXPECT_EQ(inverse, expected);
}

TEST(uint512, r_squared)
{
    uint256_t prime_256 = fr::modulus;
    // uint256_t prime_256(fr::modulus.data[0],
    //                     fr::modulus.data[1],
    //                     fr::modulus.data[2],
    //                     fr::modulus.data[3]);
    uint256_t R = -prime_256;
    uint256_t R_mod_p = R % prime_256;

    uint512_t R_512(R_mod_p);

    uint512_t R_squared = R_512 * R_512;

    uint512_t R_squared_mod_p = R_squared % uint512_t(prime_256);

    uint512_t expected{ uint256_t(
                            FrParams::r_squared_0, FrParams::r_squared_1, FrParams::r_squared_2, FrParams::r_squared_3),
                        uint256_t(0) };
    EXPECT_EQ(R_squared_mod_p, expected);
}