#include "fq2.hpp"
#include <gtest/gtest.h>

using namespace bb;

TEST(fq2, eq)
{
    fq2 a{ { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } };
    fq2 b{ { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } };
    fq2 c{ { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } };
    fq2 d{ { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } };
    fq2 e{ { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } };
    fq2 f{ { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } };
    fq2 g{ { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } };
    fq2 h{ { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } };
    fq2 i{ { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } };
    fq2 j{ { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } };

    EXPECT_EQ(a == b, true);
    EXPECT_EQ(a == c, false);
    EXPECT_EQ(a == d, false);
    EXPECT_EQ(a == e, false);
    EXPECT_EQ(a == f, false);
    EXPECT_EQ(a == g, false);
    EXPECT_EQ(a == h, false);
    EXPECT_EQ(a == i, false);
    EXPECT_EQ(a == j, false);
}

TEST(fq2, IsZero)
{
    fq2 a = fq2::zero();
    fq2 b = fq2::zero();
    fq2 c = fq2::zero();
    b.c0.data[0] = 1;
    c.c1.data[0] = 1;
    EXPECT_EQ(a.is_zero(), true);
    EXPECT_EQ(b.is_zero(), false);
    EXPECT_EQ(c.is_zero(), false);
}

TEST(fq2, RandomElement)
{
    fq2 a = fq2::random_element();
    fq2 b = fq2::random_element();

    EXPECT_EQ(a == b, false);
    EXPECT_EQ(a.is_zero(), false);
    EXPECT_EQ(b.is_zero(), false);
}

TEST(fq2, MulCheckAgainstConstants)
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    fq2 a = { { 0xd673ba38b8c4bc86, 0x860cd1cb9e2f0c85, 0x3185f9f9166177b7, 0xd043f963ced2529 },
              { 0xd4d2fad9a3de5d98, 0x260f72ca434ef415, 0xca5c20c435accb2d, 0x122a54f828a07ffe } };
    fq2 b = { { 0x37710e0986ad0fab, 0xd9b1f41ba9d3bd92, 0xf71f600e90104795, 0x24e1f6018a4d85c6 },
              { 0x5e65448f225b0f60, 0x7783aecd5d7bfa84, 0xc7a76eed72d68723, 0xc8f427c031af99a } };
    fq2 expected = { { 0x1652ca66b00ad519, 0x6619a315656ea7c7, 0x1d8491b044e9a08f, 0xcbe6d11bff2e56b },
                     { 0x9694fb422eff4e79, 0xebdbcf03e8539a17, 0xc4787fb63b8d10e8, 0x1a5cc397aae8811f } };
#else
    fq2 a = { { 0xed72e66054afa688UL, 0x58ee4e882533c50UL, 0x6e3d116ec0243404UL, 0x1d657f309417a3d8UL },
              { 0xc8d8ca2255efd3acUL, 0xa7dd5a778489041bUL, 0xa7c0d3f8a3894141UL, 0x96f1a285bc7de4UL } };
    fq2 b = { { 0x4b149f0c89ea36b8UL, 0x21c85d36fccb509UL, 0x9c6578b5dde8a9f5UL, 0x12d7656c2d09b4f5UL },
              { 0xeba4312d877a01c8UL, 0x346a85206bf0fc21UL, 0x326baffa4ec62182UL, 0xec5dbe959d2320bUL } };
    fq2 expected = { { 0xe954ec1f3d72b8e8UL, 0x7290e216a46a478UL, 0xee10085491294f00UL, 0x14ab2ea0f4cfac15UL },
                     { 0xd4761ac17f9cfd69UL, 0x6be1ccd51ae4cf91UL, 0x51bb55a8d80b3ee6UL, 0x14ef3d5468c48133UL } };
#endif

    fq2 result = a * b;
    EXPECT_EQ(result, expected);
}

