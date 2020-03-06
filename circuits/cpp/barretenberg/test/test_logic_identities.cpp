#include <gtest/gtest.h>

#include <cstdint>
#include <inttypes.h>
#include <random>

#include "./test_helpers.hpp"

bool quaternary_ror(const std::vector<uint64_t>& accumulators, size_t ror, uint64_t target)
{
    size_t n = accumulators.size();

    if (ror == 0 || ror == 32) {
        return true;
    }

    if ((ror & 1) == 0) {
        size_t x = ror >> 1;
        size_t pivot = (n - 1 - x);

        uint64_t left = accumulators[pivot];
        uint64_t right = accumulators[n - 1];
        uint64_t t0 = (1ULL << (x * 2));
        uint64_t t1 = (1ULL << (n - x) * 2);
        uint64_t t2 = t0 * t1;

        uint64_t left_shift_factor = 1 - t2;
        uint64_t right_shift_factor = t1;

        uint64_t out = left * left_shift_factor + right * right_shift_factor;
        return (out == target);
    }

    size_t x = (ror >> 1) + 1;
    size_t pivot = (n - 1 - x);

    uint64_t a_pivot = ror == 31 ? 0 : accumulators[pivot];

    uint64_t a_n = accumulators[n - 1];

    uint64_t delta_quad = ror == 31 ? accumulators[0] : accumulators[pivot + 1] - 4 * accumulators[pivot];
    uint64_t b_lo = delta_quad & 1ULL;
    uint64_t b_hi = (delta_quad & 2ULL) >> 1ULL;

    uint64_t t0 = a_pivot * (2 - (1ULL << (2 * n + 1ULL)));
    uint64_t t1 = a_n * (1ULL << ((n - x) * 2ULL + 1ULL));
    uint64_t t2 = b_hi * (1ULL - (1ULL << (2 * n)));

    uint64_t out = t0 + t1 + t2;

    bool delta_valid = b_lo + b_hi + b_hi == delta_quad;

    return (out == target) && delta_valid;
}

bool quaternary_left_shift(const std::vector<uint64_t>& accumulators, size_t shift, uint64_t target)
{
    size_t n = accumulators.size();

    if (shift == 0) {
        return true;
    }

    if ((shift & 1) == 0) {
        size_t x = (shift >> 1);
        uint64_t right = accumulators[x - 1];
        uint64_t base = accumulators[n - 1];

        uint64_t base_shift_factor = 1ULL << ((x)*2);
        uint64_t right_shift_factor = 1ULL << ((n)*2);
        uint64_t diff = base * base_shift_factor - right * right_shift_factor;

        return diff == target;
    }

    size_t x = (shift >> 1);
    uint64_t right = (shift == 1) ? 0 : accumulators[x - 1];
    uint64_t left = accumulators[x];
    uint64_t base = accumulators[n - 1];

    uint64_t base_shift_factor = 1UL << ((x)*2);
    uint64_t right_shift_factor = 1ULL << ((n)*2);

    uint64_t delta_quad = left - (4 * right);
    uint64_t b_lo = delta_quad & 1ULL;
    uint64_t b_hi = (delta_quad & 2ULL) >> 1ULL;

    uint64_t out = (2 * base * base_shift_factor - (2 * right + b_hi) * right_shift_factor);

    bool delta_valid = b_lo + b_hi + b_hi == delta_quad;

    return (out == target) && delta_valid;
}

bool quaternary_rol(const std::vector<uint64_t>& accumulators, size_t rol, uint64_t target)
{
    return quaternary_ror(accumulators, 32 - rol, target);
}

bool quaternary_right_shift(const std::vector<uint64_t>& accumulators, size_t shift, uint64_t target)
{
    size_t n = accumulators.size();

    if ((shift & 1) == 0) {
        return accumulators[(n - 1 - (shift >> 1))] == target;
    }

    size_t x = (n - 1 - (shift >> 1));
    uint64_t right = accumulators[x];
    uint64_t left = shift == 31 ? 0 : accumulators[x - 1];

    uint64_t delta_quad = right - (4 * left);
    uint64_t b_lo = delta_quad & 1ULL;
    uint64_t b_hi = (delta_quad & 2ULL) >> 1ULL;

    uint64_t quaternary_contribution = left * 2;
    uint64_t binary_contribution = b_hi;

    bool delta_valid = b_lo + b_hi + b_hi == delta_quad;

    uint64_t out = quaternary_contribution + binary_contribution;

    return (out == target) && delta_valid;
}

