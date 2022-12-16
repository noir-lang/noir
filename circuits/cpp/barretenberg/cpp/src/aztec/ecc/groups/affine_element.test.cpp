#include <ecc/curves/bn254/g1.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <ecc/curves/secp256k1/secp256k1.hpp>
#include <ecc/curves/secp256r1/secp256r1.hpp>
#include <common/test.hpp>
#include <fstream>
#include <common/serialize.hpp>

namespace test_affine_element {
template <typename G1> class test_affine_element : public testing::Test {
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

    // Regression test to ensure that the point at infinity is not equal to its coordinate-wise reduction, which may lie
    // on the curve, depending on the y-coordinate.
    // TODO: add corresponding typed test class
    static void test_infinity_regression()
    {
        affine_element P;
        P.self_set_infinity();
        affine_element R(0, P.y);
        ASSERT_FALSE(P == R);
    }
};

typedef testing::Types<barretenberg::g1, grumpkin::g1, secp256k1::g1, secp256r1::g1> TestTypes;

TYPED_TEST_SUITE(test_affine_element, TestTypes);

TYPED_TEST(test_affine_element, read_write_buffer)
{
    TestFixture::test_read_write_buffer();
}

TYPED_TEST(test_affine_element, point_compression)
{
    if constexpr (TypeParam::Fq::modulus.data[3] >= 0x4000000000000000ULL) {
        GTEST_SKIP();
    } else {
        TestFixture::test_point_compression();
    }
}

// Regression test to ensure that the point at infinity is not equal to its coordinate-wise reduction, which may lie
// on the curve, depending on the y-coordinate.
TEST(affine_element, infinity_ordering_regression)
{
    secp256k1::g1::affine_element P(0, 1), Q(0, 1);

    P.self_set_infinity();
    EXPECT_NE(P < Q, Q < P);
}
} // namespace test_affine_element