TEST(fq2, SqrCheckAgainstConstants)
{
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    fq2 a = { { 0x26402fd760069ee8, 0x17828cf3bf7dd3e3, 0x4e7449f7b1149987, 0x102f6467805d7298 },
              { 0xa2a31bf895eaf6f8, 0xf0c88d415c372b16, 0xa65ccca8b7806691, 0x1b51e4526673451f } };
    fq2 expected = { { 0xb51c9049894c45f3, 0xf8ef65c0244dfc90, 0x42c37c0f7d09aacb, 0x64ddfb845b2901f },
                     { 0x9e176fa8cdca97b1, 0xd04ae89dab7da31e, 0x637b83e950322d50, 0x155cccfadafc70b4 } };
#else
    fq2 a = { { 0x6ec082078bf1f83aUL, 0x54374c9db4892e0UL, 0x9b6685d51385bd3bUL, 0x22017c733fbe1168UL },
              { 0x1a19a57784951002UL, 0x71f829f22ee524e6UL, 0xd5f4ae41d4f49ba9UL, 0x32f0638f8eb6105UL } };
    fq2 expected = { { 0xb30fd8d5c794c944UL, 0xbfe70dbee7f867e1UL, 0x772e6b159b2ff808UL, 0x82abd3d318b8341UL },
                     { 0x79264bd9e27d1c3eUL, 0xc0493fc1b97b501aUL, 0x5b0cad2ef132d4fbUL, 0x61d55130ed75444UL } };
#endif

    fq2 result = a.sqr();
    EXPECT_EQ(result, expected);
}

TEST(fq2, AddCheckAgainstConstants)
{

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    fq2 a = { { 0x517c157ce1664f30, 0x114ba401b0996437, 0x11b9ae2d856012e8, 0xcc19341ea7cf685 },
              { 0x17c6020dde15fdc0, 0x310bc25961b2f002, 0xa766e7e94a865c0d, 0x20176bc8e6b82863 } };
    fq2 b = { { 0xffad1c8ac38be684, 0x2a953b27cb1f541d, 0xfc12b9dfe76a0f12, 0x434c570deb975a6 },
              { 0x87430d4b17897ace, 0x33ab4d0e55e8932a, 0xe4465ff65990dd31, 0x83db0b3c55f9e9f } };
    fq2 expected = { { 0x51293207a4f235b4, 0x3be0df297bb8b855, 0xdcc680d6cca21fa, 0x10f658b2c9366c2c },
                     { 0x9f090f58f59f788e, 0x64b70f67b79b832c, 0x8bad47dfa417393e, 0x28551c7cac17c703 } };
#else
    fq2 a = { { 0x4e7e4ee568e1fbc8UL, 0x6d692baacf9e3280UL, 0x74b397fc9ff79a15UL, 0x150ff4a64611cf54UL },
              { 0xa14c3dc007ef12dUL, 0xb3da8d3ea50862adUL, 0xce474530b12f41f8UL, 0xab309b05df2e908UL } };
    fq2 b = { { 0x7d62792ac082d5f2UL, 0x23a48fd69306eea5UL, 0x11b6b08fea3f318aUL, 0x25d0113614cb748cUL },
              { 0xbbbeecf0b6be675dUL, 0x7fe28cf3b2d9708eUL, 0xef3aa23aaa94ec52UL, 0x15c08e3a45fbb32bUL } };
    fq2 expected = { { 0x8fc03bf950e7d473UL, 0xf98c50effa335698UL, 0xce1a02d608b57341UL, 0xa7bb76979aba3b6UL },
                     { 0xc5d3b0ccb73d588aUL, 0x33bd1a3257e1d33bUL, 0xbd81e76b5bc42e4bUL, 0x207397eaa3ee9c34UL } };
#endif
    fq2 result = a + b;
    EXPECT_EQ(result, expected);
}