bool and_xor_identity(uint64_t a_x,
                      uint64_t a_x_omega,
                      uint64_t b_x,
                      uint64_t b_x_omega,
                      uint64_t w,
                      uint64_t c_x,
                      uint64_t c_x_omega,
                      uint64_t selector,
                      size_t i,
                      size_t j)
{
    uint64_t delta_a = a_x_omega - (4 * a_x);
    uint64_t delta_b = b_x_omega - (4 * b_x);
    uint64_t delta_c = c_x_omega - (4 * c_x);

    uint64_t delta_a_squared = (delta_a * delta_a);
    uint64_t delta_b_squared = (delta_b * delta_b);
    uint64_t delta_c_squared = (delta_c * delta_c);

    uint64_t five_delta_a = delta_a + delta_a;
    uint64_t three_delta_a = five_delta_a + delta_a;
    five_delta_a = five_delta_a + three_delta_a;

    uint64_t five_delta_b = delta_b + delta_b;
    uint64_t three_delta_b = five_delta_b + delta_b;
    five_delta_b = five_delta_b + three_delta_b;

    uint64_t five_delta_c = delta_c + delta_c;
    uint64_t three_delta_c = five_delta_c + delta_c;
    five_delta_c = five_delta_c + three_delta_c;

    uint64_t delta_a_test = (delta_a_squared - delta_a) * (delta_a_squared - five_delta_a + 6);
    uint64_t delta_b_test = (delta_b_squared - delta_b) * (delta_b_squared - five_delta_b + 6);
    uint64_t delta_c_test = (delta_c_squared - delta_c) * (delta_c_squared - five_delta_c + 6);

    uint64_t w_test = (delta_a * delta_b - w);

    uint64_t delta_sum_three = three_delta_a + three_delta_b;
    uint64_t delta_sum_nine = delta_sum_three + delta_sum_three;
    delta_sum_nine = delta_sum_nine + delta_sum_three;
    uint64_t delta_sum_eighteen = delta_sum_nine + delta_sum_nine;
    uint64_t delta_sum_eighty_one = delta_sum_eighteen + delta_sum_eighteen;
    delta_sum_eighty_one = delta_sum_eighty_one + delta_sum_eighty_one;
    delta_sum_eighty_one = delta_sum_eighty_one + delta_sum_nine;

    uint64_t delta_squared_sum = delta_a_squared + delta_b_squared;
    uint64_t delta_squared_sum_eighteen = delta_squared_sum + delta_squared_sum;          // 2
    uint64_t delta_squared_sum_three = delta_squared_sum_eighteen + delta_squared_sum;    // 3
    delta_squared_sum_eighteen = delta_squared_sum_three + delta_squared_sum_three;       // 6
    delta_squared_sum_eighteen = delta_squared_sum_eighteen + delta_squared_sum_three;    // 9
    delta_squared_sum_eighteen = delta_squared_sum_eighteen + delta_squared_sum_eighteen; // 18

    uint64_t r1 = w + w;   // 2
    uint64_t r2 = r1 + r1; // 4

    uint64_t poly_and_a = r2 - delta_sum_eighteen + 81;
    uint64_t poly_and_b = -delta_sum_eighty_one + delta_squared_sum_eighteen + 83;
    uint64_t poly_and_c = w * (w * poly_and_a + poly_and_b);

    uint64_t delta_c_2 = delta_c + delta_c;
    uint64_t delta_c_3 = delta_c_2 + delta_c;
    uint64_t delta_c_6 = delta_c_3 + delta_c_3;
    uint64_t delta_c_9 = delta_c_6 + delta_c_3;
    uint64_t identity = selector * (selector * (delta_c_9 - delta_sum_three) +
                                    (delta_c_3 - (poly_and_c + poly_and_c) + delta_sum_three));

    return !(delta_a_test || delta_b_test || delta_c_test || w_test || identity);
}
bool xor_identity(uint64_t a_x,
                  uint64_t a_x_omega,
                  uint64_t b_x,
                  uint64_t b_x_omega,
                  uint64_t w,
                  uint64_t c_x,
                  uint64_t c_x_omega,
                  size_t i,
                  size_t j)
{
    uint64_t delta_a = a_x_omega - (4 * a_x);
    uint64_t delta_b = b_x_omega - (4 * b_x);
    uint64_t delta_c = c_x_omega - (4 * c_x);

    uint64_t delta_a_test = (delta_a) * (delta_a - 1) * (delta_a - 2) * (delta_a - 3);
    uint64_t delta_b_test = (delta_b) * (delta_b - 1) * (delta_b - 2) * (delta_b - 3);
    uint64_t delta_c_test = (delta_c) * (delta_c - 1) * (delta_c - 2) * (delta_c - 3);

    uint64_t w_test = (delta_a * delta_b - w);

    uint64_t poly_a = 18 * (delta_a + delta_b) - 4 * w - 81;
    uint64_t poly_b = 81 * (delta_a + delta_b) - 18 * (delta_a * delta_a + delta_b * delta_b) - 83;
    uint64_t poly_c = w * (w * poly_a + poly_b) + 3 * (delta_a + delta_b);

    uint64_t poly_test = (delta_c * 3) - poly_c;
    if (poly_test != 0) {
        printf("invalid at index %zu : %zu  value = %" PRIu64 " \n", i, j, poly_c);
    }
    return !(delta_a_test || delta_b_test || delta_c_test || w_test || poly_test);
}

