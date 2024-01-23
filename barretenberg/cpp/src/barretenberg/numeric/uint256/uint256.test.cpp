#include "uint256.hpp"
#include "../random/engine.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::numeric;
namespace {
auto& engine = numeric::get_debug_randomness();
}

TEST(uint256, TestStringConstructors)
{
    std::string input = "9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789";
    const std::string input4("0x9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789");

    const uint256_t result1(input);
    constexpr uint256_t result2("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789");
    const uint256_t result3("0x9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789");
    const uint256_t result4(input4);
    constexpr uint256_t expected{
        0xabcdef0123456789,
        0xfedcba9876543210,
        0xa0b1c2d3e4f56789,
        0x9a807b615c4d3e2f,
    };
    EXPECT_EQ(result1, result2);
    EXPECT_EQ(result1, result3);
    EXPECT_EQ(result1, result4);
    EXPECT_EQ(result1, expected);
}

TEST(uint256, GetBit)
{
    constexpr uint256_t a{ 0b0110011001110010011001100111001001100110011100100110011001110011,
                           0b1001011101101010101010100100101101101001001010010101110101010111,
                           0b0101010010010101111100001011011010101010110101110110110111010101,
                           0b0101011010101010100010001000101011010101010101010010000100000000 };

    uint256_t res;
    for (size_t i = 0; i < 256; ++i) {
        res += a.get_bit(i) ? (uint256_t(1) << i) : 0;
    }

    EXPECT_EQ(a, res);
}

TEST(uint256, Add)
{
    constexpr uint256_t a{ 1, 2, 3, 4 };
    constexpr uint256_t b{ 5, 6, 7, 8 };

    constexpr uint256_t c = a + b;
    uint256_t d = a;
    d += b;
    EXPECT_EQ(c.data[0], 6ULL);
    EXPECT_EQ(c.data[1], 8ULL);
    EXPECT_EQ(c.data[2], 10ULL);
    EXPECT_EQ(c.data[3], 12ULL);
    EXPECT_EQ(d.data[0], 6ULL);
    EXPECT_EQ(d.data[1], 8ULL);
    EXPECT_EQ(d.data[2], 10ULL);
    EXPECT_EQ(d.data[3], 12ULL);
}

TEST(uint256, GetMsb)
{
    uint256_t a{ 0, 0, 1, 1 };
    uint256_t b{ 1, 0, 1, 0 };
    uint256_t c{ 0, 1, 0, 0 };
    uint256_t d{ 1, 0, 0, 0 };

    EXPECT_EQ(a.get_msb(), 192ULL);
    EXPECT_EQ(b.get_msb(), 128ULL);
    EXPECT_EQ(c.get_msb(), 64ULL);
    EXPECT_EQ(d.get_msb(), 0ULL);
}

TEST(uint256, Mul)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = (a + b) * (a + b);
    uint256_t d = (a * a) + (b * b) + (a * b) + (a * b);
    EXPECT_EQ(c.data[0], d.data[0]);
    EXPECT_EQ(c.data[1], d.data[1]);
    EXPECT_EQ(c.data[2], d.data[2]);
    EXPECT_EQ(c.data[3], d.data[3]);
}

TEST(uint256, DivAndMod)
{
    for (size_t i = 0; i < 256; ++i) {
        uint256_t a = engine.get_random_uint256();
        uint256_t b = engine.get_random_uint256();

        b.data[3] = (i > 0) ? 0 : b.data[3];
        b.data[2] = (i > 1) ? 0 : b.data[2];
        b.data[1] = (i > 2) ? 0 : b.data[1];
        uint256_t q = a / b;
        uint256_t r = a % b;

        uint256_t c = q * b + r;
        EXPECT_EQ(c.data[0], a.data[0]);
        EXPECT_EQ(c.data[1], a.data[1]);
        EXPECT_EQ(c.data[2], a.data[2]);
        EXPECT_EQ(c.data[3], a.data[3]);
    }

    uint256_t a = engine.get_random_uint256();
    uint256_t b = 0;

    uint256_t q = a / b;
    uint256_t r = a % b;

    EXPECT_EQ(q, uint256_t(0));
    EXPECT_EQ(r, uint256_t(0));

    b = a;
    q = a / b;
    r = a % b;

    EXPECT_EQ(q, uint256_t(1));
    EXPECT_EQ(r, uint256_t(0));
}

TEST(uint256, Sub)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = (a - b) * (a + b);
    uint256_t d = (a * a) - (b * b);

    EXPECT_EQ(c.data[0], d.data[0]);
    EXPECT_EQ(c.data[1], d.data[1]);
    EXPECT_EQ(c.data[2], d.data[2]);
    EXPECT_EQ(c.data[3], d.data[3]);

    uint256_t e = 0;
    e = e - 1;

    EXPECT_EQ(e.data[0], UINT64_MAX);
    EXPECT_EQ(e.data[1], UINT64_MAX);
    EXPECT_EQ(e.data[2], UINT64_MAX);
    EXPECT_EQ(e.data[3], UINT64_MAX);
}

TEST(uint256, RightShift)
{
    constexpr uint256_t a{ 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd };

    constexpr uint256_t b = a >> 256;
    EXPECT_EQ(b, uint256_t(0));

    constexpr uint256_t c = a >> 0;
    EXPECT_EQ(a, c);

    constexpr uint256_t d = a >> 64;
    EXPECT_EQ(d, uint256_t(0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd, 0));

    constexpr uint256_t e = a >> 123;
    constexpr uint256_t f = e * (uint256_t{ 0, 1ULL << 59ULL, 0, 0 });
    EXPECT_EQ(f, uint256_t(0, 0xb800000000000000, 0xcccccccccccccccc, 0xdddddddddddddddd));
}

