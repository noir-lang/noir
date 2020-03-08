#include "../random/engine.hpp"
#include "uint256.hpp"
#include <gtest/gtest.h>

namespace {
auto& engine = barretenberg::random::get_debug_engine();
}

TEST(uint256, get_bit)
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

TEST(uint256, add)
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

TEST(uint256, get_msb)
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

TEST(uint256, mul)
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

TEST(uint256, div_and_mod)
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

TEST(uint256, sub)
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

TEST(uint256, right_shift)
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

TEST(uint256, left_shift)
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
}

TEST(uint256, and)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = a & b;

    EXPECT_EQ(c.data[0], a.data[0] & b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] & b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] & b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] & b.data[3]);
}

TEST(uint256, or)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = a | b;

    EXPECT_EQ(c.data[0], a.data[0] | b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] | b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] | b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] | b.data[3]);
}

TEST(uint256, xor)
{
    uint256_t a = engine.get_random_uint256();
    uint256_t b = engine.get_random_uint256();

    uint256_t c = a ^ b;

    EXPECT_EQ(c.data[0], a.data[0] ^ b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] ^ b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] ^ b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] ^ b.data[3]);
}

TEST(uint256, bit_not)
{
    uint256_t a = engine.get_random_uint256();

    uint256_t c = ~a;

    EXPECT_EQ(c.data[0], ~a.data[0]);
    EXPECT_EQ(c.data[1], ~a.data[1]);
    EXPECT_EQ(c.data[2], ~a.data[2]);
    EXPECT_EQ(c.data[3], ~a.data[3]);
}

TEST(uint256, logic_not)
{
    uint256_t a{ 1, 0, 0, 0 };

    bool b = !a;

    EXPECT_EQ(b, false);

    uint256_t c{ 0, 0, 0, 0 };

    EXPECT_EQ(!c, true);
}

TEST(uint256, equality)
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

TEST(uint256, not_equal)
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

TEST(uint256, greater_than)
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

TEST(uint256, greater_than_or_equal)
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

// TODO: Move to field tests
// TEST(uint256, field_conversions)
// {
//     constexpr uint256_t a{ 0x1111, 0x2222, 0x3333, 0x4444 };

//     constexpr barretenberg::fr b(a);
//     constexpr uint256_t c = b;

//     static_assert(a == c);
//     EXPECT_EQ(a, c);
// }
