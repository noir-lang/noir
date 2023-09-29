#include "array.hpp"

#include "aztec3/utils/dummy_circuit_builder.hpp"

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

TEST(utils_array_validation, test_vector_all_zeros)
{
    const size_t SIZE = 64;
    std::array<barretenberg::fr, SIZE> test_vec{};
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with all zeros");

    EXPECT_FALSE(dummyBuilder.failed()) << dummyBuilder.get_first_failure();
}

TEST(utils_array_validation, test_vector_all_non_zeros)
{
    const size_t SIZE = 64;
    std::array<barretenberg::fr, SIZE> test_vec;
    unsigned int gen = 4127;
    for (size_t i = 0; i < SIZE; i++) {
        test_vec[i] = fr(gen);
        gen = 761 * gen % 5619;
    }

    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with all non zeros");

    EXPECT_FALSE(dummyBuilder.failed()) << dummyBuilder.get_first_failure();
}

TEST(utils_array_validation, test_vector_valid_one_zero)
{
    const size_t SIZE = 110;
    std::array<barretenberg::fr, SIZE> test_vec{};
    unsigned int gen = 4159;
    for (size_t i = 0; i < SIZE - 1; i++) {
        test_vec[i] = fr(gen);
        gen = 71 * gen % 2613;
    }

    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with a single zero at the end");

    EXPECT_FALSE(dummyBuilder.failed()) << dummyBuilder.get_first_failure();
}

TEST(utils_array_validation, test_vector_valid_one_non_zero)
{
    const size_t SIZE = 110;
    std::array<barretenberg::fr, SIZE> test_vec{};
    test_vec[0] = fr(124);
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with a single non-zero at the beginning");

    EXPECT_FALSE(dummyBuilder.failed()) << dummyBuilder.get_first_failure();
}

TEST(utils_array_validation, test_vector_invalid_one_zero_middle)
{
    const size_t SIZE = 128;
    std::array<barretenberg::fr, SIZE> test_vec{};
    unsigned int gen = 354;
    for (size_t i = 0; i < SIZE; i++) {
        test_vec[i] = fr(gen);
        gen = 319 * gen % 2213;
    }
    test_vec[67] = fr(0);
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with a single zero in the middle");

    EXPECT_TRUE(dummyBuilder.failed());
}

TEST(utils_array_validation, test_vector_invalid_one_zero_beginning)
{
    const size_t SIZE = 128;
    std::array<barretenberg::fr, SIZE> test_vec{};
    unsigned int gen = 447;
    for (size_t i = 0; i < SIZE; i++) {
        test_vec[i] = fr(gen);
        gen = 39 * gen % 12313;
    }
    test_vec[0] = fr(0);
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with a single zero at the beginning");

    EXPECT_TRUE(dummyBuilder.failed());
}

TEST(utils_array_validation, test_vector_invalid_zero_both_ends)
{
    const size_t SIZE = 128;
    std::array<barretenberg::fr, SIZE> test_vec{};
    unsigned int gen = 47;
    for (size_t i = 0; i < SIZE; i++) {
        test_vec[i] = fr(gen);
        gen = 6439 * gen % 82313;
    }
    test_vec[0] = fr(0);
    test_vec[SIZE - 1] = fr(0);
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with a zero at each end");

    EXPECT_TRUE(dummyBuilder.failed());
}

TEST(utils_array_validation, test_vector_invalid_non_zero_last)
{
    const size_t SIZE = 203;
    std::array<barretenberg::fr, SIZE> test_vec{};
    test_vec[SIZE - 1] = fr(785);
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with a non-zero at the end");

    EXPECT_TRUE(dummyBuilder.failed());
}

TEST(utils_array_validation, test_vector_invalid_alternate)
{
    const size_t SIZE = 203;
    std::array<barretenberg::fr, SIZE> test_vec{};
    unsigned int gen = 83;
    for (size_t i = 0; i < SIZE; i += 2) {
        test_vec[i] = fr(gen);
        gen = 2437 * gen % 2314;
    }
    DummyCircuitBuilder dummyBuilder("Builder for array validation test vectors");
    validate_array(dummyBuilder, test_vec, "Test vector with alternating zero and non-zero values.");

    EXPECT_TRUE(dummyBuilder.failed());
}

}  // namespace aztec3::utils