TEST(uint256, LeftShift)
{
    uint256_t a{ 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc, 0xdddddddddddddddd };

    uint256_t b = a << 256;
    EXPECT_EQ(b, uint256_t(0));

    uint256_t c = a << 0;
    EXPECT_EQ(a, c);

    uint256_t d = a << 64;
    EXPECT_EQ(d, uint256_t(0, 0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xcccccccccccccccc));

    uint256_t e = a << 123;
    e = e >> 123;
    EXPECT_EQ(e, uint256_t(0xaaaaaaaaaaaaaaaa, 0xbbbbbbbbbbbbbbbb, 0xc, 0));

    uint256_t large_shift = uint256_t(1) << 64;
    uint256_t f = a << large_shift;
    EXPECT_EQ(f, uint256_t(0));
}

TEST(uint256, And)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = a & b;

    EXPECT_EQ(c.data[0], a.data[0] & b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] & b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] & b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] & b.data[3]);
}

TEST(uint256, Or)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = a | b;

    EXPECT_EQ(c.data[0], a.data[0] | b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] | b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] | b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] | b.data[3]);
}

TEST(uint256, Xor)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = a ^ b;

    EXPECT_EQ(c.data[0], a.data[0] ^ b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] ^ b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] ^ b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] ^ b.data[3]);
}

TEST(uint256, BitNot)
{
    uint256_t a = engine.get_random_uint256();

    uint256_t c = ~a;

    EXPECT_EQ(c.data[0], ~a.data[0]);
    EXPECT_EQ(c.data[1], ~a.data[1]);
    EXPECT_EQ(c.data[2], ~a.data[2]);
    EXPECT_EQ(c.data[3], ~a.data[3]);
}

TEST(uint256, LogicNot)
{
    uint256_t a{ 1, 0, 0, 0 };

    bool b = !a;

    EXPECT_EQ(b, false);

    uint256_t c{ 0, 0, 0, 0 };

    EXPECT_EQ(!c, true);
}

TEST(uint256, Equality)
{
    uint256_t a{ 1, 0, 0, 0 };
    uint256_t b{ 1, 0, 0, 0 };
    EXPECT_EQ(a == b, true);

    a = uint256_t{ 0, 1, 0, 0 };
    EXPECT_EQ(a == b, false);

    a = uint256_t{ 0, 0, 1, 0 };
    EXPECT_EQ(a == b, false);

    a = uint256_t{ 0, 0, 0, 1 };
    EXPECT_EQ(a == b, false);

    a = uint256_t{ 555, 0, 0, 1 };
    b = uint256_t{ 535, 0, 0, 1 };
    EXPECT_EQ(a == b, false);
}

TEST(uint256, NotEqual)
{
    uint256_t a{ 1, 0, 0, 0 };
    uint256_t b{ 1, 0, 0, 0 };
    EXPECT_EQ(a != b, false);

    a = uint256_t{ 0, 1, 0, 0 };
    EXPECT_EQ(a != b, true);

    a = uint256_t{ 0, 0, 1, 0 };
    EXPECT_EQ(a != b, true);

    a = uint256_t{ 0, 0, 0, 1 };
    EXPECT_EQ(a != b, true);

    a = uint256_t{ 555, 0, 0, 1 };
    b = uint256_t{ 535, 0, 0, 1 };
    EXPECT_EQ(a != b, true);
}

TEST(uint256, GreaterThan)
{
    constexpr uint256_t a{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX };
    constexpr uint256_t b{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a > b, false);

    constexpr uint256_t c = uint256_t{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX - 1 };
    EXPECT_EQ(a > c, true);

    constexpr uint256_t d = uint256_t{ UINT64_MAX, UINT64_MAX, UINT64_MAX - 1, UINT64_MAX };
    EXPECT_EQ(a > d, true);

    constexpr uint256_t e = uint256_t{ UINT64_MAX, UINT64_MAX - 1, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a > e, true);

    constexpr uint256_t f = uint256_t{ UINT64_MAX - 1, UINT64_MAX, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a > f, true);
}

TEST(uint256, GreaterThanOrEqual)
{
    uint256_t a{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX - 1 };
    uint256_t b{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a >= b, false);

    b = uint256_t{ UINT64_MAX, UINT64_MAX, UINT64_MAX, UINT64_MAX - 1 };
    EXPECT_EQ(a > b, false);
    EXPECT_EQ(a >= b, true);

    b = uint256_t{ UINT64_MAX, UINT64_MAX, UINT64_MAX - 1, UINT64_MAX };
    EXPECT_EQ(a >= b, false);

    a = uint256_t{ UINT64_MAX, UINT64_MAX - 1, UINT64_MAX - 1, UINT64_MAX };
    EXPECT_EQ(a >= b, false);

    b = uint256_t{ UINT64_MAX - 1, UINT64_MAX, UINT64_MAX, UINT64_MAX };
    EXPECT_EQ(a >= b, false);
}

TEST(uint256, ToFromBuffer)
{
    uint256_t a{ 1, 2, 3, 4 };
    auto buf = to_buffer(a);
    auto b = from_buffer<uint256_t>(buf);
    EXPECT_EQ(a, b);
}
