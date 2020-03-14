#include "grumpkin.hpp"
#include <gtest/gtest.h>

namespace test_grumpkin {

TEST(grumpkin, random_element)
{
    grumpkin::g1::element result = grumpkin::g1::element::random_element();
    EXPECT_EQ(result.on_curve(), true);
}

TEST(grumpkin, random_affine_element)
{
    grumpkin::g1::affine_element result = grumpkin::g1::affine_element(grumpkin::g1::element::random_element());
    EXPECT_EQ(result.on_curve(), true);
}

TEST(grumpkin, eq)
{
    grumpkin::g1::element a = grumpkin::g1::element::random_element();
    grumpkin::g1::element b = a.normalize();

    EXPECT_EQ(a == b, true);
    EXPECT_EQ(a == a, true);

    b.self_set_infinity();

    EXPECT_EQ(a == b, false);
    grumpkin::g1::element c = grumpkin::g1::element::random_element();

    EXPECT_EQ(a == c, false);

    a.self_set_infinity();

    EXPECT_EQ(a == b, true);
}

TEST(grumpkin, check_group_modulus)
{
    // grumpkin::g1::affine_element expected = grumpkin::g1::affine_one;
    grumpkin::fr exponent = -grumpkin::fr(1);
    grumpkin::g1::element result = grumpkin::g1::one * exponent;
    result += grumpkin::g1::one;
    result += grumpkin::g1::one;
    EXPECT_EQ(result.on_curve(), true);
    EXPECT_EQ(result == grumpkin::g1::one, true);
}

// TEST(grumpkin, mixed_add_check_against_constants)
// {
//     fq a_x = {{0x92716caa6cac6d26, 0x1e6e234136736544, 0x1bb04588cde00af0, 0x9a2ac922d97e6f5}};
//     fq a_y = {{0x9e693aeb52d79d2d, 0xf0c1895a61e5e975, 0x18cd7f5310ced70f, 0xac67920a22939ad}};
//     fq a_z = {{0xfef593c9ce1df132, 0xe0486f801303c27d, 0x9bbd01ab881dc08e, 0x2a589badf38ec0f9}};
//     fq b_x = {{0xa1ec5d1398660db8, 0x6be3e1f6fd5d8ab1, 0x69173397dd272e11, 0x12575bbfe1198886}};
//     fq b_y = {{0xcfbfd4441138823e, 0xb5f817e28a1ef904, 0xefb7c5629dcc1c42, 0x1a9ed3d6f846230e}};
//     fq expected_x = {{0x2a9d0201fccca20, 0x36f969b294f31776, 0xee5534422a6f646, 0x911dbc6b02310b6}};
//     fq expected_y = {{0x14c30aaeb4f135ef, 0x9c27c128ea2017a1, 0xf9b7d80c8315eabf, 0x35e628df8add760}};
//     fq expected_z = {{0xa43fe96673d10eb3, 0x88fbe6351753d410, 0x45c21cc9d99cb7d, 0x3018020aa6e9ede5}};
//     grumpkin::g1::element lhs;
//     grumpkin::g1::affine_element rhs;
//     grumpkin::g1::element result;
//     grumpkin::g1::element expected;
//     fq::__to_montgomery_form(a_x, lhs.x);
//     fq::__to_montgomery_form(a_y, lhs.y);
//     fq::__to_montgomery_form(a_z, lhs.z);
//     fq::__to_montgomery_form(b_x, rhs.x);
//     fq::__to_montgomery_form(b_y, rhs.y);
//     fq::__to_montgomery_form(expected_x, expected.x);
//     fq::__to_montgomery_form(expected_y, expected.y);
//     fq::__to_montgomery_form(expected_z, expected.z);
//     result = lhs + rhs;

//     EXPECT_EQ(result == expected, true);
// }

// TEST(grumpkin, dbl_check_against_constants)
// {
//     fq a_x = {{0x8d1703aa518d827f, 0xd19cc40779f54f63, 0xabc11ce30d02728c, 0x10938940de3cbeec}};
//     fq a_y = {{0xcf1798994f1258b4, 0x36307a354ad90a25, 0xcd84adb348c63007, 0x6266b85241aff3f}};
//     fq a_z = {{0xe213e18fd2df7044, 0xb2f42355982c5bc8, 0xf65cf5150a3a9da1, 0xc43bde08b03aca2}};
//     fq expected_x = {{0xd5c6473044b2e67c, 0x89b185ea20951f3a, 0x4ac597219cf47467, 0x2d00482f63b12c86}};
//     fq expected_y = {{0x4e7e6c06a87e4314, 0x906a877a71735161, 0xaa7b9893cc370d39, 0x62f206bef795a05}};
//     fq expected_z = {{0x8813bdca7b0b115a, 0x929104dffdfabd22, 0x3fff575136879112, 0x18a299c1f683bdca}};
//     grumpkin::g1::element lhs;
//     grumpkin::g1::element result;
//     grumpkin::g1::element expected;
//     fq::__to_montgomery_form(a_x, lhs.x);
//     fq::__to_montgomery_form(a_y, lhs.y);
//     fq::__to_montgomery_form(a_z, lhs.z);
//     fq::__to_montgomery_form(expected_x, expected.x);
//     fq::__to_montgomery_form(expected_y, expected.y);
//     fq::__to_montgomery_form(expected_z, expected.z);

//     result = lhs.dbl();
//     result.self_dbl();
//     result.self_dbl();

//     EXPECT_EQ(result == expected, true);
// }

// TEST(grumpkin, add_check_against_constants)
// {
//     fq a_x = {{0x184b38afc6e2e09a, 0x4965cd1c3687f635, 0x334da8e7539e71c4, 0xf708d16cfe6e14}};
//     fq a_y = {{0x2a6ff6ffc739b3b6, 0x70761d618b513b9, 0xbf1645401de26ba1, 0x114a1616c164b980}};
//     fq a_z = {{0x10143ade26bbd57a, 0x98cf4e1f6c214053, 0x6bfdc534f6b00006, 0x1875e5068ababf2c}};
//     fq b_x = {{0xafdb8a15c98bf74c, 0xac54df622a8d991a, 0xc6e5ae1f3dad4ec8, 0x1bd3fb4a59e19b52}};
//     fq b_y = {{0x21b3bb529bec20c0, 0xaabd496406ffb8c1, 0xcd3526c26ac5bdcb, 0x187ada6b8693c184}};
//     fq b_z = {{0xffcd440a228ed652, 0x8a795c8f234145f1, 0xd5279cdbabb05b95, 0xbdf19ba16fc607a}};
//     fq expected_x = {{0x18764da36aa4cd81, 0xd15388d1fea9f3d3, 0xeb7c437de4bbd748, 0x2f09b712adf6f18f}};
//     fq expected_y = {{0x50c5f3cab191498c, 0xe50aa3ce802ea3b5, 0xd9d6125b82ebeff8, 0x27e91ba0686e54fe}};
//     fq expected_z = {{0xe4b81ef75fedf95, 0xf608edef14913c75, 0xfd9e178143224c96, 0xa8ae44990c8accd}};
//     grumpkin::g1::element lhs;
//     grumpkin::g1::element rhs;
//     grumpkin::g1::element result;
//     grumpkin::g1::element expected;

//     fq::__to_montgomery_form(a_x, lhs.x);
//     fq::__to_montgomery_form(a_y, lhs.y);
//     fq::__to_montgomery_form(a_z, lhs.z);
//     fq::__to_montgomery_form(b_x, rhs.x);
//     fq::__to_montgomery_form(b_y, rhs.y);
//     fq::__to_montgomery_form(b_z, rhs.z);
//     fq::__to_montgomery_form(expected_x, expected.x);
//     fq::__to_montgomery_form(expected_y, expected.y);
//     fq::__to_montgomery_form(expected_z, expected.z);

//     result = lhs + rhs;

//     EXPECT_EQ(result == expected, true);
// }

TEST(grumpkin, add_exception_test_infinity)
{
    grumpkin::g1::element lhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element rhs;
    grumpkin::g1::element result;

    rhs = -lhs;

    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    grumpkin::g1::element rhs_b;
    rhs_b = rhs;
    rhs_b.self_set_infinity();

    result = lhs + rhs_b;

    EXPECT_EQ(lhs == result, true);

    lhs.self_set_infinity();
    result = lhs + rhs;

    EXPECT_EQ(rhs == result, true);
}

TEST(grumpkin, add_exception_test_dbl)
{
    grumpkin::g1::element lhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element rhs;
    rhs = lhs;

    grumpkin::g1::element result;
    grumpkin::g1::element expected;

    result = lhs + rhs;
    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, add_dbl_consistency)
{
    grumpkin::g1::element a = grumpkin::g1::element::random_element();
    grumpkin::g1::element b = grumpkin::g1::element::random_element();

    grumpkin::g1::element c;
    grumpkin::g1::element d;
    grumpkin::g1::element add_result;
    grumpkin::g1::element dbl_result;

    c = a + b;
    b = -b;
    d = a + b;

    add_result = c + d;
    dbl_result = a.dbl();

    EXPECT_EQ(add_result == dbl_result, true);
}

TEST(grumpkin, add_dbl_consistency_repeated)
{
    grumpkin::g1::element a = grumpkin::g1::element::random_element();
    grumpkin::g1::element b;
    grumpkin::g1::element c;
    grumpkin::g1::element d;
    grumpkin::g1::element e;

    grumpkin::g1::element result;
    grumpkin::g1::element expected;

    b = a.dbl(); // b = 2a
    c = b.dbl(); // c = 4a

    d = a + b;      // d = 3a
    e = a + c;      // e = 5a
    result = d + e; // result = 8a

    expected = c.dbl(); // expected = 8a

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, mixed_add_exception_test_infinity)
{
    grumpkin::g1::element lhs = grumpkin::g1::one;
    grumpkin::g1::affine_element rhs = grumpkin::g1::affine_element(grumpkin::g1::element::random_element());
    grumpkin::fq::__copy(rhs.x, lhs.x);
    lhs.y = -rhs.y;

    grumpkin::g1::element result;
    result = lhs + rhs;

    EXPECT_EQ(result.is_point_at_infinity(), true);

    lhs.self_set_infinity();
    result = lhs + rhs;
    grumpkin::g1::element rhs_c;
    rhs_c = grumpkin::g1::element(rhs);

    EXPECT_EQ(rhs_c == result, true);
}

TEST(grumpkin, mixed_add_exception_test_dbl)
{
    grumpkin::g1::affine_element rhs = grumpkin::g1::affine_element(grumpkin::g1::element::random_element());
    grumpkin::g1::element lhs;
    lhs = grumpkin::g1::element(rhs);

    grumpkin::g1::element result;
    grumpkin::g1::element expected;
    result = lhs + rhs;

    expected = lhs.dbl();

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, add_mixed_add_consistency_check)
{
    grumpkin::g1::affine_element rhs = grumpkin::g1::affine_element(grumpkin::g1::element::random_element());
    grumpkin::g1::element lhs = grumpkin::g1::element::random_element();
    grumpkin::g1::element rhs_b;
    rhs_b = grumpkin::g1::element(rhs);

    grumpkin::g1::element add_result;
    grumpkin::g1::element mixed_add_result;
    add_result = lhs + rhs_b;
    mixed_add_result = lhs + rhs;

    EXPECT_EQ(add_result == mixed_add_result, true);
}

TEST(grumpkin, on_curve)
{
    for (size_t i = 0; i < 100; ++i) {
        grumpkin::g1::element test = grumpkin::g1::element::random_element();
        EXPECT_EQ(test.on_curve(), true);
        grumpkin::g1::affine_element affine_test =
            grumpkin::g1::affine_element(grumpkin::g1::element::random_element());
        EXPECT_EQ(affine_test.on_curve(), true);
    }
}
TEST(grumpkin, batch_normalize)
{
    size_t num_points = 2;
    grumpkin::g1::element points[num_points];
    grumpkin::g1::element normalized[num_points];
    for (size_t i = 0; i < num_points; ++i) {
        grumpkin::g1::element a = grumpkin::g1::element::random_element();
        grumpkin::g1::element b = grumpkin::g1::element::random_element();
        points[i] = a + b;
        normalized[i] = points[i];
    }
    grumpkin::g1::element::batch_normalize(normalized, num_points);

    for (size_t i = 0; i < num_points; ++i) {
        grumpkin::fq zz;
        grumpkin::fq zzz;
        grumpkin::fq result_x;
        grumpkin::fq result_y;
        zz = points[i].z.sqr();
        zzz = points[i].z * zz;
        result_x = normalized[i].x * zz;
        result_y = normalized[i].y * zzz;

        EXPECT_EQ((result_x == points[i].x), true);
        EXPECT_EQ((result_y == points[i].y), true);
    }
}

TEST(grumpkin, group_exponentiation_zero_and_one)
{
    grumpkin::g1::affine_element result = grumpkin::g1::one * grumpkin::fr::zero();

    EXPECT_EQ(result.is_point_at_infinity(), true);

    result = grumpkin::g1::one * grumpkin::fr::one();

    EXPECT_EQ(result == grumpkin::g1::affine_one, true);
}

TEST(grumpkin, group_exponentiation_consistency_check)
{
    grumpkin::fr a = grumpkin::fr::random_element();
    grumpkin::fr b = grumpkin::fr::random_element();

    grumpkin::fr c;
    c = a * b;

    grumpkin::g1::affine_element input = grumpkin::g1::affine_one;
    grumpkin::g1::affine_element result = input * a;
    result = result * b;

    grumpkin::g1::affine_element expected = input * c;

    EXPECT_EQ(result == expected, true);
}

TEST(grumpkin, derive_generators)
{
    constexpr size_t num_generators = 128;
    std::array<grumpkin::g1::affine_element, num_generators> result = grumpkin::g1::derive_generators<num_generators>();

    const auto is_unique = [&result](const grumpkin::g1::affine_element& y, const size_t j) {
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
} // namespace test_grumpkin