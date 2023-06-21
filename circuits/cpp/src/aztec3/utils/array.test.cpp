#include "array.hpp"

#include <gtest/gtest.h>

#include <array>
#include <cstddef>
#include <string>

namespace aztec3::utils {

using fr = NT::fr;

template <size_t N>
void rearrange_and_check(std::array<fr, N>& input, std::array<fr, N> const& expected, std::string name)
{
    array_rearrange(input);

    for (size_t i = 0; i < N; i++) {
        ASSERT_EQ(input[i], expected[i]) << "Mismatch for test vector " << name << " at position " << i;
    }
};

TEST(utils_array_tests, rearrange_test_vector1)
{
    std::array<fr, 5> test_vec{ fr(2), fr(4), fr(0), fr(12), fr(0) };
    std::array<fr, 5> const test_vec_rearranged{ fr(2), fr(4), fr(12), fr(0), fr(0) };

    rearrange_and_check(test_vec, test_vec_rearranged, "1");
}

TEST(utils_array_tests, rearrange_test_vector2)
{
    std::array<fr, 6> test_vec{ fr(0), fr(99), fr(0), fr(103), fr(0), fr(17) };
    std::array<fr, 6> const test_vec_rearranged{ fr(99), fr(103), fr(17), fr(0), fr(0), fr(0) };

    rearrange_and_check(test_vec, test_vec_rearranged, "2");
}

TEST(utils_array_tests, rearrange_test_vector3)
{
    std::array<fr, 4> test_vec{ fr(0), fr(0), fr(12), fr(0) };
    std::array<fr, 4> const test_vec_rearranged{ fr(12), fr(0), fr(0), fr(0) };

    rearrange_and_check(test_vec, test_vec_rearranged, "3");
}

TEST(utils_array_tests, rearrange_test_vector_identical)
{
    std::array<fr, 5> test_vec{ fr(2), fr(4), fr(7), fr(12), fr(9) };
    std::array<fr, 5> const test_vec_rearranged{ fr(2), fr(4), fr(7), fr(12), fr(9) };

    rearrange_and_check(test_vec, test_vec_rearranged, "identical");
}

TEST(utils_array_tests, rearrange_test_vector_empty)
{
    std::array<fr, 0> test_vec{};
    std::array<fr, 0> const test_vec_rearranged{};

    rearrange_and_check(test_vec, test_vec_rearranged, "empty");
}

TEST(utils_array_tests, rearrange_test_vector_all_zeros)
{
    std::array<fr, 7> test_vec{ fr(0), fr(0), fr(0), fr(0), fr(0), fr(0), fr(0) };
    std::array<fr, 7> const test_vec_rearranged{ fr(0), fr(0), fr(0), fr(0), fr(0), fr(0), fr(0) };

    rearrange_and_check(test_vec, test_vec_rearranged, "all zeros");
}

TEST(utils_array_tests, rearrange_test_vector_long_alternated)
{
    const size_t SIZE = 10000;
    std::array<fr, SIZE> test_vec{};
    std::array<fr, SIZE> test_vec_rearranged{};

    for (size_t i = 0; i < SIZE; i++) {
        test_vec[i] = (i % 2 == 0) ? fr(0) : fr(i);
    }

    for (size_t i = 0; i < SIZE / 2; i++) {
        test_vec_rearranged[i] = fr(2 * i + 1);
    }

    for (size_t i = SIZE / 2; i < SIZE; i++) {
        test_vec_rearranged[i] = fr(0);
    }

    rearrange_and_check(test_vec, test_vec_rearranged, "long alternated");
}

TEST(utils_array_tests, rearrange_test_vector_long_zeros_right)
{
    const size_t SIZE = 10000;
    std::array<fr, SIZE> test_vec{};
    std::array<fr, SIZE> test_vec_rearranged{};

    for (size_t i = 0; i < SIZE / 2; i++) {
        test_vec[i] = fr(i + 1);
        test_vec_rearranged[i] = fr(i + 1);
    }

    for (size_t i = SIZE / 2; i < SIZE; i++) {
        test_vec[i] = fr(0);
        test_vec_rearranged[i] = fr(0);
    }

    rearrange_and_check(test_vec, test_vec_rearranged, "long zeros right");
}

TEST(utils_array_tests, rearrange_test_vector_long_zeros_left)
{
    const size_t SIZE = 10000;
    std::array<fr, SIZE> test_vec{};
    std::array<fr, SIZE> test_vec_rearranged{};

    for (size_t i = 0; i < SIZE / 2; i++) {
        test_vec[i] = fr(0);
        test_vec_rearranged[i] = fr(i + 1);
    }

    for (size_t i = SIZE / 2; i < SIZE; i++) {
        test_vec[i] = fr(i - SIZE / 2 + 1);
        test_vec_rearranged[i] = fr(0);
    }

    rearrange_and_check(test_vec, test_vec_rearranged, "long zeros left");
}

}  // namespace aztec3::utils