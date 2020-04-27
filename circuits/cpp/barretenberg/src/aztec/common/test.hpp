#pragma once
#include <gtest/gtest.h>

#ifdef DISABLE_HEAVY_TESTS
#define HEAVY_TEST(x, y) TEST(x, DISABLED_##y)
#define HEAVY_TEST_F(x, y) TEST_F(x, DISABLED_##y)
#else
#define HEAVY_TEST(x, y) TEST(x, y)
#define HEAVY_TEST_F(x, y) TEST_F(x, y)
#endif
