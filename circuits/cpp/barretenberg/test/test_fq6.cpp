#include <gtest/gtest.h>

#include <barretenberg/curves/bn254/fq6.hpp>

using namespace barretenberg;

TEST(fq6, eq)
{
    fq6 a{ { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } };
    fq6 b{ { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } };
    fq6 c{ { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x05 }, { 0x06, 0x07, 0x08, 0x09 } } };
    fq6 d{ { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x04, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } };
    fq6 e{ { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x03, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } };
    fq6 f{ { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } },
           { { 0x02, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x09 } } };
    fq6 g{ { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x07, 0x07, 0x08, 0x09 } } };
    fq6 h{ { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x08, 0x08, 0x09 } } };
    fq6 i{ { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x09, 0x09 } } };
    fq6 j{ { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } },
           { { 0x01, 0x02, 0x03, 0x04 }, { 0x06, 0x07, 0x08, 0x0a } } };

    EXPECT_EQ((a == b), true);
    EXPECT_EQ((a == c), false);
    EXPECT_EQ((a == d), false);
    EXPECT_EQ((a == e), false);
    EXPECT_EQ((a == f), false);
    EXPECT_EQ((a == g), false);
    EXPECT_EQ((a == h), false);
    EXPECT_EQ((a == i), false);
    EXPECT_EQ((a == j), false);
}

TEST(fq6, is_zero)
{
    fq6 a = fq6::zero();
    fq6 b = fq6::zero();
    fq6 c = fq6::zero();
    fq6 d = fq6::zero();
    b.c0.c0.data[0] = 1;
    c.c1.c0.data[0] = 1;
    d.c2.c0.data[0] = 1;
    EXPECT_EQ(a.is_zero(), true);
    EXPECT_EQ(b.is_zero(), false);
    EXPECT_EQ(c.is_zero(), false);
    EXPECT_EQ(d.is_zero(), false);
}

TEST(fq6, random_element)
{
    fq6 a = fq6::random_element();
    fq6 b = fq6::random_element();

    EXPECT_EQ((a == b), false);
    EXPECT_EQ(a.is_zero(), false);
    EXPECT_EQ(b.is_zero(), false);
}

TEST(fq6, add_check_against_constants)
{
    fq6 a{ { { 0x68138b3c3e5e820b, 0x9bf71d36786da85f, 0x815831c12e257996, 0x2280b875a27e6d1d },
             { 0xff4b05a0ed0df393, 0x94fbe538fe78c1e7, 0xea26586b05301ac2, 0xc90d9fbd6f0360a } },
           { { 0x52c807d7b79f5d98, 0xb81d0afcd80e2015, 0xfaf3309048d58378, 0x2b214cef3a9c4595 },
             { 0xffb4f038e11aba58, 0x3090e3e2be73918, 0xb5e3ff91aea9ea10, 0x259c4ada803d9709 } },
           { { 0x558fd8ed6c427bbb, 0xa96c066b688d01be, 0xf9f575fe06c46881, 0x1e7fa15a71e843c9 },
             { 0xa24a278e3b53d755, 0xd6038431b043e405, 0xe0591d1304349843, 0x2d54f10885a3553 } } };
    fq6 b{ { { 0x77c31be6f025aa97, 0x1b5e9b0f80b0bd4, 0xdcd589b606f17780, 0x1b96f57a8bae9f62 },
             { 0x9d1544f116a22cea, 0x2fbff58e7d1148d7, 0xd3bdeba899719e45, 0x2c0f12a9af45c0df } },
           { { 0xc29dbca781a36282, 0xfa3111cc49f4189e, 0x239461afa706e127, 0x26b933107ae211b6 },
             { 0x2eaeb7bf42b03771, 0x5e33621ea59ed72d, 0xf29b1b9d29c2b509, 0x21a3f6e4192b48f5 } },
           { { 0x37d865d5cf9054e4, 0x120abe21b24abd40, 0x22ae39101351e0e, 0x3d256f12acefd0e },
             { 0x2ae9dbb8fe966761, 0x3a10cc50c534e2f9, 0xdf0883c7482be190, 0x2fd2b4f6573d9828 } } };
    fq6 expected{ { { 0xa3b61b0c56072f5b, 0x62b9c560806e9a6, 0xa5dd75c0b39598b9, 0xdb35f7d4cfb6c56 },
                    { 0x603fbe7b2b332336, 0x2d3a703613184032, 0x593fe5d1d2060aa, 0x83b9e32a50456c1 } },
                  { { 0xd945386860c5c2d3, 0x1accb237b9906e26, 0x66374c896e5b0c43, 0x2176318cd44cb722 },
                    { 0xf2431be14b4df482, 0xc9bb05cb691445b8, 0xf02ed57856eb46bb, 0x16dbf34bb8373fd5 } },
                  { { 0x8d683ec33bd2d09f, 0xbb76c48d1ad7befe, 0xfc20598f07f9868f, 0x2251f84b9cb740d7 },
                    { 0x91137730616d416f, 0x7892e5f10d06fc71, 0x7115b23cadf2176, 0x243b593fe662d53 } } };
    fq6 result = a + b;
    EXPECT_EQ(result, expected);
}

