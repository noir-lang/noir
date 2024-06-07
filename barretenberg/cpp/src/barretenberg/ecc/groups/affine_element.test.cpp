#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/curves/secp256k1/secp256k1.hpp"
#include "barretenberg/ecc/curves/secp256r1/secp256r1.hpp"
#include "barretenberg/ecc/groups/element.hpp"
#include "barretenberg/serialize/test_helper.hpp"

#include "gmock/gmock.h"
#include <algorithm>
#include <fstream>
#include <gtest/gtest.h>
#include <iterator>
#include <tuple>

using ::testing::Each;
using ::testing::ElementsAreArray;
using ::testing::Eq;
using ::testing::Property;

using namespace bb;

namespace {
template <typename G1> class TestAffineElement : public testing::Test {
    using element = typename G1::element;
    using affine_element = typename G1::affine_element;

  public:
    static void test_read_write_buffer()
    {
        // a generic point
        {
            affine_element P = affine_element(element::random_element());
            affine_element Q;
            affine_element R;

            std::vector<uint8_t> v(65); // extra byte to allow a bad read
            uint8_t* ptr = v.data();
            affine_element::serialize_to_buffer(P, ptr);

            // bad read
            Q = affine_element::serialize_from_buffer(ptr + 1);
            ASSERT_FALSE(Q.on_curve() && !Q.is_point_at_infinity());
            ASSERT_FALSE(P == Q);

            // good read
            R = affine_element::serialize_from_buffer(ptr);
            ASSERT_TRUE(R.on_curve());
            ASSERT_TRUE(P == R);
        }

        // point at infinity
        {
            affine_element P = affine_element(element::random_element());
            P.self_set_infinity();
            affine_element R;

            std::vector<uint8_t> v(64);
            uint8_t* ptr = v.data();
            affine_element::serialize_to_buffer(P, ptr);

            R = affine_element::serialize_from_buffer(ptr);
            ASSERT_TRUE(R.is_point_at_infinity());
            ASSERT_TRUE(P == R);
        }
    }

    static void test_read_and_write()
    {
        // a generic point
        {
            affine_element P = affine_element(element::random_element());
            [[maybe_unused]] affine_element R;

            std::vector<uint8_t> v(sizeof(R));
            uint8_t* ptr = v.data();
            write(ptr, P);
            ASSERT_TRUE(P.on_curve());

            // // Reset to start?
            // ptr = v.data();

            const uint8_t* read_ptr = v.data();
            // good read
            read(read_ptr, R);
            ASSERT_TRUE(R.on_curve());
            ASSERT_TRUE(P == R);
        }
    }

    static void test_point_compression()
    {
        for (size_t i = 0; i < 10; i++) {
            affine_element P = affine_element(element::random_element());
            uint256_t compressed = P.compress();
            affine_element Q = affine_element::from_compressed(compressed);
            EXPECT_EQ(P, Q);
        }
    }

    static void test_point_compression_unsafe()
    {
        for (size_t i = 0; i < 100; i++) {
            affine_element P = affine_element(element::random_element());
            uint256_t compressed = uint256_t(P.x);

            // Note that we do not check the point Q_points[1] because its highly unlikely to hit a point P on the curve
            // such that r < P.x < q.
            std::array<affine_element, 2> Q_points = affine_element::from_compressed_unsafe(compressed);
            EXPECT_EQ(P, Q_points[0]);
        }
    }

    // Regression test to ensure that the point at infinity is not equal to its coordinate-wise reduction, which may lie
    // on the curve, depending on the y-coordinate.
    // TODO(@Rumata888): add corresponding typed test class
    static void test_infinity_regression()
    {
        affine_element P;
        P.self_set_infinity();
        affine_element R(0, P.y);
        ASSERT_FALSE(P == R);
    }
    // Regression test to ensure that the point at infinity is not equal to its coordinate-wise reduction, which may lie
    // on the curve, depending on the y-coordinate.
    static void test_infinity_ordering_regression()
    {
        affine_element P(0, 1);
        affine_element Q(0, 1);

        P.self_set_infinity();
        EXPECT_NE(P < Q, Q < P);
    }

    /**
     * @brief A regression test to make sure the -1 case is covered
     *
     */
    static void test_batch_endomorphism_by_minus_one()
    {
        constexpr size_t num_points = 2;
        std::vector<affine_element> affine_points(num_points, affine_element::one());

        std::vector<affine_element> result =
            element::batch_mul_with_endomorphism(affine_points, -affine_element::Fr::one());

        for (size_t i = 0; i < num_points; i++) {
            EXPECT_EQ(affine_points[i], -result[i]);
        }
    }
};

using TestTypes = testing::Types<bb::g1>;
// using TestTypes = testing::Types<bb::g1, grumpkin::g1, secp256k1::g1, secp256r1::g1>;
} // namespace

TYPED_TEST_SUITE(TestAffineElement, TestTypes);

TYPED_TEST(TestAffineElement, ReadWrite)
{
    TestFixture::test_read_and_write();
}

TYPED_TEST(TestAffineElement, ReadWriteBuffer)
{
    TestFixture::test_read_write_buffer();
}

TYPED_TEST(TestAffineElement, PointCompression)
{
    if constexpr (TypeParam::Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        GTEST_SKIP();
    } else {
        TestFixture::test_point_compression();
    }
}

TYPED_TEST(TestAffineElement, PointCompressionUnsafe)
{
    if constexpr (TypeParam::Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        TestFixture::test_point_compression_unsafe();
    } else {
        GTEST_SKIP();
    }
}

