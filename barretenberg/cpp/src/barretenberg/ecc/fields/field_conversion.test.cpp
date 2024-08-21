#include "barretenberg/ecc/fields/field_conversion.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <gtest/gtest.h>

namespace bb::field_conversion_tests {

class FieldConversionTest : public ::testing::Test {
  public:
    template <typename T> void check_conversion(T x)
    {
        size_t len = bb::field_conversion::calc_num_bn254_frs<T>();
        auto frs = bb::field_conversion::convert_to_bn254_frs(x);
        EXPECT_EQ(len, frs.size());
        auto y = bb::field_conversion::convert_from_bn254_frs<T>(frs);
        EXPECT_EQ(x, y);
    }
};

/**
 * @brief Field conversion test for uint32_t
 */
TEST_F(FieldConversionTest, FieldConversionUint32)
{
    auto x = static_cast<uint32_t>(1) << 31;
    check_conversion(x);
}

/**
 * @brief Field conversion test for bb::fr
 */
TEST_F(FieldConversionTest, FieldConversionFr)
{
    bb::fr x1(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")); // 256 bits
    check_conversion(x1);

    bb::fr x2(bb::fr::modulus_minus_two); // modulus - 2
    check_conversion(x2);
}

/**
 * @brief Field conversion test for grumpkin::fr
 *
 */
TEST_F(FieldConversionTest, FieldConversionGrumpkinFr)
{
    grumpkin::fr x1(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")); // 256 bits
    check_conversion(x1);
}

namespace {
bb::fq derive_bn254_y(bb::fq x)
{
    auto [found, y] = (x.sqr() * x + Bn254G1Params::b).sqrt();
    EXPECT_TRUE(found);
    return y;
}
} // namespace

/**
 * @brief Field conversion test for curve::BN254::AffineElement
 *
 */
TEST_F(FieldConversionTest, FieldConversionBN254AffineElement)
{
    curve::BN254::AffineElement x1(1, derive_bn254_y(1));
    check_conversion(x1);

    curve::BN254::AffineElement x2(grumpkin::fr::modulus_minus_two, derive_bn254_y(grumpkin::fr::modulus_minus_two));
    check_conversion(x2);
}

namespace {
bb::grumpkin::fq derive_grumpkin_y(bb::grumpkin::fq x)
{
    auto [found, y] = (x.sqr() * x + grumpkin::G1Params::b + x * grumpkin::G1Params::a).sqrt();
    EXPECT_TRUE(found);
    return y;
}
} // namespace

/**
 * @brief Field conversion test for curve::Grumpkin::AffineElement
 */
TEST_F(FieldConversionTest, FieldConversionGrumpkinAffineElement)
{
    curve::Grumpkin::AffineElement x1(1, derive_grumpkin_y(1));
    check_conversion(x1);

    curve::Grumpkin::AffineElement x2(bb::fr::modulus_minus_two, derive_grumpkin_y(bb::fr::modulus_minus_two));
    check_conversion(x2);
}

/**
 * @brief Field conversion test for std::array<bb::fr, N>
 *
 */
TEST_F(FieldConversionTest, FieldConversionArrayBn254Fr)
{
    std::array<bb::fr, 4> x1{ 1, 2, 3, 4 };
    check_conversion(x1);

    std::array<bb::fr, 7> x2{ bb::fr::modulus_minus_two,
                              bb::fr::modulus_minus_two - 123,
                              215215125,
                              102701750,
                              367032,
                              12985028,
                              bb::fr::modulus_minus_two - 125015028 };
    check_conversion(x2);
}

/**
 * @brief Field conversion test for std::array<grumpkin::fr, N>
 *
 */
TEST_F(FieldConversionTest, FieldConversionArrayGrumpkinFr)
{
    std::array<grumpkin::fr, 4> x1{ 1, 2, 3, 4 };
    check_conversion(x1);

    std::array<grumpkin::fr, 7> x2{ grumpkin::fr::modulus_minus_two,
                                    grumpkin::fr::modulus_minus_two - 123,
                                    215215125,
                                    102701750,
                                    367032,
                                    12985028,
                                    grumpkin::fr::modulus_minus_two - 125015028 };
    check_conversion(x2);
}

/**
 * @brief Field conversion test for bb::Univariate<bb::fr, N>
 *
 */
TEST_F(FieldConversionTest, FieldConversionUnivariateBn254Fr)
{
    std::array<bb::fr, 4> x1_arr{ 1, 2, 3, 4 };
    bb::Univariate<bb::fr, 4> x1{ x1_arr };
    check_conversion(x1);
}

/**
 * @brief Field conversion test for bb::Univariate<grumpkin::fr, N>
 *
 */
TEST_F(FieldConversionTest, FieldConversionUnivariateGrumpkinFr)
{
    std::array<grumpkin::fr, 4> x1_arr{ 1, 2, 3, 4 };
    bb::Univariate<grumpkin::fr, 4> x1{ x1_arr };
    check_conversion(x1);
}

/**
 * @brief Convert challenge test for grumpkin::fr
 *
 */
TEST_F(FieldConversionTest, ConvertChallengeGrumpkinFr)
{
    bb::fr chal(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")); // 256 bits
    auto result = bb::field_conversion::convert_challenge<grumpkin::fr>(chal);
    auto expected = uint256_t(chal);
    EXPECT_EQ(uint256_t(result), expected);
}

} // namespace bb::field_conversion_tests