bool and_identity(uint64_t a_x,
                  uint64_t a_x_omega,
                  uint64_t b_x,
                  uint64_t b_x_omega,
                  uint64_t w,
                  uint64_t c_x,
                  uint64_t c_x_omega,
                  size_t i,
                  size_t j)
{
    uint64_t delta_a = a_x_omega - (4 * a_x);
    uint64_t delta_b = b_x_omega - (4 * b_x);
    uint64_t delta_c = c_x_omega - (4 * c_x);

    uint64_t delta_a_test = (delta_a) * (delta_a - 1) * (delta_a - 2) * (delta_a - 3);
    uint64_t delta_b_test = (delta_b) * (delta_b - 1) * (delta_b - 2) * (delta_b - 3);
    uint64_t delta_c_test = (delta_c) * (delta_c - 1) * (delta_c - 2) * (delta_c - 3);

    uint64_t w_test = (delta_a * delta_b - w);

    uint64_t poly_a = 4 * w - 18 * (delta_a + delta_b) + 81;
    uint64_t poly_b = 18 * (delta_a * delta_a + delta_b * delta_b) - 81 * (delta_a + delta_b) + 83;
    uint64_t poly_c = w * (w * poly_a + poly_b);

    uint64_t poly_test = (delta_c * 6) - poly_c;
    if (poly_test != 0) {
        printf("invalid at index %zu : %zu , value = %" PRIu64 " \n", i, j, poly_c);
    }
    return !(delta_a_test || delta_b_test || delta_c_test || w_test || poly_test);
}

TEST(test_logic_identities, quaternary_rol)
{
    for (size_t i = 0; i < 100; ++i) {
        size_t input = test_helpers::get_pseudorandom_uint32();
        size_t rol = test_helpers::get_pseudorandom_uint32() & 31ULL;
        std::vector<uint64_t> accumulators;
        uint64_t accumulator = 0;
        for (uint64_t i = 0; i < 32; i += 2) {
            uint64_t t0 = (input >> (30 - i)) & 1ULL;
            uint64_t t1 = (input >> (30 - i)) & 2ULL;
            uint64_t slice = t0 + t1;
            accumulator = 4 * accumulator + slice;
            accumulators.emplace_back(accumulator);
        }

        size_t target = rol == 0 ? input : ((input << (rol)) & 0xffffffff) | (input >> (32 - rol));

        if (!quaternary_rol(accumulators, rol, target)) {
            printf("ror failed. target = %zu, ror = %zu \n", target, rol);
        }
        EXPECT_EQ(quaternary_rol(accumulators, rol, target), true);
    }
}

TEST(test_logic_identities, quaternary_ror)
{
    for (size_t i = 0; i < 100; ++i) {
        uint64_t input = test_helpers::get_pseudorandom_uint32();
        size_t ror = test_helpers::get_pseudorandom_uint32() & 31ULL;
        std::vector<uint64_t> accumulators;
        uint64_t accumulator = 0;
        for (uint64_t i = 0; i < 32; i += 2) {
            uint64_t t0 = (input >> (30 - i)) & 1ULL;
            uint64_t t1 = (input >> (30 - i)) & 2ULL;
            uint64_t slice = t0 + t1;
            accumulator = 4 * accumulator + slice;
            accumulators.emplace_back(accumulator);
        }

        size_t target = ror == 0 ? input : ((input << (32 - ror)) & 0xffffffff) | (input >> ror);

        if (!quaternary_ror(accumulators, ror, target)) {
            printf("ror failed. target = %zu, ror = %zu \n", target, ror);
        }
        EXPECT_EQ(quaternary_ror(accumulators, ror, target), true);
    }
}

