#include <common/test.hpp>
#include <common/log.hpp>
#include <gtest/gtest.h>

#include "polynomial_cache.hpp"

namespace waffle {

using namespace barretenberg;

size_t CAPACITY = 1024 * 64;

#if !defined(__wasm__)
TEST(polynomial_cache, get_unknown)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    EXPECT_THROW(cache.get("unknown"), std::runtime_error);
}
#endif

TEST(polynomial_cache, get_unknown_with_size)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    auto poly = cache.get("unknown", 1024);
    EXPECT_EQ(poly.size(), 1024);
}

TEST(polynomial_cache, creates_polynomial)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    polynomial poly(1024);
    EXPECT_EQ(poly.size(), 1024);
}

TEST(polynomial_cache, put_moves_polynomial)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    polynomial poly(1024);
    cache.put("id", std::move(poly));
    EXPECT_EQ(poly.size(), 0);
}

TEST(polynomial_cache, put_polynomial_overwrite)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    {
        polynomial poly(1024);
        cache.put("id", std::move(poly));
        auto got = cache.get("id");
        EXPECT_EQ(got.size(), 1024);
        EXPECT_EQ(cache.get_volume(), 1024 * 32);
    }
    {
        polynomial poly(2048);
        cache.put("id", std::move(poly));
        auto got = cache.get("id");
        EXPECT_EQ(got.size(), 2048);
        EXPECT_EQ(cache.get_volume(), 2048 * 32);
    }
}

TEST(polynomial_cache, put_polynomial_self_overwrite)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    polynomial poly(1024);
    cache.put("id", std::move(poly));
    cache.put("id", std::move(cache.get("id")));
    auto got = cache.get("id");
    EXPECT_EQ(got.size(), 1024);
    EXPECT_EQ(cache.get_volume(), 1024 * 32);
}

TEST(polynomial_cache, get_polynomial)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    polynomial poly(1024);
    cache.put("id", std::move(poly));
    auto got = cache.get("id");
    EXPECT_EQ(got.size(), 1024);
}

TEST(polynomial_cache, get_polynomial_from_store)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    polynomial poly(1024);
    mem_store.put("id", poly);
    auto got = cache.get("id");
    EXPECT_EQ(got.size(), 1024);
}

TEST(polynomial_cache, overflow_flush_and_get)
{
    PolynomialStoreMem mem_store;
    PolynomialCache cache(&mem_store, CAPACITY);
    cache.put("id1", polynomial(512));
    cache.put("id2", polynomial(512));
    cache.put("id3", polynomial(1024));
    // First 2 polys will be flushed.
    cache.put("id4", polynomial(1024));

    EXPECT_EQ(cache.get_lru(), (std::vector<std::string>{ "id4", "id3" }));

    EXPECT_EQ(cache.get("id1").size(), 512);
    EXPECT_EQ(cache.get_lru(), (std::vector<std::string>{ "id1", "id4" }));

    EXPECT_EQ(cache.get("id2").size(), 512);
    EXPECT_EQ(cache.get_lru(), (std::vector<std::string>{ "id2", "id1", "id4" }));

    EXPECT_EQ(cache.get("id3").size(), 1024);
    EXPECT_EQ(cache.get_lru(), (std::vector<std::string>{ "id3", "id2", "id1" }));

    EXPECT_EQ(cache.get("id4").size(), 1024);
    EXPECT_EQ(cache.get_lru(), (std::vector<std::string>{ "id4", "id3" }));
}

TEST(polynomial_cache, no_store)
{
    PolynomialCache cache;
    polynomial poly(1024);
    cache.put("id", std::move(poly));
    auto got = cache.get("id");
    EXPECT_EQ(got.size(), 1024);
}

} // namespace waffle