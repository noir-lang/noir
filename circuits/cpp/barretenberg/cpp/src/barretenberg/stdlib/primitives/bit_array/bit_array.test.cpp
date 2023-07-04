#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "bit_array.hpp"
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-local-typedefs"

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Composer = TypeParam;                                                                                        \
    using witness_ct = stdlib::witness_t<Composer>;                                                                    \
    using byte_array_ct = stdlib::byte_array<Composer>;                                                                \
    using field_ct = stdlib::field_t<Composer>;                                                                        \
    using uint32_ct = stdlib::uint32<Composer>;                                                                        \
    using bit_array_ct = stdlib::bit_array<Composer>;                                                                  \
    using bool_ct = stdlib::bool_t<Composer>;

namespace test_stdlib_bit_array {

using namespace barretenberg;
using namespace proof_system::plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <class Composer> class BitArrayTest : public ::testing::Test {};

using CircuitTypes = ::testing::
    Types<proof_system::StandardCircuitBuilder, proof_system::TurboCircuitBuilder, proof_system::UltraCircuitBuilder>;
TYPED_TEST_SUITE(BitArrayTest, CircuitTypes);

TYPED_TEST(BitArrayTest, test_uint32_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    uint32_t a_expected = engine.get_random_uint32();
    uint32_t b_expected = engine.get_random_uint32();

    uint32_ct a = witness_ct(&composer, a_expected);
    uint32_ct b = witness_ct(&composer, b_expected);

    std::vector<uint32_ct> inputs = { a, b };
    bit_array_ct test_bit_array = bit_array_ct(inputs);

    std::vector<uint32_ct> result = test_bit_array.to_uint32_vector();

    EXPECT_EQ(result.size(), 2UL);

    auto a_result =
        static_cast<uint32_t>(composer.get_variable(result[0].get_witness_index()).from_montgomery_form().data[0]);
    auto b_result =
        static_cast<uint32_t>(composer.get_variable(result[1].get_witness_index()).from_montgomery_form().data[0]);

    EXPECT_EQ(a_result, a_expected);
    EXPECT_EQ(b_result, b_expected);
}

TYPED_TEST(BitArrayTest, test_binary_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    bit_array_ct test_bit_array = bit_array_ct(&composer, 5);

    test_bit_array[0] = bool_ct(witness_ct(&composer, true));
    test_bit_array[1] = bool_ct(witness_ct(&composer, false));
    test_bit_array[2] = bool_ct(witness_ct(&composer, true));
    test_bit_array[3] = bool_ct(witness_ct(&composer, true));
    test_bit_array[4] = bool_ct(witness_ct(&composer, false));

    std::vector<uint32_ct> uint32_vec = test_bit_array.to_uint32_vector();

    EXPECT_EQ(uint32_vec.size(), 1UL);

    auto result =
        static_cast<uint32_t>(composer.get_variable(uint32_vec[0].get_witness_index()).from_montgomery_form().data[0]);

    auto expected = 0b01101;
    EXPECT_EQ(result, expected);
}

TYPED_TEST(BitArrayTest, test_string_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    std::string expected = "string literals inside a SNARK circuit? What nonsense!";
    bit_array_ct test_bit_array = bit_array_ct(&composer, expected);

    std::string result = test_bit_array.get_witness_as_string();

    EXPECT_EQ(result, expected);
}

TYPED_TEST(BitArrayTest, test_byte_array_conversion)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    std::string expected = "string literals inside a SNARK circuit? What nonsense!";
    bit_array_ct test_bit_array = bit_array_ct(&composer, expected);

    byte_array_ct test_bytes(test_bit_array);
    bit_array_ct test_output(test_bytes);
    std::string result = test_output.get_witness_as_string();

    EXPECT_EQ(result, expected);
}

TYPED_TEST(BitArrayTest, test_uint32_vector_constructor)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    uint32_t a_expected = engine.get_random_uint32();
    uint32_t b_expected = engine.get_random_uint32();

    uint32_ct a = witness_ct(&composer, a_expected);
    uint32_ct b = witness_ct(&composer, b_expected);

    std::vector<uint32_ct> inputs = { a, b };
    bit_array_ct test_bit_array = bit_array_ct(inputs);

    std::vector<uint32_ct> result = test_bit_array.to_uint32_vector();

    bit_array_ct test_bit_array_2 = bit_array_ct(result);

    static_cast<byte_array_ct>(test_bit_array_2).get_value();
}
} // namespace test_stdlib_bit_array
