#include <cstddef>
#include <gtest/gtest.h>

#include "barretenberg/polynomials/polynomial.hpp"

using namespace bb;

// Simple test/demonstration of shifted functionality
TEST(Polynomial, Shifted)
{
    using FF = bb::fr;
    using Polynomial = Polynomial<FF>;
    const size_t SIZE = 10;
    auto poly = Polynomial::random(SIZE);
    poly[0] = 0; // make it shiftable

    // Instantiate the shift via the shited method
    auto poly_shifted = poly.shifted();

    EXPECT_EQ(poly_shifted.size(), poly.size());

    // The shift is indeed the shift
    for (size_t i = 0; i < poly_shifted.size(); ++i) {
        EXPECT_EQ(poly_shifted.at(i), poly.at(i + 1));
    }

    // If I change the original polynomial, the shift is updated accordingly
    poly[3] = 25;
    for (size_t i = 0; i < poly_shifted.size(); ++i) {
        EXPECT_EQ(poly_shifted.at(i), poly.at(i + 1));
    }
}

// Simple test/demonstration of share functionality
TEST(Polynomial, Share)
{
    using FF = bb::fr;
    using Polynomial = Polynomial<FF>;
    const size_t SIZE = 10;
    auto poly = Polynomial::random(SIZE);

    // "clone" the poly via the share method
    auto poly_clone = poly.share();

    // The two are indeed equal
    EXPECT_EQ(poly_clone, poly);

    // Changing one changes the other
    poly[3] = 25;
    EXPECT_EQ(poly_clone, poly);

    poly_clone[2] = 13;
    EXPECT_EQ(poly_clone, poly);

    // If reset the original poly, it will no longer be equal to the clone made earlier
    // Note: if we had not made a clone, the memory from the original poly would be leaked
    auto poly2 = Polynomial::random(SIZE);
    poly = poly2.share();

    EXPECT_NE(poly_clone, poly);
}
