#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/curves/secp256k1/secp256k1.hpp"
#include "barretenberg/ecc/curves/secp256r1/secp256r1.hpp"
#include "barretenberg/serialize/test_helper.hpp"
#include <fstream>

namespace TestAffineElement {
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
};

using TestTypes = testing::Types<bb::g1, grumpkin::g1, secp256k1::g1, secp256r1::g1>;

TYPED_TEST_SUITE(TestAffineElement, TestTypes);

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

// Regression test to ensure that the point at infinity is not equal to its coordinate-wise reduction, which may lie
// on the curve, depending on the y-coordinate.
TEST(AffineElement, InfinityOrderingRegression)
{
    secp256k1::g1::affine_element P(0, 1);
    secp256k1::g1::affine_element Q(0, 1);

    P.self_set_infinity();
    EXPECT_NE(P < Q, Q < P);
}

TEST(AffineElement, Msgpack)
{
    auto [actual, expected] = msgpack_roundtrip(secp256k1::g1::affine_element{ 1, 1 });
    EXPECT_EQ(actual, expected);
}
} // namespace TestAffineElement
