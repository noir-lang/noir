#include <common/test.hpp>
#include <common/log.hpp>
#include <gtest/gtest.h>

#include "polynomial_store.hpp"

namespace waffle {

using namespace barretenberg;

TEST(polynomial_store, get_unknown)
{
    PolynomialStoreMem store;
    auto poly = store.get("unknown");
    EXPECT_EQ(poly.size(), 0);
    EXPECT_EQ(store.get_stats()["unknown"].first, 0);
    EXPECT_EQ(store.get_stats()["unknown"].second, 1);
}

TEST(polynomial_store, put_copies_polynomial)
{
    PolynomialStoreMem store;
    auto poly = polynomial(1024);
    store.put("id", poly);
    EXPECT_EQ(poly.size(), 1024);
    EXPECT_EQ(store.get_stats()["id"].first, 1);
    EXPECT_EQ(store.get_stats()["id"].second, 0);
}

TEST(polynomial_store, get_polynomial)
{
    PolynomialStoreMem store;
    auto poly = polynomial(1024);
    store.put("id", poly);
    auto got = store.get("id");
    EXPECT_EQ(got.size(), 1024);
    EXPECT_EQ(store.get_stats()["id"].first, 1);
    EXPECT_EQ(store.get_stats()["id"].second, 1);
}

} // namespace waffle