TEST(fq6, sub_check_against_constants)
{
    fq6 a{ { { 0xa1167f5753441035, 0xc7712ba686dd96d, 0x1da0e185b8aa61a3, 0xc875cfdb65ae0b0 },
             { 0x69f9322c2f24bd33, 0x322a253d10e59171, 0xa661cfb9aaa595e8, 0x250efd5132c6f2be } },
           { { 0xe6fdc4742cbd9e3e, 0x5b0b304af8cdc721, 0x10a0156bb0bc7290, 0x256c970c31d4972d },
             { 0xb429fe8e4c500bd2, 0x191028f54b1d65f1, 0xe930ce25b92fbc40, 0x1203a8e8132f7703 } },
           { { 0x383cf556e7a8a1a1, 0x97aaa453a478cbf2, 0x5d51590dd12ab32c, 0x1cab1676bbef3e10 },
             { 0xee35e8a619840798, 0x39ec8a0397d6273f, 0x5b28d68b9ce7c868, 0x296cc5743d512a0d } } };
    fq6 b{ { { 0xda9d8930b9dbf521, 0xd0a38aaebc0e6255, 0xf849d807bb1c9d6d, 0x258ee7adb2034340 },
             { 0x67aabbf80885bd34, 0x4325cf38941244f0, 0xbd676021456927a9, 0xd5fd49a012953d8 } },
           { { 0xcf926e1ad2750891, 0x679470254ff702d4, 0x83fca49364e86848, 0xab3582e8e3e3c76 },
             { 0x5ffdc7f985ec6125, 0xd4e33612dc9b87cc, 0x779b43b3c26ed9e3, 0x472e0d2bc245c48 } },
           { { 0xb6a89730a6d59b36, 0xc1bf9260d155a94d, 0xc0db3b4a4d46dc11, 0x10d396b15aca55ae },
             { 0xfa34e14bae100b8a, 0x3f51a8e757d8a520, 0x693add3f9871ef91, 0x23dfee5479c36f12 } } };
    fq6 expected{ { { 0x299823d71e5185b, 0xd354f29d14d141a5, 0xdda74f347f0f1c92, 0x175cc3c2e5893d98 },
                    { 0x24e7634269effff, 0xef0456047cd34c81, 0xe8fa6f98653c6e3e, 0x17af28b7319d9ee5 } },
                  { { 0x176b56595a4895ad, 0xf376c025a8d6c44d, 0x8ca370d84bd40a47, 0x1ab93edda3965ab6 },
                    { 0x542c3694c663aaad, 0x442cf2e26e81de25, 0x71958a71f6c0e25c, 0xd90c815570b1abb } },
                  { { 0x81945e2640d3066b, 0xd5eb11f2d32322a4, 0x9c761dc383e3d71a, 0xbd77fc56124e861 },
                    { 0xf401075a6b73fc0e, 0xfa9ae11c3ffd821e, 0xf1edf94c0475d8d6, 0x58cd71fc38dbafa } } };
    fq6 result = a - b;
    EXPECT_EQ(result, expected);
}

