#include "../random/engine.hpp"
#include "uint128.hpp"
#include <gtest/gtest.h>
#ifdef __i386__
namespace {
auto& engine = numeric::random::get_debug_engine();
}

using namespace numeric;

TEST(uint128, get_bit)
{
    constexpr uint128_t a{ 0b0110011001110010011001100,
                           0b1001011101101010101010100,
                           0b0101010010010101111100001,
                           0b0101011010101010100010001 };

    uint128_t res;
    for (size_t i = 0; i < 128; ++i) {
        res += a.get_bit(i) ? (uint128_t(1) << i) : 0;
    }

    EXPECT_EQ(a, res);
}

TEST(uint128, add)
{
    constexpr uint128_t a{ 1, 2, 3, 4 };
    constexpr uint128_t b{ 5, 6, 7, 8 };

    constexpr uint128_t c = a + b;
    uint128_t d = a;
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

TEST(uint128, get_msb)
{
    uint128_t a{ 0, 0, 1, 1 };
    uint128_t b{ 1, 0, 1, 0 };
    uint128_t c{ 0, 1, 0, 0 };
    uint128_t d{ 1, 0, 0, 0 };

    EXPECT_EQ(a.get_msb(), 96ULL);
    EXPECT_EQ(b.get_msb(), 64ULL);
    EXPECT_EQ(c.get_msb(), 32ULL);
    EXPECT_EQ(d.get_msb(), 0ULL);
}

TEST(uint128, mul)
{
    uint128_t a = engine.get_random_uint128();
    uint128_t b = engine.get_random_uint128();

    uint128_t c = (a + b) * (a + b);
    uint128_t d = (a * a) + (b * b) + (a * b) + (a * b);
    EXPECT_EQ(c.data[0], d.data[0]);
    EXPECT_EQ(c.data[1], d.data[1]);
    EXPECT_EQ(c.data[2], d.data[2]);
    EXPECT_EQ(c.data[3], d.data[3]);
}

TEST(uint128, div_and_mod)
{
    for (size_t i = 0; i < 128; ++i) {
        uint128_t a = engine.get_random_uint128();
        uint128_t b = engine.get_random_uint128();

        b.data[3] = (i > 0) ? 0 : b.data[3];
        b.data[2] = (i > 1) ? 0 : b.data[2];
        b.data[1] = (i > 2) ? 0 : b.data[1];
        uint128_t q = a / b;
        uint128_t r = a % b;

        uint128_t c = q * b + r;
        EXPECT_EQ(c.data[0], a.data[0]);
        EXPECT_EQ(c.data[1], a.data[1]);
        EXPECT_EQ(c.data[2], a.data[2]);
        EXPECT_EQ(c.data[3], a.data[3]);
    }

    uint128_t a = engine.get_random_uint128();
    uint128_t b = 0;

    uint128_t q = a / b;
    uint128_t r = a % b;

    EXPECT_EQ(q, uint128_t(0));
    EXPECT_EQ(r, uint128_t(0));

    b = a;
    q = a / b;
    r = a % b;

    EXPECT_EQ(q, uint128_t(1));
    EXPECT_EQ(r, uint128_t(0));
}

TEST(uint128, sub)
{
    uint128_t a = engine.get_random_uint128();
    uint128_t b = engine.get_random_uint128();

    uint128_t c = (a - b) * (a + b);
    uint128_t d = (a * a) - (b * b);

    EXPECT_EQ(c.data[0], d.data[0]);
    EXPECT_EQ(c.data[1], d.data[1]);
    EXPECT_EQ(c.data[2], d.data[2]);
    EXPECT_EQ(c.data[3], d.data[3]);

    uint128_t e = 0;
    e = e - 1;

    EXPECT_EQ(e.data[0], UINT32_MAX);
    EXPECT_EQ(e.data[1], UINT32_MAX);
    EXPECT_EQ(e.data[2], UINT32_MAX);
    EXPECT_EQ(e.data[3], UINT32_MAX);
}

TEST(uint128, right_shift)
{
    constexpr uint128_t a{ 0xaaaaaaaa, 0xbbbbbbbb, 0xcccccccc, 0xdddddddd };

    constexpr uint128_t b = a >> 128;
    EXPECT_EQ(b, uint128_t(0));

    constexpr uint128_t c = a >> 0;
    EXPECT_EQ(a, c);

    constexpr uint128_t d = a >> 32;
    EXPECT_EQ(d, uint128_t(0xbbbbbbbb, 0xcccccccc, 0xdddddddd, 0));

    constexpr uint128_t e = a >> 59;
    constexpr uint128_t f = e * (uint128_t{ 0, 1ULL << 27ULL, 0, 0 });
    EXPECT_EQ(f, uint128_t(0, 0xb8000000, 0xcccccccc, 0xdddddddd));
}

TEST(uint128, left_shift)
{
    uint128_t a{ 0xaaaaaaaa, 0xbbbbbbbb, 0xcccccccc, 0xdddddddd };

    uint128_t b = a << 128;
    EXPECT_EQ(b, uint128_t(0));

    uint128_t c = a << 0;
    EXPECT_EQ(a, c);

    uint128_t d = a << 32;
    EXPECT_EQ(d, uint128_t(0, 0xaaaaaaaa, 0xbbbbbbbb, 0xcccccccc));

    uint128_t e = a << 123;
    e = e >> 123;
    EXPECT_EQ(e, uint128_t(0xa, 0, 0, 0));

    uint128_t large_shift = uint128_t(1) << 64;
    uint128_t f = a << large_shift;
    EXPECT_EQ(f, uint128_t(0));
}

TEST(uint128, and)
{
    uint128_t a = engine.get_random_uint128();
    uint128_t b = engine.get_random_uint128();

    uint128_t c = a & b;

    EXPECT_EQ(c.data[0], a.data[0] & b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] & b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] & b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] & b.data[3]);
}

