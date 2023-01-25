#include "bit_array.hpp"
#include <gtest/gtest.h>
#include <numeric/random/engine.hpp>
#include <honk/composer/standard_honk_composer.hpp>
// #include <plonk/composer/standard_composer.hpp>

namespace test_stdlib_bit_array {

using namespace barretenberg;
using namespace plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

typedef stdlib::bool_t<honk::StandardHonkComposer> bool_t;
typedef stdlib::field_t<honk::StandardHonkComposer> field_t;
typedef stdlib::uint32<honk::StandardHonkComposer> uint32;
typedef stdlib::witness_t<honk::StandardHonkComposer> witness_t;
typedef stdlib::bit_array<honk::StandardHonkComposer> bit_array;
typedef stdlib::byte_array<honk::StandardHonkComposer> byte_array;

TEST(stdlib_bit_array, test_uint32_input_output_consistency)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    uint32_t a_expected = engine.get_random_uint32();
    uint32_t b_expected = engine.get_random_uint32();

    uint32 a = witness_t(&composer, a_expected);
    uint32 b = witness_t(&composer, b_expected);

    std::vector<uint32> inputs = { a, b };
    bit_array test_bit_array = bit_array(inputs);

    std::vector<uint32> result = test_bit_array.to_uint32_vector();

    EXPECT_EQ(result.size(), 2UL);

    uint32_t a_result =
        static_cast<uint32_t>(composer.get_variable(result[0].get_witness_index()).from_montgomery_form().data[0]);
    uint32_t b_result =
        static_cast<uint32_t>(composer.get_variable(result[1].get_witness_index()).from_montgomery_form().data[0]);

    EXPECT_EQ(a_result, a_expected);
    EXPECT_EQ(b_result, b_expected);
}

TEST(stdlib_bit_array, test_binary_input_output_consistency)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    bit_array test_bit_array = bit_array(&composer, 5);

    test_bit_array[0] = bool_t(witness_t(&composer, true));
    test_bit_array[1] = bool_t(witness_t(&composer, false));
    test_bit_array[2] = bool_t(witness_t(&composer, true));
    test_bit_array[3] = bool_t(witness_t(&composer, true));
    test_bit_array[4] = bool_t(witness_t(&composer, false));

    std::vector<uint32> uint32_vec = test_bit_array.to_uint32_vector();

    EXPECT_EQ(uint32_vec.size(), 1UL);

    uint32_t result =
        static_cast<uint32_t>(composer.get_variable(uint32_vec[0].get_witness_index()).from_montgomery_form().data[0]);

    uint32_t expected = 0b01101;
    EXPECT_EQ(result, expected);
}

TEST(stdlib_bit_array, test_string_input_output_consistency)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    std::string expected = "string literals inside a SNARK circuit? What nonsense!";
    bit_array test_bit_array = bit_array(&composer, expected);

    std::string result = test_bit_array.get_witness_as_string();

    EXPECT_EQ(result, expected);
}

TEST(stdlib_bit_array, test_byte_array_conversion)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    std::string expected = "string literals inside a SNARK circuit? What nonsense!";
    bit_array test_bit_array = bit_array(&composer, expected);

    byte_array test_bytes(test_bit_array);
    bit_array test_output(test_bytes);
    std::string result = test_output.get_witness_as_string();

    EXPECT_EQ(result, expected);
}

TEST(stdlib_bit_array, test_uint32_vector_constructor)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    uint32_t a_expected = engine.get_random_uint32();
    uint32_t b_expected = engine.get_random_uint32();

    uint32 a = witness_t(&composer, a_expected);
    uint32 b = witness_t(&composer, b_expected);

    std::vector<uint32> inputs = { a, b };
    bit_array test_bit_array = bit_array(inputs);

    std::vector<uint32> result = test_bit_array.to_uint32_vector();

    bit_array test_bit_array_2 = bit_array(result);

    static_cast<byte_array>(test_bit_array_2).get_value();
}
} // namespace test_stdlib_bit_array
