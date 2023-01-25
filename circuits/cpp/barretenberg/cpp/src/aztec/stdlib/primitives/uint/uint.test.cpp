#include "uint.hpp"
#include "honk/composer/standard_honk_composer.hpp"
#include <functional>
#include <gtest/gtest.h>
#include <numeric/random/engine.hpp>

using namespace barretenberg;
using namespace plonk;
using namespace bonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

// NOTE: We only test width 32, but widths 8, 16, 32 and 64 can all be tested.
//       In widths 8, 16, 32: all tests pass.
//       In width 64, the following tests fail for UltraComposer.
//           test_xor_special, test_xor_more_constants, test_and_constants, test_and_special, test_or_special,
//           test_ror_special, test_hash_rounds, test_and, test_xor, test_or.
// They fail with 'C++ exception with description"Last key slice greater than 64" thrown in the test body."'
namespace test_stdlib_uint {
typedef uint32_t uint_native;
size_t uint_native_width = 8 * sizeof(uint_native);
uint_native uint_native_max = static_cast<uint_native>((static_cast<uint256_t>(1) << uint_native_width) - 1);

template <typename Native> Native get_random()
{
    return static_cast<uint_native>(engine.get_random_uint64());
};

template <typename Native> std::vector<Native> get_several_random(size_t num)
{
    std::vector<Native> result;
    for (size_t i = 0; i < num; ++i) {
        result.emplace_back(get_random<Native>());
    }
    return result;
}

/**
 * @brief Utility function for testing the uint_ct comparison operators
 *
 * @details Given a uint_ct a and a constant const_b, this  allows to create a
 * uint_ct b having a desired relation to a (either >. = or <).
 */
uint_native impose_comparison(uint_native const_a,
                              uint_native const_b,
                              uint_native a_val,
                              bool force_equal = false,
                              bool force_gt = false,
                              bool force_lt = false)
{
    uint_native b_val;
    if (force_equal) {
        b_val = a_val + const_a - const_b;
    } else if (force_lt) { // forcing b < a
        // if   a_val + const_a != const_b, then we set up b_val + const_b = a_val + const_a - 1
        // elif a_val + const_a  = const_b, then we set up b_val + const_b = a_val + const_a
        //   and we increment a by 1, leading to           a_val + const_a = b_val + const_b + 1.
        b_val = (a_val + const_a - const_b) ? a_val + const_a - const_b - 1 : const_a - const_b + (a_val++);
    } else if (force_gt) { // forcing b > a
        // set b_val + const_b = a_val + const_a + 1 unless that would wrap, in which case we instead
        // set b_val + const_b = a then decrease a by 1.
        b_val = (a_val + const_a - const_b) == uint_native_width ? const_a - const_b + (a_val--)
                                                                 : a_val + const_a - const_b + 1;
    } else {
        b_val = get_random<uint_native>();
    }
    return b_val;
}

uint_native rotate(uint_native value, size_t rotation)
{
    return rotation ? static_cast<uint_native>(value >> rotation) +
                          static_cast<uint_native>(value << (uint_native_width - rotation))
                    : value;
}
template <typename Composer> class stdlib_uint : public testing::Test {
    typedef typename std::conditional<Composer::type == waffle::ComposerType::PLOOKUP,
                                      stdlib::uint_plookup<Composer, uint_native>,
                                      stdlib::uint<Composer, uint_native>>::type uint_ct;
    typedef stdlib::bool_t<Composer> bool_ct;
    typedef stdlib::witness_t<Composer> witness_ct;
    typedef stdlib::byte_array<Composer> byte_array_ct;

    static inline std::vector<uint_native> special_values{ 0U,
                                                           1U,
                                                           2U,
                                                           static_cast<uint_native>(1 << uint_native_width / 4),
                                                           static_cast<uint_native>(1 << uint_native_width / 2),
                                                           static_cast<uint_native>((1 << uint_native_width / 2) + 1),
                                                           uint_native_max };

    static std::vector<uint_ct> get_special_uints(Composer* ctx)
    {
        std::vector<uint_ct> special_uints;
        for (size_t i = 0; i != special_values.size(); ++i) {
            special_uints.emplace_back(witness_ct(ctx, special_values[i]));
        };
        return special_uints;
    };

  public:
    static void test_weak_normalize()
    {
        auto run_test = [](bool constant_only, bool add_constant) {
            Composer composer = Composer();
            uint_ct a;
            uint_native a_val = get_random<uint_native>();
            uint_native const_a = get_random<uint_native>();
            uint_native expected;

            if (constant_only) {
                a = const_a;
                expected = const_a;
            } else {
                a = witness_ct(&composer, a_val);
                expected = a_val;
                if (add_constant) {
                    a += const_a;
                    expected += const_a;
                }
            };

            EXPECT_EQ(expected, a.get_value());
            auto prover = composer.create_prover();
            auto verifier = composer.create_verifier();
            waffle::plonk_proof proof = prover.construct_proof();
            bool verified = verifier.verify_proof(proof);
            EXPECT_EQ(verified, true);
        };

        run_test(true, false);
        run_test(false, false);
        run_test(false, true);
    }

    static void test_byte_array_conversion()
    {
        Composer composer = Composer();
        uint_ct a = witness_ct(&composer, 0x7f6f5f4f10111213);
        std::string longest_expected = { 0x7f, 0x6f, 0x5f, 0x4f, 0x10, 0x11, 0x12, 0x13 };
        // truncate, so we are running different tests for different choices of uint_native
        std::string expected = longest_expected.substr(longest_expected.length() - sizeof(uint_native));
        byte_array_ct arr(&composer);
        arr.write(static_cast<byte_array_ct>(a));

        EXPECT_EQ(arr.size(), sizeof(uint_native));
        EXPECT_EQ(arr.get_string(), expected);
    }

    static void test_input_output_consistency()
    {
        Composer composer = Composer();

        for (size_t i = 1; i < 1024; i *= 2) {
            uint_native a_expected = (uint_native)i;
            uint_native b_expected = (uint_native)i;

            uint_ct a = witness_ct(&composer, a_expected);
            uint_ct b = witness_ct(&composer, b_expected);

            byte_array_ct arr(&composer);

            arr.write(static_cast<byte_array_ct>(a));
            arr.write(static_cast<byte_array_ct>(b));

            EXPECT_EQ(arr.size(), 2 * sizeof(uint_native));

            uint_ct a_result(arr.slice(0, sizeof(uint_native)));
            uint_ct b_result(arr.slice(sizeof(uint_native)));

            EXPECT_EQ(a_result.get_value(), a_expected);
            EXPECT_EQ(b_result.get_value(), b_expected);
        }
    }

    static void test_create_from_wires()
    {
        Composer composer = Composer();

        uint_ct a = uint_ct(&composer,
                            std::vector<bool_ct>{
                                bool_ct(false),
                                bool_ct(false),
                                bool_ct(false),
                                bool_ct(false),
                                bool_ct(false),
                                bool_ct(false),
                                bool_ct(false),
                                witness_ct(&composer, true),
                            });

        EXPECT_EQ(a.at(0).get_value(), false);
        EXPECT_EQ(a.at(7).get_value(), true);
        EXPECT_EQ(static_cast<uint32_t>(a.get_value()), 128U);
    }

    /**
     * @brief Test addition of special values.
     * */
    static void test_add_special()
    {
        Composer composer = Composer();

        witness_ct first_input(&composer, 1U);
        witness_ct second_input(&composer, 0U);

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a + b;
        /**
         * Fibbonacci sequence a(0) = 0, a(1), ..., a(2 + 32) = 5702887
         * a | 1 | 1 | 2 | 3 | 5 | ...
         * b | 0 | 1 | 1 | 2 | 3 | ...
         * c | 1 | 2 | 3 | 5 | 8 | ...
         */
        for (size_t i = 0; i < uint_native_width; ++i) {
            b = a;
            a = c;
            c = a + b;
        }

        auto special_uints = get_special_uints(&composer);
        for (size_t i = 0; i != special_values.size(); ++i) {
            uint_native x = special_values[i];
            uint_ct x_ct = special_uints[i];

            for (size_t j = i; j != special_values.size(); ++j) {
                uint_native y = special_values[j];
                uint_ct y_ct = special_uints[j];

                uint_native expected_value = x + y;
                uint_ct z_ct = x_ct + y_ct;
                uint_native value = static_cast<uint_native>(z_ct.get_value());

                EXPECT_EQ(value, expected_value);
            }
        };

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_sub_special()
    {
        Composer composer = Composer();

        witness_ct a_val(&composer, static_cast<uint_native>(4));
        // witness_ct b_val(&composer, static_cast<uint_native>(5));
        uint_native const_a = 1;
        uint_native const_b = 2;
        uint_ct a = uint_ct(a_val) + const_a;
        // uint_ct b = uint_ct(b_val) + const_b;
        uint_ct b = const_b;
        uint_ct diff = a - b;

        auto special_uints = get_special_uints(&composer);
        for (size_t i = 0; i != special_values.size(); ++i) {
            uint_native x = special_values[i];
            uint_ct x_ct = special_uints[i];

            for (size_t j = i; j != special_values.size(); ++j) {
                uint_native y = special_values[j];
                uint_ct y_ct = special_uints[j];

                uint_native expected_value = x - y;
                uint_ct z_ct = x_ct - y_ct;
                uint_native value = static_cast<uint_native>(z_ct.get_value());

                EXPECT_EQ(value, expected_value);
            }
        };

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);

        EXPECT_EQ(verified, true);
    }

    static void test_add_with_constants()
    {
        size_t n = 8;
        std::vector<uint_native> witnesses = get_several_random<uint_native>(3 * n);
        uint_native expected[8];
        for (size_t i = 2; i < n; ++i) {
            expected[0] = witnesses[3 * i];
            expected[1] = witnesses[3 * i + 1];
            expected[2] = witnesses[3 * i + 2];
            expected[3] = expected[0] + expected[1];
            expected[4] = expected[1] + expected[0];
            expected[5] = expected[1] + expected[2];
            expected[6] = expected[3] + expected[4];
            expected[7] = expected[4] + expected[5];
        }
        Composer composer = Composer();
        uint_ct result[8];
        for (size_t i = 2; i < n; ++i) {
            result[0] = uint_ct(&composer, witnesses[3 * i]);
            result[1] = (witness_ct(&composer, witnesses[3 * i + 1]));
            result[2] = (witness_ct(&composer, witnesses[3 * i + 2]));
            result[3] = result[0] + result[1];
            result[4] = result[1] + result[0];
            result[5] = result[1] + result[2];
            result[6] = result[3] + result[4];
            result[7] = result[4] + result[5];
        }

        for (size_t i = 0; i < n; ++i) {
            EXPECT_EQ(result[i].get_value(), expected[i]);
        }

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_valid = verifier.verify_proof(proof);
        EXPECT_EQ(proof_valid, true);
    }

    static void test_mul_special()
    {
        uint_native a_expected = 1U;
        uint_native b_expected = 2U;
        uint_native c_expected = a_expected + b_expected;
        for (size_t i = 0; i < 100; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = a_expected * b_expected;
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, 1U);
        witness_ct second_input(&composer, 2U);

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a + b;
        for (size_t i = 0; i < 100; ++i) {
            b = a;
            a = c;
            c = a * b;
        }
        uint_native c_result =
            static_cast<uint_native>(composer.get_variable(c.get_witness_index()).from_montgomery_form().data[0]);
        EXPECT_EQ(c_result, c_expected);

        auto special_uints = get_special_uints(&composer);
        for (size_t i = 0; i != special_values.size(); ++i) {
            uint_native x = special_values[i];
            uint_ct x_ct = special_uints[i];

            for (size_t j = i; j != special_values.size(); ++j) {
                uint_native y = special_values[j];
                uint_ct y_ct = special_uints[j];

                uint_native expected_value = x * y;
                uint_ct z_ct = x_ct * y_ct;
                uint_native value = static_cast<uint_native>(z_ct.get_value());

                EXPECT_EQ(value, expected_value);
            }
        };

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_mul_big()
    {
        uint_native max = uint_native_max;

        Composer composer = Composer();
        uint_ct a = witness_ct(&composer, max);
        a = a + max;
        uint_ct b = a;
        uint_ct c = a * b;

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_xor_special()
    {
        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected ^ b_expected;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = a_expected + b_expected;
            a_expected = c_expected ^ a_expected;
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, static_cast<uint_native>(0x10000000a3b10422));
        witness_ct second_input(&composer, static_cast<uint_native>(0xfafab007eac21343));

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a ^ b;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b = a;
            a = c;
            c = a + b;
            a = c ^ a;
        }
        uint_native a_result =
            static_cast<uint_native>(composer.get_variable(a.get_witness_index()).from_montgomery_form().data[0]);

        EXPECT_EQ(a_result, a_expected);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_xor_constants()
    {
        Composer composer = Composer();

        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected ^ b_expected;

        uint_ct const_a(&composer, static_cast<uint_native>(0x10000000a3b10422));
        uint_ct const_b(&composer, static_cast<uint_native>(0xfafab007eac21343));
        uint_ct c = const_a ^ const_b;
        c.get_witness_index();

        EXPECT_EQ(c.get_additive_constant(), uint256_t(c_expected));
    }

    static void test_xor_more_constants()
    {
        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected ^ b_expected;
        for (size_t i = 0; i < 1; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = (a_expected + b_expected) ^
                         (static_cast<uint_native>(0x10000000a3b10422) ^ static_cast<uint_native>(0xfafab007eac21343));
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, static_cast<uint_native>(0x10000000a3b10422));
        witness_ct second_input(&composer, static_cast<uint_native>(0xfafab007eac21343));

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a ^ b;
        for (size_t i = 0; i < 1; ++i) {
            uint_ct const_a = static_cast<uint_native>(0x10000000a3b10422);
            uint_ct const_b = static_cast<uint_native>(0xfafab007eac21343);
            b = a;
            a = c;
            c = (a + b) ^ (const_a ^ const_b);
        }
        uint32_t c_witness_index = c.get_witness_index();
        uint_native c_result =
            static_cast<uint_native>(composer.get_variable(c_witness_index).from_montgomery_form().data[0]);
        EXPECT_EQ(c_result, c_expected);
        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_and_constants()
    {
        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected & b_expected;
        for (size_t i = 0; i < 1; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = (~a_expected & static_cast<uint_native>(0x10000000a3b10422)) +
                         (b_expected & static_cast<uint_native>(0xfafab007eac21343));
            // c_expected = (a_expected + b_expected) & (static_cast<uint_native>(0x10000000a3b10422) &
            // static_cast<uint_native>(0xfafab007eac21343));
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, static_cast<uint_native>(0x10000000a3b10422));
        witness_ct second_input(&composer, static_cast<uint_native>(0xfafab007eac21343));

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a & b;
        for (size_t i = 0; i < 1; ++i) {
            uint_ct const_a = static_cast<uint_native>(0x10000000a3b10422);
            uint_ct const_b = static_cast<uint_native>(0xfafab007eac21343);
            b = a;
            a = c;
            c = (~a & const_a) + (b & const_b);
        }
        uint32_t c_witness_index = c.get_witness_index();
        uint_native c_result =
            static_cast<uint_native>(composer.get_variable(c_witness_index).from_montgomery_form().data[0]);
        EXPECT_EQ(c_result, c_expected);
        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_and_special()
    {
        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected + b_expected;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = a_expected + b_expected;
            a_expected = c_expected & a_expected;
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, static_cast<uint_native>(0x10000000a3b10422));
        witness_ct second_input(&composer, static_cast<uint_native>(0xfafab007eac21343));

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a + b;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b = a;
            a = c;
            c = a + b;
            a = c & a;
        }
        uint_native a_result =
            static_cast<uint_native>(composer.get_variable(a.get_witness_index()).from_montgomery_form().data[0]);
        EXPECT_EQ(a_result, a_expected);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_or_special()
    {
        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected ^ b_expected;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = a_expected + b_expected;
            a_expected = c_expected | a_expected;
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, static_cast<uint_native>(0x10000000a3b10422));
        witness_ct second_input(&composer, static_cast<uint_native>(0xfafab007eac21343));

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a ^ b;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b = a;
            a = c;
            c = a + b;
            a = c | a;
        }
        uint_native a_result =
            static_cast<uint_native>(composer.get_variable(a.get_witness_index()).from_montgomery_form().data[0]);
        EXPECT_EQ(a_result, a_expected);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_gt_special()
    {
        const auto run_test = [](bool lhs_constant, bool rhs_constant, int type = 0) {
            uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
            uint_native b_expected;
            switch (type) {
            case 0: {
                b_expected = static_cast<uint_native>(0x20000000bac21343); // a < b
                break;
            }
            case 1: {
                b_expected = static_cast<uint_native>(0x0000000000002f12); // a > b
                break;
            }
            case 2: {
                b_expected = static_cast<uint_native>(0x10000000a3b10422); // a = b
                break;
            }
            default: {
                b_expected = static_cast<uint_native>(0x20000000bac21343); // a < b
            }
            }
            bool c_expected = a_expected > b_expected;

            Composer composer = Composer();

            uint_ct a;
            uint_ct b;
            if (lhs_constant) {
                a = uint_ct(nullptr, a_expected);
            } else {
                a = witness_ct(&composer, a_expected);
            }
            if (rhs_constant) {
                b = uint_ct(nullptr, b_expected);
            } else {
                b = witness_ct(&composer, b_expected);
            }
            // mix in some constant terms for good measure
            a *= uint_ct(&composer, 2);
            a += uint_ct(&composer, 1);
            b *= uint_ct(&composer, 2);
            b += uint_ct(&composer, 1);

            bool_ct c = a > b;

            bool c_result = static_cast<bool>(c.get_value());
            EXPECT_EQ(c_result, c_expected);

            auto prover = composer.create_prover();

            auto verifier = composer.create_verifier();

            waffle::plonk_proof proof = prover.construct_proof();

            bool result = verifier.verify_proof(proof);
            EXPECT_EQ(result, true);
        };

        run_test(false, false, 0);
        run_test(false, true, 0);
        run_test(true, false, 0);
        run_test(true, true, 0);
        run_test(false, false, 1);
        run_test(false, true, 1);
        run_test(true, false, 1);
        run_test(true, true, 1);
        run_test(false, false, 2);
        run_test(false, true, 2);
        run_test(true, false, 2);
        run_test(true, true, 2);
    }

    static uint_native rotate(uint_native value, size_t rotation)
    {
        return rotation ? static_cast<uint_native>(value >> rotation) +
                              static_cast<uint_native>(value << (uint_native_width - rotation))
                        : value;
    }

    static void test_ror_special()
    {
        uint_native a_expected = static_cast<uint_native>(0x10000000a3b10422);
        uint_native b_expected = static_cast<uint_native>(0xfafab007eac21343);
        uint_native c_expected = a_expected ^ b_expected;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b_expected = a_expected;
            a_expected = c_expected;
            c_expected = a_expected + b_expected;
            a_expected = rotate(c_expected, i % 31) + rotate(a_expected, (i + 1) % 31);
        }

        Composer composer = Composer();

        witness_ct first_input(&composer, static_cast<uint_native>(0x10000000a3b10422));
        witness_ct second_input(&composer, static_cast<uint_native>(0xfafab007eac21343));

        uint_ct a = first_input;
        uint_ct b = second_input;
        uint_ct c = a ^ b;
        for (size_t i = 0; i < uint_native_width; ++i) {
            b = a;
            a = c;
            c = a + b;
            a = c.ror(static_cast<uint_native>(i % 31)) + a.ror(static_cast<uint_native>((i + 1) % 31));
        }
        uint_native a_result =
            static_cast<uint_native>(composer.get_variable(a.get_witness_index()).from_montgomery_form().data[0]);
        EXPECT_EQ(a_result, a_expected);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    /**
     * @brief If uint_native_width == 32, test part of SHA256. Otherwise, do something similar.
     *
     * @details Notes that the static casts have to be there becuase of -Wc++11-narrowing flag.
     *
     * TurboPLONK:     19896 gates
     * StandardPLONK: 210363 gates
     */
    static void test_hash_rounds()
    {
        std::vector<uint_native> k_constants(64);
        std::vector<uint_native> round_values(8);
        if (uint_native_width == 32) {
            k_constants = { static_cast<uint_native>(0x428a2f98), static_cast<uint_native>(0x71374491),
                            static_cast<uint_native>(0xb5c0fbcf), static_cast<uint_native>(0xe9b5dba5),
                            static_cast<uint_native>(0x3956c25b), static_cast<uint_native>(0x59f111f1),
                            static_cast<uint_native>(0x923f82a4), static_cast<uint_native>(0xab1c5ed5),
                            static_cast<uint_native>(0xd807aa98), static_cast<uint_native>(0x12835b01),
                            static_cast<uint_native>(0x243185be), static_cast<uint_native>(0x550c7dc3),
                            static_cast<uint_native>(0x72be5d74), static_cast<uint_native>(0x80deb1fe),
                            static_cast<uint_native>(0x9bdc06a7), static_cast<uint_native>(0xc19bf174),
                            static_cast<uint_native>(0xe49b69c1), static_cast<uint_native>(0xefbe4786),
                            static_cast<uint_native>(0x0fc19dc6), static_cast<uint_native>(0x240ca1cc),
                            static_cast<uint_native>(0x2de92c6f), static_cast<uint_native>(0x4a7484aa),
                            static_cast<uint_native>(0x5cb0a9dc), static_cast<uint_native>(0x76f988da),
                            static_cast<uint_native>(0x983e5152), static_cast<uint_native>(0xa831c66d),
                            static_cast<uint_native>(0xb00327c8), static_cast<uint_native>(0xbf597fc7),
                            static_cast<uint_native>(0xc6e00bf3), static_cast<uint_native>(0xd5a79147),
                            static_cast<uint_native>(0x06ca6351), static_cast<uint_native>(0x14292967),
                            static_cast<uint_native>(0x27b70a85), static_cast<uint_native>(0x2e1b2138),
                            static_cast<uint_native>(0x4d2c6dfc), static_cast<uint_native>(0x53380d13),
                            static_cast<uint_native>(0x650a7354), static_cast<uint_native>(0x766a0abb),
                            static_cast<uint_native>(0x81c2c92e), static_cast<uint_native>(0x92722c85),
                            static_cast<uint_native>(0xa2bfe8a1), static_cast<uint_native>(0xa81a664b),
                            static_cast<uint_native>(0xc24b8b70), static_cast<uint_native>(0xc76c51a3),
                            static_cast<uint_native>(0xd192e819), static_cast<uint_native>(0xd6990624),
                            static_cast<uint_native>(0xf40e3585), static_cast<uint_native>(0x106aa070),
                            static_cast<uint_native>(0x19a4c116), static_cast<uint_native>(0x1e376c08),
                            static_cast<uint_native>(0x2748774c), static_cast<uint_native>(0x34b0bcb5),
                            static_cast<uint_native>(0x391c0cb3), static_cast<uint_native>(0x4ed8aa4a),
                            static_cast<uint_native>(0x5b9cca4f), static_cast<uint_native>(0x682e6ff3),
                            static_cast<uint_native>(0x748f82ee), static_cast<uint_native>(0x78a5636f),
                            static_cast<uint_native>(0x84c87814), static_cast<uint_native>(0x8cc70208),
                            static_cast<uint_native>(0x90befffa), static_cast<uint_native>(0xa4506ceb),
                            static_cast<uint_native>(0xbef9a3f7), static_cast<uint_native>(0xc67178f2) };

            round_values = { static_cast<uint_native>(0x01020304), static_cast<uint_native>(0x0a0b0c0d),
                             static_cast<uint_native>(0x1a2b3e4d), static_cast<uint_native>(0x03951bd3),
                             static_cast<uint_native>(0x0e0fa3fe), static_cast<uint_native>(0x01000000),
                             static_cast<uint_native>(0x0f0eeea1), static_cast<uint_native>(0x12345678) };
        } else {
            k_constants = get_several_random<uint_native>(64);
            round_values = get_several_random<uint_native>(8);
        };

        std::vector<uint_native> w_alt = get_several_random<uint_native>(64);

        uint_native a_alt = round_values[0];
        uint_native b_alt = round_values[1];
        uint_native c_alt = round_values[2];
        uint_native d_alt = round_values[3];
        uint_native e_alt = round_values[4];
        uint_native f_alt = round_values[5];
        uint_native g_alt = round_values[6];
        uint_native h_alt = round_values[7];
        for (size_t i = 0; i < 64; ++i) {
            uint_native S1_alt = rotate(e_alt, 7 % uint_native_width) ^ rotate(e_alt, 11 % uint_native_width) ^
                                 rotate(e_alt, 25 % uint_native_width);
            uint_native ch_alt = (e_alt & f_alt) ^ ((~e_alt) & g_alt);
            uint_native temp1_alt = h_alt + S1_alt + ch_alt + k_constants[i % 64] + w_alt[i];

            uint_native S0_alt = rotate(a_alt, 2 % uint_native_width) ^ rotate(a_alt, 13 % uint_native_width) ^
                                 rotate(a_alt, 22 % uint_native_width);
            uint_native maj_alt = (a_alt & b_alt) ^ (a_alt & c_alt) ^ (b_alt & c_alt);
            uint_native temp2_alt = S0_alt + maj_alt;

            h_alt = g_alt;
            g_alt = f_alt;
            f_alt = e_alt;
            e_alt = d_alt + temp1_alt;
            d_alt = c_alt;
            c_alt = b_alt;
            b_alt = a_alt;
            a_alt = temp1_alt + temp2_alt;
        }
        Composer composer = Composer();

        std::vector<uint_ct> w;
        std::vector<uint_ct> k;
        for (size_t i = 0; i < 64; ++i) {
            w.emplace_back(uint_ct(witness_ct(&composer, w_alt[i])));
            k.emplace_back(uint_ct(&composer, k_constants[i % 64]));
        }
        uint_ct a = witness_ct(&composer, round_values[0]);
        uint_ct b = witness_ct(&composer, round_values[1]);
        uint_ct c = witness_ct(&composer, round_values[2]);
        uint_ct d = witness_ct(&composer, round_values[3]);
        uint_ct e = witness_ct(&composer, round_values[4]);
        uint_ct f = witness_ct(&composer, round_values[5]);
        uint_ct g = witness_ct(&composer, round_values[6]);
        uint_ct h = witness_ct(&composer, round_values[7]);
        for (size_t i = 0; i < 64; ++i) {
            uint_ct S1 =
                e.ror(7U % uint_native_width) ^ e.ror(11U % uint_native_width) ^ e.ror(25U % uint_native_width);
            uint_ct ch = (e & f) + ((~e) & g);
            uint_ct temp1 = h + S1 + ch + k[i] + w[i];

            uint_ct S0 =
                a.ror(2U % uint_native_width) ^ a.ror(13U % uint_native_width) ^ a.ror(22U % uint_native_width);
            uint_ct T0 = (b & c);
            uint_ct T1 = (b - T0) + (c - T0);
            uint_ct T2 = a & T1;
            uint_ct maj = T2 + T0;
            uint_ct temp2 = S0 + maj;

            h = g;
            g = f;
            f = e;
            e = d + temp1;
            d = c;
            c = b;
            b = a;
            a = temp1 + temp2;
        }

        uint_native a_result =
            static_cast<uint_native>(composer.get_variable(a.get_witness_index()).from_montgomery_form().data[0]);
        uint_native b_result =
            static_cast<uint_native>(composer.get_variable(b.get_witness_index()).from_montgomery_form().data[0]);
        uint_native c_result =
            static_cast<uint_native>(composer.get_variable(c.get_witness_index()).from_montgomery_form().data[0]);
        uint_native d_result =
            static_cast<uint_native>(composer.get_variable(d.get_witness_index()).from_montgomery_form().data[0]);
        uint_native e_result =
            static_cast<uint_native>(composer.get_variable(e.get_witness_index()).from_montgomery_form().data[0]);
        uint_native f_result =
            static_cast<uint_native>(composer.get_variable(f.get_witness_index()).from_montgomery_form().data[0]);
        uint_native g_result =
            static_cast<uint_native>(composer.get_variable(g.get_witness_index()).from_montgomery_form().data[0]);
        uint_native h_result =
            static_cast<uint_native>(composer.get_variable(h.get_witness_index()).from_montgomery_form().data[0]);

        EXPECT_EQ(a_result, a_alt);
        EXPECT_EQ(b_result, b_alt);
        EXPECT_EQ(c_result, c_alt);
        EXPECT_EQ(d_result, d_alt);
        EXPECT_EQ(e_result, e_alt);
        EXPECT_EQ(f_result, f_alt);
        EXPECT_EQ(g_result, g_alt);
        EXPECT_EQ(h_result, h_alt);

        auto prover = composer.create_prover();

        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    // BELOW HERE ARE TESTS FORMERLY MARKED AS TURBO

    /**
     * @brief Test addition of random uint's, trying all combinations of (constant, witness).
     */
    static void test_add()
    {
        Composer composer = Composer();

        const auto add_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = get_random<uint_native>();
            uint_native expected = a_val + b_val;
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct c = a + b;
            c = c.normalize();

            uint_native result = uint_native(c.get_value());

            EXPECT_EQ(result, expected);
        };

        add_integers(false, false);
        add_integers(false, true);
        add_integers(true, false);
        add_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_sub()
    {
        Composer composer = Composer();

        const auto sub_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = get_random<uint_native>();
            uint_native const_shift_val = get_random<uint_native>();
            uint_native expected = a_val - (b_val + const_shift_val);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct b_shift = uint_ct(&composer, const_shift_val);
            uint_ct c = b + b_shift;
            uint_ct d = a - c;
            d = d.normalize();

            uint_native result = uint_native(d.get_value());

            EXPECT_EQ(result, expected);
        };

        sub_integers(false, false);
        sub_integers(false, true);
        sub_integers(true, false);
        sub_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_mul()
    {
        Composer composer = Composer();

        const auto mul_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = get_random<uint_native>();
            uint_native const_a = get_random<uint_native>();
            uint_native const_b = get_random<uint_native>();
            uint_native expected =
                static_cast<uint_native>(a_val + const_a) * static_cast<uint_native>(b_val + const_b);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c * d;
            e = e.normalize();

            uint_native result = uint_native(e.get_value());

            EXPECT_EQ(result, expected);
        };

        mul_integers(false, false);
        mul_integers(false, true);
        mul_integers(true, false);
        mul_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_divide()
    {
        Composer composer = Composer();

        const auto divide_integers = [&composer](bool lhs_constant = false,
                                                 bool rhs_constant = false,
                                                 bool dividend_is_divisor = false,
                                                 bool dividend_zero = false,
                                                 bool divisor_zero = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = dividend_is_divisor ? a_val : get_random<uint_native>();
            uint_native const_a = dividend_zero ? 0 - a_val : get_random<uint_native>();
            uint_native const_b =
                divisor_zero ? 0 - b_val : (dividend_is_divisor ? const_a : get_random<uint_native>());
            uint_native expected =
                static_cast<uint_native>(a_val + const_a) / static_cast<uint_native>(b_val + const_b);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c / d;
            e = e.normalize();

            uint_native result = static_cast<uint_native>(e.get_value());

            EXPECT_EQ(result, expected);
        };

        divide_integers(false, false, false, false, false);
        divide_integers(false, false, false, false, false);
        divide_integers(false, false, false, false, false);
        divide_integers(false, false, false, false, false);
        divide_integers(false, false, false, false, false);

        divide_integers(false, true, false, false, false);
        divide_integers(true, false, false, false, false);
        divide_integers(true, true, false, false, false); // fails; 0 != 1

        divide_integers(false, false, true, false, false);
        divide_integers(false, true, true, false, false);
        divide_integers(true, false, true, false, false);
        divide_integers(true, true, true, false, false);

        divide_integers(false, false, false, true, false);
        divide_integers(false, true, false, true, false); // fails; 0 != 1
        divide_integers(true, false, false, true, false);
        divide_integers(true, true, false, true, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_modulo()
    {
        Composer composer = Composer();

        const auto mod_integers = [&composer](bool lhs_constant = false,
                                              bool rhs_constant = false,
                                              bool dividend_is_divisor = false,
                                              bool dividend_zero = false,
                                              bool divisor_zero = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = dividend_is_divisor ? a_val : get_random<uint_native>();
            uint_native const_a = dividend_zero ? 0 - a_val : get_random<uint_native>();
            uint_native const_b =
                divisor_zero ? 0 - b_val : (dividend_is_divisor ? const_a : get_random<uint_native>());
            uint_native expected =
                static_cast<uint_native>(a_val + const_a) % static_cast<uint_native>(b_val + const_b);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c % d;
            e = e.normalize();

            uint_native result = uint_native(e.get_value());

            EXPECT_EQ(result, expected);
        };

        mod_integers(false, false, false, false, false);
        mod_integers(false, true, false, false, false);
        mod_integers(true, false, false, false, false);
        mod_integers(true, true, false, false, false);

        mod_integers(false, false, true, false, false);
        mod_integers(false, true, true, false, false);
        mod_integers(true, false, true, false, false);
        mod_integers(true, true, true, false, false);

        mod_integers(false, false, false, true, false);
        mod_integers(false, true, false, true, false);
        mod_integers(true, false, false, true, false);
        mod_integers(true, true, false, true, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_divide_by_zero_fails()
    {

        const auto divide_integers = [](bool lhs_constant = false,
                                        bool rhs_constant = false,
                                        bool dividend_is_divisor = false,
                                        bool dividend_zero = false,
                                        bool divisor_zero = false) {
            Composer composer = Composer();

            uint_native a_val = get_random<uint_native>();
            uint_native b_val = dividend_is_divisor ? a_val : get_random<uint_native>();
            uint_native const_a = dividend_zero ? 0 - a_val : get_random<uint_native>();
            uint_native const_b =
                divisor_zero ? 0 - b_val : (dividend_is_divisor ? const_a : get_random<uint_native>());
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c / d;
            e = e.normalize();

            auto prover = composer.create_prover();

            auto verifier = composer.create_verifier();

            waffle::plonk_proof proof = prover.construct_proof();

            bool proof_result = verifier.verify_proof(proof);
            EXPECT_EQ(proof_result, false);
        };

        divide_integers(false, false, false, false, true);
        divide_integers(false, false, false, true, true);
        divide_integers(true, true, false, false, true);
        divide_integers(true, true, false, true, true);
    }

    static void test_divide_special()
    {
        Composer composer = Composer();

        auto special_uints = get_special_uints(&composer);

        for (size_t i = 0; i != special_values.size(); ++i) {
            uint_native x = special_values[i];
            uint_ct x_ct = special_uints[i];

            for (size_t j = i; j != special_values.size(); ++j) {
                uint_native y = special_values[j];
                uint_ct y_ct = special_uints[j];

                // uint_native hits this error when trying to divide by zero:
                // Stop reason: signal SIGFPE: integer divide by zero
                uint_native expected_value;
                uint_ct z_ct;
                uint_native value;
                if (y != 0) {
                    expected_value = x / y;
                    z_ct = x_ct / y_ct;
                    value = static_cast<uint_native>(z_ct.get_value());
                    EXPECT_EQ(value, expected_value);
                }
            }
        };

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    /**
     * @brief Make sure we prevent proving v / v = 0 by setting the divison remainder to be v.
     * TODO: This is lifted from the implementation. Should rewrite this test after introducing framework that separates
     * circuit construction from witness generation.

     */
    static void div_remainder_constraint()
    {
        Composer composer = Composer();

        uint_native val = get_random<uint_native>();

        uint_ct a = witness_ct(&composer, val);
        uint_ct b = witness_ct(&composer, val);

        const uint32_t dividend_idx = a.get_witness_index();
        const uint32_t divisor_idx = b.get_witness_index();

        const uint256_t divisor = b.get_value();

        const uint256_t q = 0;
        const uint256_t r = val;

        const uint32_t quotient_idx = composer.add_variable(q);
        const uint32_t remainder_idx = composer.add_variable(r);

        // In this example there are no additive constaints, so we just replace them by zero below.

        // constraint: qb + const_b q + 0 b - a + r - const_a == 0
        // i.e., a + const_a = q(b + const_b) + r
        const mul_quad division_gate{ .a = quotient_idx,  // q
                                      .b = divisor_idx,   // b
                                      .c = dividend_idx,  // a
                                      .d = remainder_idx, // r
                                      .mul_scaling = fr::one(),
                                      .a_scaling = b.get_additive_constant(),
                                      .b_scaling = fr::zero(),
                                      .c_scaling = fr::neg_one(),
                                      .d_scaling = fr::one(),
                                      .const_scaling = -a.get_additive_constant() };
        composer.create_big_mul_gate(division_gate);

        // set delta = (b + const_b - r)

        // constraint: b - r - delta + const_b == 0
        const uint256_t delta = divisor - r - 1;
        const uint32_t delta_idx = composer.add_variable(delta);

        const add_triple delta_gate{
            .a = divisor_idx,   // b
            .b = remainder_idx, // r
            .c = delta_idx,     // d
            .a_scaling = fr::one(),
            .b_scaling = fr::neg_one(),
            .c_scaling = fr::neg_one(),
            .const_scaling = b.get_additive_constant(),
        };

        composer.create_add_gate(delta_gate);

        // validate delta is in the correct range
        stdlib::field_t<Composer>::from_witness_index(&composer, delta_idx)
            .create_range_constraint(uint_native_width,
                                     "delta range constraint fails in div_remainder_constraint test");

        // normalize witness quotient and remainder
        // minimal bit range for quotient: from 0 (in case a = b-1) to width (when b = 1).
        uint_ct quotient(&composer);
        composer.create_range_constraint(
            quotient_idx, uint_native_width, "quotient range constraint fails in div_remainder_constraint test");

        // constrain remainder to lie in [0, 2^width-1]
        uint_ct remainder(&composer);
        composer.create_range_constraint(
            remainder_idx, uint_native_width, "remainder range constraint fails in div_remainder_constraint test");

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }

    static void test_and()
    {
        Composer composer = Composer();

        const auto and_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = get_random<uint_native>();
            uint_native const_a = get_random<uint_native>();
            uint_native const_b = get_random<uint_native>();
            uint_native expected = (a_val + const_a) & (b_val + const_b);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c & d;
            e = e.normalize();

            uint_native result = uint_native(e.get_value());

            EXPECT_EQ(result, expected);
        };

        and_integers(false, false);
        and_integers(false, true);
        and_integers(true, false);
        and_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_xor()
    {
        Composer composer = Composer();

        const auto xor_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = get_random<uint_native>();
            uint_native const_a = get_random<uint_native>();
            uint_native const_b = get_random<uint_native>();
            uint_native expected = (a_val + const_a) ^ (b_val + const_b);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c ^ d;
            e = e.normalize();

            uint_native result = uint_native(e.get_value());

            EXPECT_EQ(result, expected);
        };

        xor_integers(false, false);
        xor_integers(false, true);
        xor_integers(true, false);
        xor_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_or()
    {
        Composer composer = Composer();

        const auto or_integers = [&composer](bool lhs_constant = false, bool rhs_constant = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native b_val = get_random<uint_native>();
            uint_native const_a = get_random<uint_native>();
            uint_native const_b = get_random<uint_native>();
            uint_native expected = (a_val + const_a) | (b_val + const_b);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct b = rhs_constant ? uint_ct(&composer, b_val) : witness_ct(&composer, b_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct b_shift = uint_ct(&composer, const_b);
            uint_ct c = a + a_shift;
            uint_ct d = b + b_shift;
            uint_ct e = c | d;
            e = e.normalize();

            uint_native result = uint_native(e.get_value());

            EXPECT_EQ(result, expected);
        };

        or_integers(false, false);
        or_integers(false, false);
        or_integers(false, false);
        or_integers(false, false);
        or_integers(false, false);
        or_integers(false, true);
        or_integers(true, false);
        or_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_not()
    {
        Composer composer = Composer();

        const auto not_integers = [&composer](bool lhs_constant = false, bool = false) {
            uint_native a_val = get_random<uint_native>();
            uint_native const_a = get_random<uint_native>();
            uint_native expected = ~(a_val + const_a);
            uint_ct a = lhs_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            uint_ct e = ~c;
            e = e.normalize();

            uint_native result = uint_native(e.get_value());

            EXPECT_EQ(result, expected);
        };

        not_integers(false, false);
        not_integers(false, true);
        not_integers(true, false);
        not_integers(true, true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_gt()
    {
        Composer composer = Composer();
        const auto compare_integers =
            [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
                uint_native const_a = get_random<uint_native>();
                uint_native const_b = get_random<uint_native>();
                uint_native a_val = get_random<uint_native>();
                uint_native b_val = impose_comparison(const_a, const_b, a_val, force_equal, force_gt, force_lt);

                bool expected = static_cast<uint_native>(b_val + const_b) > static_cast<uint_native>(a_val + const_a);
                uint_ct a = witness_ct(&composer, a_val);
                uint_ct b = witness_ct(&composer, b_val);
                uint_ct a_shift = uint_ct(&composer, const_a);
                uint_ct b_shift = uint_ct(&composer, const_b);
                uint_ct c = a + a_shift;
                uint_ct d = b + b_shift;
                bool_ct e = d > c;
                bool result = bool(e.get_value());

                EXPECT_EQ(result, expected);
            };

        compare_integers(false, false, false); // both are random
        compare_integers(false, false, false); //       ''
        compare_integers(false, false, false); //       ''
        compare_integers(false, false, false); //       ''
        compare_integers(false, false, true);  //      b < a
        compare_integers(false, true, false);  //      b > a
        compare_integers(true, false, false);  //      b = a
        compare_integers(false, true, false);  //      b > a
        compare_integers(true, false, false);  //      b = a

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_lt()
    {
        Composer composer = Composer();

        const auto compare_integers =
            [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
                uint_native const_a = get_random<uint_native>();
                uint_native const_b = get_random<uint_native>();
                uint_native a_val = get_random<uint_native>();
                uint_native b_val = impose_comparison(const_a, const_b, a_val, force_equal, force_gt, force_lt);

                bool expected = static_cast<uint_native>(b_val + const_b) < static_cast<uint_native>(a_val + const_a);
                uint_ct a = witness_ct(&composer, a_val);
                uint_ct b = witness_ct(&composer, b_val);
                uint_ct a_shift = uint_ct(&composer, const_a);
                uint_ct b_shift = uint_ct(&composer, const_b);
                uint_ct c = a + a_shift;
                uint_ct d = b + b_shift;
                bool_ct e = d < c;
                bool result = bool(e.get_value());

                EXPECT_EQ(result, expected);
            };

        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_gte()
    {
        Composer composer = Composer();

        const auto compare_integers =
            [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
                uint_native const_a = get_random<uint_native>();
                uint_native const_b = get_random<uint_native>();
                uint_native a_val = get_random<uint_native>();
                uint_native b_val = impose_comparison(const_a, const_b, a_val, force_equal, force_gt, force_lt);

                bool expected = static_cast<uint_native>(b_val + const_b) >= static_cast<uint_native>(a_val + const_a);
                uint_ct a = witness_ct(&composer, a_val);
                uint_ct b = witness_ct(&composer, b_val);
                uint_ct a_shift = uint_ct(&composer, const_a);
                uint_ct b_shift = uint_ct(&composer, const_b);
                uint_ct c = a + a_shift;
                uint_ct d = b + b_shift;
                bool_ct e = d >= c;
                bool result = bool(e.get_value());
                EXPECT_EQ(result, expected);
            };

        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_lte()
    {
        Composer composer = Composer();

        const auto compare_integers =
            [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
                uint_native const_a = get_random<uint_native>();
                uint_native const_b = get_random<uint_native>();
                uint_native a_val = get_random<uint_native>();
                uint_native b_val = impose_comparison(const_a, const_b, a_val, force_equal, force_gt, force_lt);

                bool expected = static_cast<uint_native>(b_val + const_b) <= static_cast<uint_native>(a_val + const_a);
                uint_ct a = witness_ct(&composer, a_val);
                uint_ct b = witness_ct(&composer, b_val);
                uint_ct a_shift = uint_ct(&composer, const_a);
                uint_ct b_shift = uint_ct(&composer, const_b);
                uint_ct c = a + a_shift;
                uint_ct d = b + b_shift;
                bool_ct e = d <= c;
                bool result = bool(e.get_value());

                EXPECT_EQ(result, expected);
            };

        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_equality_operator()
    {
        Composer composer = Composer();

        const auto compare_integers =
            [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
                uint_native const_a = get_random<uint_native>();
                uint_native const_b = get_random<uint_native>();
                uint_native a_val = get_random<uint_native>();
                uint_native b_val = impose_comparison(const_a, const_b, a_val, force_equal, force_gt, force_lt);

                bool expected = static_cast<uint_native>(b_val + const_b) == static_cast<uint_native>(a_val + const_a);
                uint_ct a = witness_ct(&composer, a_val);
                uint_ct b = witness_ct(&composer, b_val);
                uint_ct a_shift = uint_ct(&composer, const_a);
                uint_ct b_shift = uint_ct(&composer, const_b);
                uint_ct c = a + a_shift;
                uint_ct d = b + b_shift;
                bool_ct e = d == c;
                bool result = bool(e.get_value());

                EXPECT_EQ(result, expected);
            };

        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_not_equality_operator()
    {
        Composer composer = Composer();

        const auto compare_integers =
            [&composer](bool force_equal = false, bool force_gt = false, bool force_lt = false) {
                uint_native const_a = get_random<uint_native>();
                uint_native const_b = get_random<uint_native>();
                uint_native a_val = get_random<uint_native>();
                uint_native b_val = impose_comparison(const_a, const_b, a_val, force_equal, force_gt, force_lt);

                bool expected = static_cast<uint_native>(b_val + const_b) != static_cast<uint_native>(a_val + const_a);
                uint_ct a = witness_ct(&composer, a_val);
                uint_ct b = witness_ct(&composer, b_val);
                uint_ct a_shift = uint_ct(&composer, const_a);
                uint_ct b_shift = uint_ct(&composer, const_b);
                uint_ct c = a + a_shift;
                uint_ct d = b + b_shift;
                bool_ct e = d != c;
                bool result = bool(e.get_value());

                EXPECT_EQ(result, expected);
            };

        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);
        compare_integers(false, false, true);
        compare_integers(false, true, false);
        compare_integers(true, false, false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_logical_not()
    {
        Composer composer = Composer();

        const auto not_integer = [&composer](bool force_zero) {
            uint_native const_a = get_random<uint_native>();
            uint_native a_val = force_zero ? 0 - const_a : get_random<uint_native>();
            bool expected = !static_cast<uint_native>(const_a + a_val);
            uint_ct a = witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            bool_ct e = !c;
            bool result = bool(e.get_value());

            EXPECT_EQ(result, expected);
        };

        not_integer(true);
        not_integer(true);
        not_integer(false);
        not_integer(false);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_right_shift()
    {
        Composer composer = Composer();

        const auto shift_integer = [&composer](const bool is_constant, const uint_native shift) {
            uint_native const_a = get_random<uint_native>();
            uint_native a_val = get_random<uint_native>();
            uint_native expected = static_cast<uint_native>(a_val + const_a) >> shift;
            uint_ct a = is_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            uint_ct d = c >> shift;
            uint_native result = uint_native(d.get_value());

            EXPECT_EQ(result, expected);
        };

        for (uint_native i = 0; i < uint_native_width; ++i) {
            shift_integer(false, i);
            shift_integer(true, i);
        }

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_left_shift()
    {
        Composer composer = Composer();

        const auto shift_integer = [&composer](const bool is_constant, const uint_native shift) {
            uint_native const_a = get_random<uint_native>();
            uint_native a_val = get_random<uint_native>();
            uint_native expected = static_cast<uint_native>((a_val + const_a) << shift);
            uint_ct a = is_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            uint_ct d = c << shift;
            uint_native result = uint_native(d.get_value());

            EXPECT_EQ(result, expected);
        };

        for (uint_native i = 0; i < uint_native_width; ++i) {
            shift_integer(true, i);
            shift_integer(false, i);
        }

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_ror()
    {
        Composer composer = Composer();

        const auto ror_integer = [&composer](const bool is_constant, const uint_native rotation) {
            const auto ror = [](const uint_native in, const uint_native rval) {
                return rval ? (in >> rval) | (in << (uint_native_width - rval)) : in;
            };

            uint_native const_a = get_random<uint_native>();
            uint_native a_val = get_random<uint_native>();
            uint_native expected = static_cast<uint_native>(ror(static_cast<uint_native>(const_a + a_val), rotation));
            uint_ct a = is_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            uint_ct d = c.ror(rotation);
            uint_native result = uint_native(d.get_value());

            EXPECT_EQ(result, expected);
        };

        for (uint_native i = 0; i < uint_native_width; ++i) {
            ror_integer(true, i);
            ror_integer(false, i);
        }

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_rol()
    {
        Composer composer = Composer();

        const auto rol_integer = [&composer](const bool is_constant, const uint_native rotation) {
            const auto rol = [](const uint_native in, const uint_native rval) {
                return rval ? (in << rval) | (in >> (uint_native_width - rval)) : in;
            };

            uint_native const_a = get_random<uint_native>();
            uint_native a_val = get_random<uint_native>();
            uint_native expected = static_cast<uint_native>(rol(static_cast<uint_native>(const_a + a_val), rotation));
            uint_ct a = is_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            uint_ct d = c.rol(rotation);
            uint_native result = uint_native(d.get_value());

            EXPECT_EQ(result, expected);
        };

        for (uint_native i = 0; i < uint_native_width; ++i) {
            rol_integer(true, i);
            rol_integer(false, i);
        }

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    /**
     * @brief Test the the function uint_ct::at used to extract bits.
     */
    static void test_at()
    {
        Composer composer = Composer();

        const auto bit_test = [&composer](const bool is_constant) {
            // construct a sum of uint_ct's, where at least one is a constant,
            // and validate its correctness bitwise
            uint_native const_a = get_random<uint_native>();
            uint_native a_val = get_random<uint_native>();
            uint_native c_val = const_a + a_val;
            uint_ct a = is_constant ? uint_ct(&composer, a_val) : witness_ct(&composer, a_val);
            uint_ct a_shift = uint_ct(&composer, const_a);
            uint_ct c = a + a_shift;
            for (size_t i = 0; i < uint_native_width; ++i) {
                bool_ct result = c.at(i);
                bool expected = (((c_val >> i) & 1UL) == 1UL) ? true : false;
                EXPECT_EQ(result.get_value(), expected);
                EXPECT_EQ(result.get_context(), c.get_context());
            }
        };

        bit_test(false);
        bit_test(true);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }
};

typedef testing::
    Types<waffle::UltraComposer, waffle::TurboComposer, waffle::StandardComposer, honk::StandardHonkComposer>
        ComposerTypes;

TYPED_TEST_SUITE(stdlib_uint, ComposerTypes);

TYPED_TEST(stdlib_uint, test_weak_normalize)
{
    TestFixture::test_weak_normalize();
}
TYPED_TEST(stdlib_uint, test_byte_array_conversion)
{
    TestFixture::test_byte_array_conversion();
}
TYPED_TEST(stdlib_uint, test_input_output_consistency)
{
    TestFixture::test_input_output_consistency();
}
TYPED_TEST(stdlib_uint, test_create_from_wires)
{
    TestFixture::test_create_from_wires();
}
TYPED_TEST(stdlib_uint, test_add_special)
{
    TestFixture::test_add_special();
}
TYPED_TEST(stdlib_uint, test_sub_special)
{
    TestFixture::test_sub_special();
}
TYPED_TEST(stdlib_uint, test_add_with_constants)
{
    TestFixture::test_add_with_constants();
}
TYPED_TEST(stdlib_uint, test_mul_special)
{
    TestFixture::test_mul_special();
}
TYPED_TEST(stdlib_uint, test_mul_big)
{
    TestFixture::test_mul_big();
}
TYPED_TEST(stdlib_uint, test_xor_special)
{
    TestFixture::test_xor_special();
}
TYPED_TEST(stdlib_uint, test_xor_constants)
{
    TestFixture::test_xor_constants();
}
TYPED_TEST(stdlib_uint, test_xor_more_constants)
{
    TestFixture::test_xor_more_constants();
}
TYPED_TEST(stdlib_uint, test_and_constants)
{
    TestFixture::test_and_constants();
}
TYPED_TEST(stdlib_uint, test_and_special)
{
    TestFixture::test_and_special();
}
TYPED_TEST(stdlib_uint, test_or_special)
{
    TestFixture::test_or_special();
}
TYPED_TEST(stdlib_uint, test_gt_special)
{
    TestFixture::test_gt_special();
}
TYPED_TEST(stdlib_uint, test_ror_special)
{
    TestFixture::test_ror_special();
}
TYPED_TEST(stdlib_uint, test_hash_rounds)
{
    TestFixture::test_hash_rounds();
}
// BELOW HERE ARE TESTS FORMERLY MARKED AS TURBO
TYPED_TEST(stdlib_uint, test_add)
{
    TestFixture::test_add();
}
TYPED_TEST(stdlib_uint, test_sub)
{
    TestFixture::test_sub();
}
TYPED_TEST(stdlib_uint, test_mul)
{
    TestFixture::test_mul();
}
TYPED_TEST(stdlib_uint, test_divide)
{
    TestFixture::test_divide();
}
TYPED_TEST(stdlib_uint, test_modulo)
{
    TestFixture::test_modulo();
}
TYPED_TEST(stdlib_uint, test_divide_by_zero_fails)
{
    TestFixture::test_divide_by_zero_fails();
}
TYPED_TEST(stdlib_uint, test_divide_special)
{
    TestFixture::test_divide_special();
}
TYPED_TEST(stdlib_uint, div_remainder_constraint)
{
    TestFixture::div_remainder_constraint();
}
TYPED_TEST(stdlib_uint, test_and)
{
    TestFixture::test_and();
}
TYPED_TEST(stdlib_uint, test_xor)
{
    TestFixture::test_xor();
}
TYPED_TEST(stdlib_uint, test_or)
{
    TestFixture::test_or();
}
TYPED_TEST(stdlib_uint, test_not)
{
    TestFixture::test_not();
}
TYPED_TEST(stdlib_uint, test_gt)
{
    TestFixture::test_gt();
}
TYPED_TEST(stdlib_uint, test_lt)
{
    TestFixture::test_lt();
}
TYPED_TEST(stdlib_uint, test_gte)
{
    TestFixture::test_gte();
}
TYPED_TEST(stdlib_uint, test_lte)
{
    TestFixture::test_lte();
}
TYPED_TEST(stdlib_uint, test_equality_operator)
{
    TestFixture::test_equality_operator();
}
TYPED_TEST(stdlib_uint, test_not_equality_operator)
{
    TestFixture::test_not_equality_operator();
}
TYPED_TEST(stdlib_uint, test_logical_not)
{
    TestFixture::test_logical_not();
}
TYPED_TEST(stdlib_uint, test_right_shift)
{
    TestFixture::test_right_shift();
}
TYPED_TEST(stdlib_uint, test_left_shift)
{
    TestFixture::test_left_shift();
}
TYPED_TEST(stdlib_uint, test_ror)
{
    TestFixture::test_ror();
}
TYPED_TEST(stdlib_uint, test_rol)
{
    TestFixture::test_rol();
}
TYPED_TEST(stdlib_uint, test_at)
{
    TestFixture::test_at();
}

// There was one plookup-specific test in the ./plookup/uint_plookup.test.cpp
TEST(stdlib_uint32, test_accumulators_plookup_uint32)
{
    using uint32_ct = plonk::stdlib::uint32<waffle::UltraComposer>;
    using witness_ct = plonk::stdlib::witness_t<waffle::UltraComposer>;

    waffle::UltraComposer composer = waffle::UltraComposer();

    uint32_t a_val = engine.get_random_uint32();
    uint32_t b_val = engine.get_random_uint32();
    uint32_ct a = witness_ct(&composer, a_val);
    uint32_ct b = witness_ct(&composer, b_val);
    a = a ^ b;
    uint32_t val = a_val ^ b_val;
    uint32_t MASK = (1U << uint32_ct::bits_per_limb) - 1;
    const auto accumulators = a.get_accumulators();
    for (size_t i = 0; i < uint32_ct::num_accumulators(); ++i) {
        const uint64_t result = uint256_t(composer.get_variable(accumulators[i])).data[0];
        const uint64_t expected = val & MASK;
        val = val >> uint32_ct::bits_per_limb;
        EXPECT_EQ(result, expected);
    }

    printf("calling preprocess\n");
    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
} // namespace test_stdlib_uint