TEST(uint128, or)
{
    uint128_t a = engine.get_random_uint128();
    uint128_t b = engine.get_random_uint128();

    uint128_t c = a | b;

    EXPECT_EQ(c.data[0], a.data[0] | b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] | b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] | b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] | b.data[3]);
}

TEST(uint128, xor)
{
    uint128_t a = engine.get_random_uint128();
    uint128_t b = engine.get_random_uint128();

    uint128_t c = a ^ b;

    EXPECT_EQ(c.data[0], a.data[0] ^ b.data[0]);
    EXPECT_EQ(c.data[1], a.data[1] ^ b.data[1]);
    EXPECT_EQ(c.data[2], a.data[2] ^ b.data[2]);
    EXPECT_EQ(c.data[3], a.data[3] ^ b.data[3]);
}

TEST(uint128, bit_not)
{
    uint128_t a = engine.get_random_uint128();

    uint128_t c = ~a;

    EXPECT_EQ(c.data[0], ~a.data[0]);
    EXPECT_EQ(c.data[1], ~a.data[1]);
    EXPECT_EQ(c.data[2], ~a.data[2]);
    EXPECT_EQ(c.data[3], ~a.data[3]);
}

TEST(uint128, logic_not)
{
    uint128_t a{ 1, 0, 0, 0 };

    bool b = !a;

    EXPECT_EQ(b, false);

    uint128_t c{ 0, 0, 0, 0 };

    EXPECT_EQ(!c, true);
}

TEST(uint128, equality)
{
    uint128_t a{ 1, 0, 0, 0 };
    uint128_t b{ 1, 0, 0, 0 };
    EXPECT_EQ(a == b, true);

    a = uint128_t{ 0, 1, 0, 0 };
    EXPECT_EQ(a == b, false);

    a = uint128_t{ 0, 0, 1, 0 };
    EXPECT_EQ(a == b, false);

    a = uint128_t{ 0, 0, 0, 1 };
    EXPECT_EQ(a == b, false);

    a = uint128_t{ 555, 0, 0, 1 };
    b = uint128_t{ 535, 0, 0, 1 };
    EXPECT_EQ(a == b, false);
}

TEST(uint128, not_equal)
{
    uint128_t a{ 1, 0, 0, 0 };
    uint128_t b{ 1, 0, 0, 0 };
    EXPECT_EQ(a != b, false);

    a = uint128_t{ 0, 1, 0, 0 };
    EXPECT_EQ(a != b, true);

    a = uint128_t{ 0, 0, 1, 0 };
    EXPECT_EQ(a != b, true);

    a = uint128_t{ 0, 0, 0, 1 };
    EXPECT_EQ(a != b, true);

    a = uint128_t{ 555, 0, 0, 1 };
    b = uint128_t{ 535, 0, 0, 1 };
    EXPECT_EQ(a != b, true);
}

TEST(uint128, greater_than)
{
    constexpr uint128_t a{ UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX };
    constexpr uint128_t b{ UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX };
    EXPECT_EQ(a > b, false);

    constexpr uint128_t c = uint128_t{ UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX - 1 };
    EXPECT_EQ(a > c, true);

    constexpr uint128_t d = uint128_t{ UINT32_MAX, UINT32_MAX, UINT32_MAX - 1, UINT32_MAX };
    EXPECT_EQ(a > d, true);

    constexpr uint128_t e = uint128_t{ UINT32_MAX, UINT32_MAX - 1, UINT32_MAX, UINT32_MAX };
    EXPECT_EQ(a > e, true);

    constexpr uint128_t f = uint128_t{ UINT32_MAX - 1, UINT32_MAX, UINT32_MAX, UINT32_MAX };
    EXPECT_EQ(a > f, true);
}

TEST(uint128, greater_than_or_equal)
{
    uint128_t a{ UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX - 1 };
    uint128_t b{ UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX };
    EXPECT_EQ(a >= b, false);

    b = uint128_t{ UINT32_MAX, UINT32_MAX, UINT32_MAX, UINT32_MAX - 1 };
    EXPECT_EQ(a > b, false);
    EXPECT_EQ(a >= b, true);

    b = uint128_t{ UINT32_MAX, UINT32_MAX, UINT32_MAX - 1, UINT32_MAX };
    EXPECT_EQ(a >= b, false);

    a = uint128_t{ UINT32_MAX, UINT32_MAX - 1, UINT32_MAX - 1, UINT32_MAX };
    EXPECT_EQ(a >= b, false);

    b = uint128_t{ UINT32_MAX - 1, UINT32_MAX, UINT32_MAX, UINT32_MAX };
    EXPECT_EQ(a >= b, false);
}

TEST(uint128, to_from_buffer)
{
    uint128_t a{ 1, 2, 3, 4 };
    auto buf = to_buffer(a);
    uint128_t b = from_buffer<uint128_t>(buf);
    EXPECT_EQ(a, b);
}
#endif