TEST(fq6, mul_check_against_constants)
{
    fq6 a{ { { 0xa7e3494fc528b8c8, 0xc8c8906c9682e43f, 0xc6e76fc21152721c, 0x12a4c3ee3ff10dbd },
             { 0x887ce62a3ae2a578, 0x70caee28e1942bac, 0xc1a58242c34ff94f, 0x0b154d910b492542 } },
           { { 0x8c885006cc08667a, 0xee0b6c4a0dbb9592, 0xa755229d6272b51e, 0x2629b93f67eb8dd6 },
             { 0xe4ececfd79a858ea, 0xd68b54a557d3a745, 0x11583e88259747aa, 0x1843766463ee1ad4 } },
           { { 0x986c9795d5703de6, 0x57987869bb56865f, 0x42bb014b1256a07b, 0x1388b70999724ec8 },
             { 0x1ac82ca3e5494c3d, 0xc88fad298864779d, 0xc6d8b5505d04bdaa, 0x26426a0b70727a4d } } };
    fq6 b{ { { 0xdd7298233cd1b137, 0x026d2c68a832cb24, 0xbbab93115cbb50bb, 0x2aa110dfddd74cdf },
             { 0x4f4f74ad1223b0d8, 0x338a7892782a6432, 0xa5691849f99e6ac3, 0x24b57a6e0862e7fc } },
           { { 0x462cb81e0326de60, 0x4f43a8b4bd7ec7f3, 0xebdf4944e5d45872, 0x1bda04585d1e5734 },
             { 0xc88dc3fccb7def5a, 0x4363011c6f7e5ddf, 0x22ae64ed8c54e29d, 0x1ca466f35116b625 } },
           { { 0xc594e787e224746a, 0xc3d7d28811903ae5, 0xc56df7874c74120e, 0x252d38bf0b1e7e90 },
             { 0xbc5e4e1584480db3, 0x5adcfa1d2fca7c5e, 0xe47f725d6a7381e6, 0x04eb7a445804ec46 } } };
    fq6 expected{ { { 0x774f378764a25ae1, 0xa233ed1ce173ca63, 0xd17c631b84f48368, 0x0e09a8d87850c96e },
                    { 0x73ac39b5e9b9dbed, 0xf7f67afbfcbd26e2, 0x615245fa820a57f7, 0x0c752643305ac50f } },
                  { { 0xd106f17bee42abe1, 0xe29882030d238c60, 0x58001634ca9d66d1, 0x0ecae4532209b838 },
                    { 0x4b2fbc422420f06a, 0x3a8e5b388fdedd1f, 0x06006b4471134540, 0x0d4fee4f7966d63d } },
                  { { 0x4ffcbaa876979a1c, 0x32b7c1ef7d251306, 0x1b4e0712f969804e, 0x200592dfe71b710f },
                    { 0xe3eb378754bfb1ac, 0x6b517c1cae53d784, 0xd1b29c0eb1e4d46f, 0x08b42f13fdd14172 } } };
    fq6 result = a * b;
    EXPECT_EQ(result, expected);
}