TEST(fq2, SubCheckAgainstConstants)
{

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    fq2 a = { { 0x3212c3a7d7886da5, 0xcea893f4addae4aa, 0x5c8bfca7a7ed01be, 0x1a8e9dfecd598ef1 },
              { 0x4a8d9e6443fda462, 0x93248a3fde6374e7, 0xf4a6c52f75c0fc2e, 0x270aaabb4ae43370 } };
    fq2 b = { { 0x875cef17b3b46751, 0xbba7211cb92b554b, 0xa4790f1657f85606, 0x74e61182f5b5068 },
              { 0x8a84fff282dfd5a3, 0x77986fd41c21a7a3, 0xdc7072908fe375a9, 0x2e98a18c7d570269 } };
    fq2 expected = { { 0xaab5d49023d40654, 0x130172d7f4af8f5e, 0xb812ed914ff4abb8, 0x13403ce69dfe3e88 },
                     { 0xfc292a88999acc06, 0xb30d84fd2ab397d0, 0xd0869855675edee2, 0x28d657a1aebed130 } };
#else
    fq2 a = { { 0x442f277690c0e2e9UL, 0xc57a6aedcbce21e5UL, 0x542af3d6640959a2UL, 0x1b2a8a38b6e63b66UL },
              { 0x72861e4d5b7fd051UL, 0x98eddfc89951d51eUL, 0x9501d71c127de4aeUL, 0x2789ae315eadca0bUL } };
    fq2 b = { { 0xfb1bb29b1498f504UL, 0x16de795183a37f3bUL, 0xade0cbf0f9055f61UL, 0x283ae93a66a38c6dUL },
              { 0x44cf93a2fd55060eUL, 0x31e37d7946df37e4UL, 0xf4a626aecf465a37UL, 0x27530019470f8857UL } };
    fq2 expected = { { 0x853400f254a4eb2cUL, 0x461d5c2db09c6d36UL, 0x5e9a6d9bec85529fUL, 0x2353ef7131744f22UL },
                     { 0x2db68aaa5e2aca43UL, 0x670a624f52729d3aUL, 0xa05bb06d43378a77UL, 0x36ae18179e41b3UL } };
#endif

    fq2 result = a - b;
    EXPECT_EQ(result, expected);
}

TEST(fq2, ToMontgomeryForm)
{
    fq2 result = fq2::zero();
    result.c0.data[0] = 1;
    fq2 expected = fq2::one();
    result.self_to_montgomery_form();
    EXPECT_EQ(result, expected);
}

TEST(fq2, FromMontgomeryForm)
{
    fq2 result = fq2::one();
    fq2 expected = fq2::zero();
    expected.c0.data[0] = 1;
    result.self_from_montgomery_form();
    EXPECT_EQ(result, expected);
}

TEST(fq2, MulSqrConsistency)
{
    fq2 a = fq2::random_element();
    fq2 b = fq2::random_element();
    fq2 t1;
    fq2 t2;
    fq2 mul_result;
    fq2 sqr_result;
    t1 = a - b;
    t2 = a + b;
    mul_result = t1 * t2;
    t1 = a.sqr();
    t2 = b.sqr();
    sqr_result = t1 - t2;
    EXPECT_EQ(mul_result, sqr_result);
}

TEST(fq2, AddMulConsistency)
{
    fq2 multiplicand = { { 0x09, 0x00, 0x00, 0x00 }, { 0x00, 0x00, 0x00, 0x00 } };
    multiplicand = multiplicand.to_montgomery_form();

    fq2 a = fq2::random_element();
    fq2 result = a + a;
    result += result;
    result += result;
    result += a;

    fq2 expected = a * multiplicand;

    EXPECT_EQ(result, expected);
}

TEST(fq2, SubMulConsistency)
{
    fq2 multiplicand = { { 0x05, 0, 0, 0 }, { 0x00, 0x00, 0x00, 0x00 } };
    multiplicand = multiplicand.to_montgomery_form();

    fq2 a = fq2::random_element();
    fq2 result = a + a;
    result += result;
    result += result;
    result -= a;
    result -= a;
    result -= a;

    fq2 expected = a * multiplicand;

    EXPECT_EQ(result, expected);
}

TEST(fq2, Invert)
{
    fq2 input = fq2::random_element();
    fq2 inverse = input.invert();
    fq2 result = inverse * input;
    EXPECT_EQ(result, fq2::one());
}

TEST(fq2, Serialize)
{
    std::array<uint8_t, 64> buffer;
    fq expected_c0 = { 0x1234567876543210, 0x2345678987654321, 0x3456789a98765432, 0x006789abcba98765 };
    fq expected_c1 = { 0x12a4e67f76b43210, 0x23e56f898a65cc21, 0x005678add98e5432, 0x1f6789a2cba98700 };
    fq2 expected{ expected_c0, expected_c1 };

    fq2::serialize_to_buffer(expected, &buffer[0]);

    fq2 result = fq2::serialize_from_buffer(&buffer[0]);

    EXPECT_EQ(result, expected);
}