TYPED_TEST(TestAffineElement, InfinityOrderingRegression)
{
    TestFixture::test_infinity_ordering_regression();
}

namespace bb::group_elements {
// mul_with_endomorphism and mul_without_endomorphism are private in affine_element.
// We could make those public to test or create other public utilities, but to keep the API intact we
// instead mark TestElementPrivate as a friend class so that our test functions can have access.
class TestElementPrivate {
  public:
    template <typename Element, typename Scalar>
    static Element mul_without_endomorphism(const Element& element, const Scalar& scalar)
    {
        return element.mul_without_endomorphism(scalar);
    }
    template <typename Element, typename Scalar>
    static Element mul_with_endomorphism(const Element& element, const Scalar& scalar)
    {
        return element.mul_with_endomorphism(scalar);
    }
};
} // namespace bb::group_elements

// Our endomorphism-specialized multiplication should match our generic multiplication
TYPED_TEST(TestAffineElement, MulWithEndomorphismMatchesMulWithoutEndomorphism)
{
    for (int i = 0; i < 100; i++) {
        auto x1 = bb::group_elements::element(grumpkin::g1::affine_element::random_element());
        auto f1 = grumpkin::fr::random_element();
        auto r1 = bb::group_elements::TestElementPrivate::mul_without_endomorphism(x1, f1);
        auto r2 = bb::group_elements::TestElementPrivate::mul_with_endomorphism(x1, f1);
        EXPECT_EQ(r1, r2);
    }
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/909): These tests are not typed for no reason
// Multiplication of a point at infinity by a scalar should be a point at infinity
TEST(AffineElement, InfinityMulByScalarIsInfinity)
{
    auto result = grumpkin::g1::affine_element::infinity() * grumpkin::fr::random_element();
    EXPECT_TRUE(result.is_point_at_infinity());
}

// Batched multiplication of points should match
TEST(AffineElement, BatchMulMatchesNonBatchMul)
{
    constexpr size_t num_points = 512;
    std::vector<grumpkin::g1::affine_element> affine_points(num_points - 1, grumpkin::g1::affine_element::infinity());
    // Include a point at infinity to test the mixed infinity + non-infinity case
    affine_points.push_back(grumpkin::g1::affine_element::infinity());
    grumpkin::fr exponent = grumpkin::fr::random_element();
    std::vector<grumpkin::g1::affine_element> expected;
    std::transform(affine_points.begin(),
                   affine_points.end(),
                   std::back_inserter(expected),
                   [exponent](const auto& el) { return el * exponent; });

    std::vector<grumpkin::g1::affine_element> result =
        grumpkin::g1::element::batch_mul_with_endomorphism(affine_points, exponent);

    EXPECT_THAT(result, ElementsAreArray(expected));
}

// Batched multiplication of a point at infinity by a scalar should result in points at infinity
TEST(AffineElement, InfinityBatchMulByScalarIsInfinity)
{
    constexpr size_t num_points = 1024;
    std::vector<grumpkin::g1::affine_element> affine_points(num_points, grumpkin::g1::affine_element::infinity());

    std::vector<grumpkin::g1::affine_element> result =
        grumpkin::g1::element::batch_mul_with_endomorphism(affine_points, grumpkin::fr::random_element());

    EXPECT_THAT(result, Each(Property(&grumpkin::g1::affine_element::is_point_at_infinity, Eq(true))));
}

TYPED_TEST(TestAffineElement, BatchEndomoprhismByMinusOne)
{
    if constexpr (TypeParam::USE_ENDOMORPHISM) {
        TestFixture::test_batch_endomorphism_by_minus_one();
    } else {
        GTEST_SKIP();
    }
}

TEST(AffineElement, HashToCurve)
{
    std::vector<std::tuple<std::vector<uint8_t>, grumpkin::g1::affine_element>> test_vectors;
    test_vectors.emplace_back(std::vector<uint8_t>(),
                              grumpkin::g1::affine_element(
                                  fr(uint256_t("24c4cb9c1206ab5470592f237f1698abe684dadf0ab4d7a132c32b2134e2c12e")),
                                  fr(uint256_t("0668b8d61a317fb34ccad55c930b3554f1828a0e5530479ecab4defe6bbc0b2e"))));

    test_vectors.emplace_back(std::vector<uint8_t>{ 1 },
                              grumpkin::g1::affine_element(
                                  fr(uint256_t("107f1b633c6113f3222f39f6256f0546b41a4880918c86864b06471afb410454")),
                                  fr(uint256_t("050cd3823d0c01590b6a50adcc85d2ee4098668fd28805578aa05a423ea938c6"))));

    // "hello world"
    test_vectors.emplace_back(std::vector<uint8_t>{ 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64 },
                              grumpkin::g1::affine_element(
                                  fr(uint256_t("037c5c229ae495f6e8d1b4bf7723fafb2b198b51e27602feb8a4d1053d685093")),
                                  fr(uint256_t("10cf9596c5b2515692d930efa2cf3817607e4796856a79f6af40c949b066969f"))));

    for (std::tuple<std::vector<uint8_t>, grumpkin::g1::affine_element> test_case : test_vectors) {
        auto result = grumpkin::g1::affine_element::hash_to_curve(std::get<0>(test_case), 0);
        auto expected_result = std::get<1>(test_case);
        std::cout << result << std::endl;
        EXPECT_TRUE(result == expected_result);
    }
}
