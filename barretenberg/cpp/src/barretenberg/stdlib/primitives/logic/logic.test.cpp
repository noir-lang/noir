#include <gtest/gtest.h>

#include "../bool/bool.hpp"
#include "../circuit_builders/circuit_builders.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "logic.hpp"

#pragma GCC diagnostic ignored "-Wunused-local-typedefs"

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Builder = TypeParam;                                                                                         \
    using witness_ct = stdlib::witness_t<Builder>;                                                                     \
    using field_ct = stdlib::field_t<Builder>;                                                                         \
    using bool_ct = stdlib::bool_t<Builder>;                                                                           \
    using public_witness_ct = stdlib::public_witness_t<Builder>;
using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

template <class T> void ignore_unused(T&) {} // use to ignore unused variables in lambdas

template <class Builder> class LogicTest : public testing::Test {};

using CircuitTypes = ::testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder>;

TYPED_TEST_SUITE(LogicTest, CircuitTypes);

TYPED_TEST(LogicTest, TestCorrectLogic)
{
    STDLIB_TYPE_ALIASES

    auto run_test = [](size_t num_bits, Builder& builder) {
        uint256_t mask = (uint256_t(1) << num_bits) - 1;

        uint256_t a = engine.get_random_uint256() & mask;
        uint256_t b = engine.get_random_uint256() & mask;

        uint256_t and_expected = a & b;
        uint256_t xor_expected = a ^ b;

        field_ct x = witness_ct(&builder, a);
        field_ct y = witness_ct(&builder, b);

        field_ct x_const(&builder, a);
        field_ct y_const(&builder, b);

        field_ct and_result = stdlib::logic<Builder>::create_logic_constraint(x, y, num_bits, false);
        field_ct xor_result = stdlib::logic<Builder>::create_logic_constraint(x, y, num_bits, true);

        field_ct and_result_left_constant =
            stdlib::logic<Builder>::create_logic_constraint(x_const, y, num_bits, false);
        field_ct xor_result_left_constant = stdlib::logic<Builder>::create_logic_constraint(x_const, y, num_bits, true);

        field_ct and_result_right_constant =
            stdlib::logic<Builder>::create_logic_constraint(x, y_const, num_bits, false);
        field_ct xor_result_right_constant =
            stdlib::logic<Builder>::create_logic_constraint(x, y_const, num_bits, true);

        field_ct and_result_both_constant =
            stdlib::logic<Builder>::create_logic_constraint(x_const, y_const, num_bits, false);
        field_ct xor_result_both_constant =
            stdlib::logic<Builder>::create_logic_constraint(x_const, y_const, num_bits, true);

        EXPECT_EQ(uint256_t(and_result.get_value()), and_expected);
        EXPECT_EQ(uint256_t(and_result_left_constant.get_value()), and_expected);
        EXPECT_EQ(uint256_t(and_result_right_constant.get_value()), and_expected);
        EXPECT_EQ(uint256_t(and_result_both_constant.get_value()), and_expected);

        EXPECT_EQ(uint256_t(xor_result.get_value()), xor_expected);
        EXPECT_EQ(uint256_t(xor_result_left_constant.get_value()), xor_expected);
        EXPECT_EQ(uint256_t(xor_result_right_constant.get_value()), xor_expected);
        EXPECT_EQ(uint256_t(xor_result_both_constant.get_value()), xor_expected);
    };

    auto builder = Builder();
    for (size_t i = 8; i < 248; i += 8) {
        run_test(i, builder);
    }
    bool result = builder.check_circuit();
    EXPECT_EQ(result, true);
}

// Tests the constraints will fail if the operands are larger than expected even though the result contains the correct
// number of bits when using the UltraPlonkBuilder This is because the range constraints on the right and left operand
// are not being satisfied.
TYPED_TEST(LogicTest, LargeOperands)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    uint256_t mask = (uint256_t(1) << 48) - 1;
    uint256_t a = engine.get_random_uint256() & mask;
    uint256_t b = engine.get_random_uint256() & mask;

    uint256_t expected_mask = (uint256_t(1) << 40) - 1;
    uint256_t and_expected = (a & b) & expected_mask;
    uint256_t xor_expected = (a ^ b) & expected_mask;

    field_ct x = witness_ct(&builder, a);
    field_ct y = witness_ct(&builder, b);

    field_ct xor_result = stdlib::logic<Builder>::create_logic_constraint(x, y, 40, true);
    field_ct and_result = stdlib::logic<Builder>::create_logic_constraint(x, y, 40, false);
    EXPECT_EQ(uint256_t(and_result.get_value()), and_expected);
    EXPECT_EQ(uint256_t(xor_result.get_value()), xor_expected);

    bool result = builder.check_circuit();
    EXPECT_EQ(result, true);
}

// Ensures that malicious witnesses which produce the same result are detected. This potential security issue cannot
// happen if the builder doesn't support lookup gates because constraints will be created for each bit of the left and
// right operand.
TYPED_TEST(LogicTest, DifferentWitnessSameResult)
{

    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    if (HasPlookup<Builder>) {
        uint256_t a = 3758096391;
        uint256_t b = 2147483649;
        field_ct x = witness_ct(&builder, uint256_t(a));
        field_ct y = witness_ct(&builder, uint256_t(b));

        uint256_t xor_expected = a ^ b;
        const std::function<std::pair<uint256_t, uint256_t>(uint256_t, uint256_t, size_t)>& get_bad_chunk =
            [](uint256_t left, uint256_t right, size_t chunk_size) {
                (void)left;
                (void)right;
                (void)chunk_size;
                auto left_chunk = uint256_t(2684354565);
                auto right_chunk = uint256_t(3221225475);
                return std::make_pair(left_chunk, right_chunk);
            };

        field_ct xor_result = stdlib::logic<Builder>::create_logic_constraint(x, y, 32, true, get_bad_chunk);
        EXPECT_EQ(uint256_t(xor_result.get_value()), xor_expected);

        bool result = builder.check_circuit();
        EXPECT_EQ(result, false);
    }
}
