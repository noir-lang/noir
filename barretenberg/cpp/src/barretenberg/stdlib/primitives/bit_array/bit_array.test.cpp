#include "bit_array.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-local-typedefs"

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Builder = TypeParam;                                                                                         \
    using witness_ct = stdlib::witness_t<Builder>;                                                                     \
    using byte_array_ct = stdlib::byte_array<Builder>;                                                                 \
    using field_ct = stdlib::field_t<Builder>;                                                                         \
    using uint32_ct = stdlib::uint32<Builder>;                                                                         \
    using bit_array_ct = stdlib::bit_array<Builder>;                                                                   \
    using bool_ct = stdlib::bool_t<Builder>;

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

template <class Builder> class BitArrayTest : public ::testing::Test {};

using CircuitTypes = ::testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder>;
TYPED_TEST_SUITE(BitArrayTest, CircuitTypes);

TYPED_TEST(BitArrayTest, test_uint32_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    uint32_t a_expected = engine.get_random_uint32();
    uint32_t b_expected = engine.get_random_uint32();

    uint32_ct a = witness_ct(&builder, a_expected);
    uint32_ct b = witness_ct(&builder, b_expected);

    std::vector<uint32_ct> inputs = { a, b };
    bit_array_ct test_bit_array = bit_array_ct(inputs);

    std::vector<uint32_ct> result = test_bit_array.to_uint32_vector();

    EXPECT_EQ(result.size(), 2UL);

    auto a_result =
        static_cast<uint32_t>(builder.get_variable(result[0].get_witness_index()).from_montgomery_form().data[0]);
    auto b_result =
        static_cast<uint32_t>(builder.get_variable(result[1].get_witness_index()).from_montgomery_form().data[0]);

    EXPECT_EQ(a_result, a_expected);
    EXPECT_EQ(b_result, b_expected);
}

TYPED_TEST(BitArrayTest, test_binary_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    bit_array_ct test_bit_array = bit_array_ct(&builder, 5);

    test_bit_array[0] = bool_ct(witness_ct(&builder, true));
    test_bit_array[1] = bool_ct(witness_ct(&builder, false));
    test_bit_array[2] = bool_ct(witness_ct(&builder, true));
    test_bit_array[3] = bool_ct(witness_ct(&builder, true));
    test_bit_array[4] = bool_ct(witness_ct(&builder, false));

    std::vector<uint32_ct> uint32_vec = test_bit_array.to_uint32_vector();

    EXPECT_EQ(uint32_vec.size(), 1UL);

    auto result =
        static_cast<uint32_t>(builder.get_variable(uint32_vec[0].get_witness_index()).from_montgomery_form().data[0]);

    auto expected = 0b01101;
    EXPECT_EQ(result, expected);
}

TYPED_TEST(BitArrayTest, test_string_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    std::string expected = "string literals inside a SNARK circuit? What nonsense!";
    bit_array_ct test_bit_array = bit_array_ct(&builder, expected);

    std::string result = test_bit_array.get_witness_as_string();

    EXPECT_EQ(result, expected);
}

TYPED_TEST(BitArrayTest, test_byte_array_conversion)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    std::string expected = "string literals inside a SNARK circuit? What nonsense!";
    bit_array_ct test_bit_array = bit_array_ct(&builder, expected);

    byte_array_ct test_bytes(test_bit_array);
    bit_array_ct test_output(test_bytes);
    std::string result = test_output.get_witness_as_string();

    EXPECT_EQ(result, expected);
}

TYPED_TEST(BitArrayTest, test_uint32_vector_constructor)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    uint32_t a_expected = engine.get_random_uint32();
    uint32_t b_expected = engine.get_random_uint32();

    uint32_ct a = witness_ct(&builder, a_expected);
    uint32_ct b = witness_ct(&builder, b_expected);

    std::vector<uint32_ct> inputs = { a, b };
    bit_array_ct test_bit_array = bit_array_ct(inputs);

    std::vector<uint32_ct> result = test_bit_array.to_uint32_vector();

    bit_array_ct test_bit_array_2 = bit_array_ct(result);

    static_cast<byte_array_ct>(test_bit_array_2).get_value();
}