TEST(test_logic_identities, quaternary_left_shift)
{
    for (size_t i = 0; i < 100; ++i) {
        size_t input = test_helpers::get_pseudorandom_uint32();
        size_t shift = test_helpers::get_pseudorandom_uint32() & 31ULL;
        std::vector<uint64_t> accumulators;
        uint64_t accumulator = 0;
        for (uint64_t i = 0; i < 32; i += 2) {
            uint64_t t0 = (input >> (30 - i)) & 1ULL;
            uint64_t t1 = (input >> (30 - i)) & 2ULL;
            uint64_t slice = t0 + t1;
            accumulator = 4 * accumulator + slice;
            accumulators.emplace_back(accumulator);
        }

        size_t target = (input << shift) & 0xffffffff;

        if (!quaternary_left_shift(accumulators, shift, target)) {
            printf("left shift failed. target = %zu, shift = %zu \n", target, shift);
        }
        EXPECT_EQ(quaternary_left_shift(accumulators, shift, target), true);
    }
}

TEST(test_logic_identities, quaternary_right_shift)
{
    for (size_t i = 0; i < 100; ++i) {
        size_t input = test_helpers::get_pseudorandom_uint32();
        size_t shift = test_helpers::get_pseudorandom_uint32() & 31ULL;
        std::vector<uint64_t> accumulators;
        uint64_t accumulator = 0;
        for (uint64_t i = 0; i < 32; i += 2) {
            uint64_t t0 = (input >> (30 - i)) & 1ULL;
            uint64_t t1 = (input >> (30 - i)) & 2ULL;
            uint64_t slice = t0 + t1;
            accumulator = 4 * accumulator + slice;
            accumulators.emplace_back(accumulator);
        }

        size_t target = input >> shift;

        if (!quaternary_right_shift(accumulators, shift, target)) {
            printf("right shift failed. target = %zu, shift = %zu \n", target, shift);
        }
        EXPECT_EQ(quaternary_right_shift(accumulators, shift, target), true);
    }
}

TEST(test_logic_identities, xor)
{
    uint64_t base_a = 128;
    uint64_t base_b = 513;
    uint64_t base_c = base_a ^ base_b;

    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 4; ++j) {
            uint64_t a_val = i;
            uint64_t b_val = j;
            uint64_t c_val = i ^ j;

            uint64_t a_x = base_a;
            uint64_t a_x_omega = (4 * base_a) + a_val;

            uint64_t b_x = base_b;
            uint64_t b_x_omega = (4 * base_b) + b_val;

            uint64_t c_x = base_c;
            uint64_t c_x_omega = (4 * base_c) + c_val;

            uint64_t w = a_val * b_val;

            bool valid = xor_identity(a_x, a_x_omega, b_x, b_x_omega, w, c_x, c_x_omega, i, j);

            EXPECT_EQ(valid, true);
        }
    }
}

TEST(test_logic_identities, and)
{
    uint64_t base_a = 128;
    uint64_t base_b = 513;
    uint64_t base_c = base_a & base_b;

    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 4; ++j) {
            uint64_t a_val = i;
            uint64_t b_val = j;
            uint64_t c_val = i & j;

            uint64_t a_x = base_a;
            uint64_t a_x_omega = (4 * base_a) + a_val;

            uint64_t b_x = base_b;
            uint64_t b_x_omega = (4 * base_b) + b_val;

            uint64_t c_x = base_c;
            uint64_t c_x_omega = (4 * base_c) + c_val;

            uint64_t w = a_val * b_val;

            bool valid = and_identity(a_x, a_x_omega, b_x, b_x_omega, w, c_x, c_x_omega, i, j);

            // EXPECT_EQ(valid, true);
        }
    }
}

TEST(test_logic_identities, and_xor)
{
    uint64_t base_a = 128;
    uint64_t base_b = 513;
    uint64_t base_c = base_a ^ base_b;

    for (size_t k = 0; k < 2; ++k) {
        for (size_t i = 0; i < 4; ++i) {
            for (size_t j = 0; j < 4; ++j) {
                uint64_t a_val = i;
                uint64_t b_val = j;
                uint64_t c_val = k == 0 ? i & j : i ^ j;

                uint64_t a_x = base_a;
                uint64_t a_x_omega = (4 * base_a) + a_val;

                uint64_t b_x = base_b;
                uint64_t b_x_omega = (4 * base_b) + b_val;

                uint64_t c_x = base_c;
                uint64_t c_x_omega = (4 * base_c) + c_val;

                uint64_t w = a_val * b_val;

                uint64_t selector = (k == 0) ? 1 : (uint64_t(-1));
                bool valid = and_xor_identity(a_x, a_x_omega, b_x, b_x_omega, w, c_x, c_x_omega, selector, i, j);

                EXPECT_EQ(valid, true);
            }
        }
    }
}