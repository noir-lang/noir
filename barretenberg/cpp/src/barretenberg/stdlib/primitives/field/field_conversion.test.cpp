#include "barretenberg/stdlib/primitives/field/field_conversion.hpp"
#include "barretenberg/common/zip_view.hpp"
#include <gtest/gtest.h>

namespace bb::stdlib::field_conversion_tests {

template <typename Builder> using fr = field_t<Builder>;
template <typename Builder> using fq = bigfield<Builder, bb::Bn254FqParams>;
template <typename Builder> using bn254_element = element<Builder, fq<Builder>, fr<Builder>, curve::BN254::Group>;
template <typename Builder> using grumpkin_element = cycle_group<Builder>;

template <typename Builder> class StdlibFieldConversionTests : public ::testing::Test {
  public:
    template <typename T> void check_conversion(Builder& builder, T x)
    {
        size_t len = bb::stdlib::field_conversion::calc_num_bn254_frs<Builder, T>();
        auto frs = bb::stdlib::field_conversion::convert_to_bn254_frs<Builder, T>(x);
        EXPECT_EQ(len, frs.size());
        auto y = bb::stdlib::field_conversion::convert_from_bn254_frs<Builder, T>(builder, frs);
        EXPECT_EQ(x.get_value(), y.get_value());
    }

    template <typename T> void check_conversion_iterable(Builder& builder, T x)
    {
        size_t len = bb::stdlib::field_conversion::calc_num_bn254_frs<Builder, T>();
        auto frs = bb::stdlib::field_conversion::convert_to_bn254_frs<Builder, T>(x);
        EXPECT_EQ(len, frs.size());
        auto y = bb::stdlib::field_conversion::convert_from_bn254_frs<Builder, T>(builder, frs);
        EXPECT_EQ(x.size(), y.size());
        for (auto [val1, val2] : zip_view(x, y)) {
            EXPECT_EQ(val1.get_value(), val2.get_value());
        }
    }
};

using BuilderTypes = testing::Types<UltraCircuitBuilder, GoblinUltraCircuitBuilder>;

TYPED_TEST_SUITE(StdlibFieldConversionTests, BuilderTypes);

/**
 * @brief Field conversion test for fr<Builder>
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionFr)
{
    using Builder = TypeParam;
    Builder builder;
    bb::fr x1_val(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")); // 256 bits
    fr<Builder> x1(&builder, x1_val);
    this->check_conversion(builder, x1);

    bb::fr x2_val(bb::fr::modulus_minus_two); // modulus - 2
    fr<Builder> x2(&builder, x2_val);
    this->check_conversion(builder, x2);

    bb::fr x3_val(1);
    fr<Builder> x3(&builder, x3_val);
    this->check_conversion(builder, x3);
}

/**
 * @brief Field conversion test for fq<Builder>
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionGrumpkinFr)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing bigfield objects with grumpkin::fr values
    grumpkin::fr x1_val(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")); // 256 bits
    fq<Builder> x1(&builder, x1_val);
    this->check_conversion(builder, x1);
}

/**
 * @brief Field conversion test for bn254_element<Builder>
 *
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionBN254AffineElement)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing element objects with curve::BN254::AffineElement values
    curve::BN254::AffineElement x1_val(1, 2);
    bn254_element<Builder> x1 = bn254_element<Builder>::from_witness(&builder, x1_val);
    this->check_conversion(builder, x1);

    curve::BN254::AffineElement x2_val(1, grumpkin::fr::modulus_minus_two);
    bn254_element<Builder> x2 = bn254_element<Builder>::from_witness(&builder, x2_val);
    this->check_conversion(builder, x2);
}

/**
 * @brief Field conversion test for grumpkin_element<Builder>
 *
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionGrumpkinAffineElement)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing element objects with curve::Grumpkin::AffineElement values
    curve::Grumpkin::AffineElement x1_val(12, 100);
    grumpkin_element<Builder> x1 = grumpkin_element<Builder>::from_witness(&builder, x1_val);
    this->check_conversion(builder, x1);

    curve::Grumpkin::AffineElement x2_val(1, grumpkin::fr::modulus_minus_two);
    grumpkin_element<Builder> x2 = grumpkin_element<Builder>::from_witness(&builder, x2_val);
    this->check_conversion(builder, x2);
}

/**
 * @brief Field conversion test for std::array<fr<Builder>, N>
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionArrayBn254Fr)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing std::array objects with fr<Builder> values
    std::array<fr<Builder>, 4> x1{
        fr<Builder>(&builder, 1), fr<Builder>(&builder, 2), fr<Builder>(&builder, 3), fr<Builder>(&builder, 4)
    };
    this->check_conversion_iterable(builder, x1);

    std::array<fr<Builder>, 7> x2{ fr<Builder>(&builder, bb::fr::modulus_minus_two),
                                   fr<Builder>(&builder, bb::fr::modulus_minus_two - 123),
                                   fr<Builder>(&builder, 215215125),
                                   fr<Builder>(&builder, 102701750),
                                   fr<Builder>(&builder, 367032),
                                   fr<Builder>(&builder, 12985028),
                                   fr<Builder>(&builder, bb::fr::modulus_minus_two - 125015028) };
    this->check_conversion_iterable(builder, x2);
}

/**
 * @brief Field conversion test for std::array<fq<Builder>, N>
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionArrayGrumpkinFr)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing std::array objects with fq<Builder> values
    std::array<fq<Builder>, 4> x1{
        fq<Builder>(
            &builder,
            static_cast<grumpkin::fr>(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"))),
        fq<Builder>(
            &builder,
            static_cast<grumpkin::fr>(std::string("2bf1eaf87f7d27e8dc4056e9af975985bccc89077a21891d6c7b6ccce0631f95"))),
        fq<Builder>(
            &builder,
            static_cast<grumpkin::fr>(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"))),
        fq<Builder>(
            &builder,
            static_cast<grumpkin::fr>(std::string("018555a8eb50cf07f64b019ebaf3af3c925c93e631f3ecd455db07bbb52bbdd3"))),
    };
    this->check_conversion_iterable(builder, x1);
}

/**
 * @brief Field conversion test for Univariate<fr<Builder>, N>
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionUnivariateBn254Fr)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing Univariate objects with fr<Builder> values
    Univariate<fr<Builder>, 4> x{
        { fr<Builder>(&builder, 1), fr<Builder>(&builder, 2), fr<Builder>(&builder, 3), fr<Builder>(&builder, 4) }
    };
    this->check_conversion_iterable(builder, x);
}

/**
 * @brief Field conversion test for Univariate<fq<Builder>, N>
 */
TYPED_TEST(StdlibFieldConversionTests, FieldConversionUnivariateGrumpkinFr)
{
    using Builder = TypeParam;
    Builder builder;

    // Constructing std::array objects with fq<Builder> values
    Univariate<fq<Builder>, 4> x{
        { fq<Builder>(&builder,
                      static_cast<grumpkin::fr>(
                          std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"))),
          fq<Builder>(&builder,
                      static_cast<grumpkin::fr>(
                          std::string("2bf1eaf87f7d27e8dc4056e9af975985bccc89077a21891d6c7b6ccce0631f95"))),
          fq<Builder>(&builder,
                      static_cast<grumpkin::fr>(
                          std::string("018555a8eb50cf07f64b019ebaf3af3c925c93e631f3ecd455db07bbb52bbdd3"))),
          fq<Builder>(&builder,
                      static_cast<grumpkin::fr>(
                          std::string("2bf1eaf87f7d27e8dc4056e9af975985bccc89077a21891d6c7b6ccce0631f95"))) }
    };
    this->check_conversion_iterable(builder, x);
}

/**
 * @brief Convert challenge test for fq<Builder>
 *
 */
TYPED_TEST(StdlibFieldConversionTests, ConvertChallengeGrumpkinFr)
{
    using Builder = TypeParam;
    Builder builder;

    bb::fr chal_val(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789")); // 256 bits
    auto chal = fr<Builder>::from_witness(&builder, chal_val);
    auto result = bb::stdlib::field_conversion::convert_challenge<Builder, fq<Builder>>(builder, chal);
    auto expected = uint256_t(chal.get_value());
    EXPECT_EQ(uint256_t(result.get_value()), expected);
}
} // namespace bb::stdlib::field_conversion_tests