TEST(fq6, sqr_check_against_constants)
{
    fq6 a{ { { 0xe337aaa063afce6, 0xff4b5477485eb20, 0xef6dcf13b3855ef8, 0x14554c38da988ece },
             { 0x6a70e65e71431416, 0xd21f95045c45f422, 0x2a17b6c6ff517884, 0x1b01ad6487a3ff16 } },
           { { 0xea39618e9f05e1f, 0x63e9b0f7803072a6, 0xebe5538a2c75c89, 0x5312aad2ac95dcf },
             { 0xc0750291a780ebcc, 0x782b1251b609f532, 0x316fe1bcf2fdde86, 0xb69f7f7ceddf296 } },
           { { 0xdd5e9baaefbf9d33, 0x43535c72f7a3525a, 0x494ed8ac61be0d8c, 0x16c1b965f69ff74e },
             { 0x9fc5c37ebff3efb4, 0xeeb16b0eb64816d6, 0xd619c0dfe2c33664, 0x21d9e29e8b1e6f81 } } };
    fq6 expected{ { { 0x3a40e1b9d96d12da, 0xd7aeb073f376a6d6, 0xf5c2c8663cae46fa, 0x2e0c927c52712062 },
                    { 0xcdb05809afd065ba, 0xbbfda09a79b7375d, 0xd3a86218cd3576be, 0x13b97e35c676e471 } },
                  { { 0x364e36385d045b9e, 0x4e5696665faa8544, 0xfe40998043ecbab, 0x443f608c40428e },
                    { 0xd48ac80d8e6e52b5, 0x1791b8c4145bc2d3, 0x35c456444cdcf9be, 0x1eddd29d77366c08 } },
                  { { 0x56f1f8acbaed1118, 0xdd74b8bb2e47de74, 0x97525aa49c65f0fd, 0x15bbf236e098fa0f },
                    { 0xad97a94142524aeb, 0x42a508523527268b, 0x4c9c5f213de06ca8, 0x73fa6bc31efa2f2 } } };
    fq6 result = a.sqr();
    EXPECT_EQ(result, expected);
}

TEST(fq6, to_montgomery_form)
{
    fq6 result = fq6::zero();
    result.c0.c0.data[0] = 1;
    fq6 expected = fq6::one();
    result = result.to_montgomery_form();
    EXPECT_EQ(result, expected);
}

TEST(fq6, from_montgomery_form)
{
    fq6 result = fq6::one();
    fq6 expected = fq6::zero();
    expected.c0.c0.data[0] = 1;
    result = result.from_montgomery_form();
    EXPECT_EQ(result, expected);
}

TEST(fq6, mul_sqr_consistency)
{
    fq6 a = fq6::random_element();
    fq6 b = fq6::random_element();
    fq6 t1 = a - b;
    fq6 t2 = a + b;
    fq6 mul_result = t1 * t2;
    fq6 sqr_result = a.sqr() - b.sqr();

    EXPECT_EQ(mul_result, sqr_result);
}

TEST(fq6, add_mul_consistency)
{
    fq6 multiplicand = fq6::zero();
    multiplicand.c0.c0.data[0] = 9;
    multiplicand = multiplicand.to_montgomery_form();

    fq6 a = fq6::random_element();
    fq6 result = a + a;
    result += result;
    result += result;
    result += a;

    fq6 expected = a * multiplicand;
    EXPECT_EQ(result, expected);
}

TEST(fq6, sub_mul_consistency)
{
    fq6 multiplicand = fq6::zero();
    multiplicand.c0.c0.data[0] = 5;
    multiplicand = multiplicand.to_montgomery_form();
    fq6 a = fq6::random_element();
    fq6 result = a + a;
    result += result;
    result += result;
    result -= a;
    result -= a;
    result -= a;

    fq6 expected = a * multiplicand;

    EXPECT_EQ(result, expected);
}

TEST(fq6, invert)
{
    fq6 input = fq6::random_element();
    fq6 result = input.invert();

    result *= input;
    EXPECT_EQ(result, fq6::one());
}

TEST(fq6, copy)
{
    fq6 result = fq6::random_element();
    fq6 expected = result;
    EXPECT_EQ(result, expected);
}