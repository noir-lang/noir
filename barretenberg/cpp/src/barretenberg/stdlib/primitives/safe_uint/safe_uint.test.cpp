
#include "safe_uint.hpp"
#include "../byte_array/byte_array.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include <cstddef>
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-local-typedefs"

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Builder = TypeParam;                                                                                         \
    using witness_ct = stdlib::witness_t<Builder>;                                                                     \
    using field_ct = stdlib::field_t<Builder>;                                                                         \
    using bool_ct = stdlib::bool_t<Builder>;                                                                           \
    using suint_ct = stdlib::safe_uint_t<Builder>;                                                                     \
    using byte_array_ct = stdlib::byte_array<Builder>;                                                                 \
    using public_witness_ct = stdlib::public_witness_t<Builder>;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace test_stdlib_safe_uint {
using namespace bb;
using namespace bb::plonk;

template <class T> void ignore_unused(T&) {} // use to ignore unused variables in lambdas

template <class Builder> class SafeUintTest : public ::testing::Test {};

using CircuitTypes = ::testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder>;
TYPED_TEST_SUITE(SafeUintTest, CircuitTypes);

// CONSTRUCTOR

TYPED_TEST(SafeUintTest, TestConstructorWithValueOutOfRangeFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    // check incorrect range init causes failure

    field_ct a(witness_ct(&builder, 100));
    suint_ct b(a, 2, "b");

    EXPECT_FALSE(builder.check_circuit());
}

TYPED_TEST(SafeUintTest, TestConstructorWithValueInRange)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 100));
    suint_ct b(a, 7);

    EXPECT_TRUE(builder.check_circuit());
}

// * OPERATOR

/**
 * @brief Test that we overflow correctly on the border of 3**160 and 3**161.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestMultiplyOperationOutOfRangeFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // Since max is initally set to (1 << 2) - 1 = 3 (as bit range checks are easier than generic integer bounds),
    // should allow largest power of 3 smaller than r iterations, which is 159. Hence below we should exceed r, and
    // expect a throw
    field_ct a(witness_ct(&builder, 2));
    suint_ct c(a, 2);
    suint_ct d(a, 2);
    // should not fail on 159 iterations, since 3**160 < r < 3**161
    for (auto i = 0; i < 159; i++) {
        c = c * d;
    }
    EXPECT_TRUE(builder.check_circuit());
    try {
        // should throw an overflow error on the 160th iteration
        c = c * d;
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_TRUE(builder.check_circuit()); // no failing constraints should be created from multiply
        EXPECT_EQ(err.what(), std::string("exceeded modulus in safe_uint class"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}

/**
 * @brief Test that we correctly overflow multiplying by a constant on the border of 2**253 and 2**254.
 */
TYPED_TEST(SafeUintTest, TestMultiplyOperationOnConstantsOutOfRangeFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    //  Now we check that when using constants the maximum grows more slowly - since they are bounded by themselves
    //  rather than the next 2^n-1

    field_ct a(witness_ct(&builder, 2));
    suint_ct c(a, 2);
    suint_ct d(fr(2));

    // should not fail on 252 iterations
    for (auto i = 0; i < 252; i++) {
        c = c * d;
    }
    EXPECT_TRUE(builder.check_circuit());
    // Below we should exceed r, and expect a throw

    try {
        // should fail on the 253rd iteration
        c = c * d;
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_TRUE(builder.check_circuit()); // no failing constraint from multiply
        EXPECT_EQ(err.what(), std::string("exceeded modulus in safe_uint class"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
// + OPERATOR

/**
 * @brief Test that we correctly overflow on addition on the border of 3**160 and 2 * 3**160.
 */
TYPED_TEST(SafeUintTest, TestAddOperationOutOfRangeFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // Here we test the addition operator also causes a throw when exceeding r
    field_ct a(witness_ct(&builder, 2));
    suint_ct c(a, 2);
    suint_ct d(a, 2);
    // should not fail on the initial setup
    for (auto i = 0; i < 159; i++) {
        c = c * d;
    }
    EXPECT_TRUE(builder.check_circuit());
    try {
        // should fail when we add and exceed the modulus
        c = c + c;
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_TRUE(builder.check_circuit()); // no failing constraints from add or multiply
        EXPECT_EQ(err.what(), std::string("exceeded modulus in safe_uint class"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
#endif

// SUBTRACT METHOD

/**
 * @brief Test that we can subtract without underflow successfully.
 */
TYPED_TEST(SafeUintTest, TestSubtract)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, 9));
    suint_ct c(a, 2);
    suint_ct d(b, 4);
    c = d.subtract(c, 3); // result is 7, which fits in 3 bits and does not fail the range constraint

    EXPECT_TRUE(builder.check_circuit());
}

/**
 * @brief Test that range constraint fails if the value exceeds the bit limit.
 * @details difference is 7, which exceeds 2 bits, and causes the circuit to fail.
 */
TYPED_TEST(SafeUintTest, TestSubtractResultOutOfRange)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // test failure when range for difference too small

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, 9));
    suint_ct c(a, 2, "c");
    suint_ct d(b, 4, "d");
    c = d.subtract(c, 2, "d - c"); // we can't be sure that 4-bits minus 2-bits is 2-bits.

    EXPECT_FALSE(builder.check_circuit());
}

/**
 * @brief Test that underflow is caught in general case.
 * @details General case refers to when difference.current_max + other.current_max does not exceed MAX_VALUE
 *          and underflow is caught by range constraint.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestSubtractUnderflowGeneral)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 0));
    field_ct b(witness_ct(&builder, 1));
    suint_ct c(a, 0);
    suint_ct d(b, 1);
    c = c.subtract(d, suint_ct::MAX_BIT_NUM);
    EXPECT_FALSE(builder.check_circuit());
}
#endif

/**
 * @brief Test that underflow is caught in the special case.
 * @details Should throw an error because difference.current_max + other.current_max exceeds the MAX_VALUE.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestSubtractUnderflowSpecial)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, field_ct::modulus / 2));
    suint_ct c(a, 2);
    suint_ct d(b, suint_ct::MAX_BIT_NUM);
    try {
        c = c.subtract(d, suint_ct::MAX_BIT_NUM);
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_TRUE(builder.check_circuit());
        EXPECT_EQ(err.what(), std::string("maximum value exceeded in safe_uint subtract"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
#endif

// - OPERATOR

/**
 * @brief Test that valid minus operation works.
 */
TYPED_TEST(SafeUintTest, TestMinusOperator)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 9));
    field_ct b(witness_ct(&builder, 2));
    suint_ct c(a, 4);
    suint_ct d(b, 2);
    c = c - d; // 9 - 2 = 7 should not underflow

    EXPECT_TRUE(builder.check_circuit());
}

/**
 * @brief Test that valid minus operation works on 0.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestMinusOperatorValidOnZero)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, 2));
    suint_ct c(a, 2);
    suint_ct d(b, 3);
    c = c - d; // 2 - 2 = 0 should not overflow, even if d has more bits than c.
    EXPECT_TRUE(builder.check_circuit());
}
#endif

/**
 * @brief Test that checks that minus operator underflow is caught in the general case.
 * @details General case means that the special case does not happen.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestMinusUnderflowGeneral1)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, field_ct::modulus / 2));
    suint_ct c(a, 2);
    suint_ct d(b, suint_ct::MAX_BIT_NUM);
    c = c - d; // generates range constraint that the difference is in [0, 3], which it is not with these witness values
    EXPECT_FALSE(builder.check_circuit());
}
#endif

/**
 * @brief Test that checks that minus operator underflow is caught in the general case.
 * @details Testing -1 is an underflow.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestMinusUnderflowGeneral2)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, 3));
    suint_ct c(a, 2);
    suint_ct d(b, 3);
    c = c - d;
    EXPECT_FALSE(builder.check_circuit()); // underflow should cause range constraint to fail
}
#endif

/**
 * @brief Test that checks that minus operator underflow is caught from special case.
 * @details Special case refers to the check that current_max + other.current_max > MAX_VALUE, which is a potential
 * underflow case, that escapes the general check through the range constraint. Throws an error even if it is not an
 * underflow in some instantiations of the witness values.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestMinusUnderflowSpecial1)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 1));
    field_ct b(witness_ct(&builder, 0));
    suint_ct c(a, suint_ct::MAX_BIT_NUM);
    suint_ct d(b, suint_ct::MAX_BIT_NUM);
    try {
        c = c - d; // even though this is not an underflow, we cannot distinguish it from an actual underflow because
                   // the sum of maxes exceeds MAX_VALUE so we must throw an error
        FAIL() << "Expected error to be thrown";
    } catch (std::runtime_error const& err) {
        EXPECT_TRUE(builder.check_circuit()); // no incorrect constraints
        EXPECT_EQ(err.what(),
                  std::string("maximum value exceeded in safe_uint minus operator")); // possible underflow is detected
                                                                                      // with check on maxes
    } catch (...) {
        FAIL() << "Expected no error, got other error";
    }
}
#endif

/**
 * @brief Test that checks that minus operator underflow is caught from special case.
 * @details Special case refers to the check that current_max + other.current_max > MAX_VALUE, which is a potential
 * underflow case, that escapes the general check through the range constraint. Also, underflow can actually be detected
 * from range constraint.
 */
#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestMinusUnderflowSpecial2)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 0));
    field_ct b(witness_ct(&builder, 1));
    suint_ct c(a, suint_ct::MAX_BIT_NUM);
    suint_ct d(b, suint_ct::MAX_BIT_NUM);
    try {
        c = c - d; // underflow and error should be thrown
        FAIL() << "Expected error to be thrown";
    } catch (std::runtime_error const& err) {
        EXPECT_FALSE(builder.check_circuit()); // underflow causes failing constraint
        EXPECT_EQ(err.what(),
                  std::string("maximum value exceeded in safe_uint minus operator")); // possible underflow is detected
                                                                                      // with check on maxes
    } catch (...) {
        FAIL() << "Expected no error, got other error";
    }
}
#endif

// DIVIDE METHOD

TYPED_TEST(SafeUintTest, TestDivideMethod)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a1(witness_ct(&builder, 2));
    field_ct b1(witness_ct(&builder, 9));
    suint_ct c1(a1, 2);
    suint_ct d1(b1, 4);
    c1 = d1.divide(c1, 3, 1);

    field_ct a2(witness_ct(&builder, engine.get_random_uint8()));
    field_ct b2(witness_ct(&builder, engine.get_random_uint32()));
    suint_ct c2(a2, 8);
    suint_ct d2(b2, 32);
    c2 = d2.divide(c2, 32, 8);

    EXPECT_TRUE(builder.check_circuit());
}

TYPED_TEST(SafeUintTest, TestDivideMethodQuotientRangeTooSmallFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct a(witness_ct(&builder, 2));
    field_ct b(witness_ct(&builder, 32));
    suint_ct c(a, 2);
    suint_ct d(b, 6);
    d = d.divide(c, 4, 1, "d/c");

    EXPECT_FALSE(builder.check_circuit());
}

#if !defined(__wasm__)
TYPED_TEST(SafeUintTest, TestDivideRemainderTooLarge)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // test failure when range for remainder too small

    field_ct a(witness_ct(&builder, 5));
    suint_ct c(a, 3);
    suint_ct d((fr::modulus - 1) / 3);
    suint_ct b;
    EXPECT_ANY_THROW(b = c / d);
}
#endif

TYPED_TEST(SafeUintTest, TestDivideMethodQuotientRemainderIncorrectFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // test failure when quotient and remainder values are wrong

    field_ct a(witness_ct(&builder, 5));
    field_ct b(witness_ct(&builder, 19));
    suint_ct c(a, 3);
    suint_ct d(b, 5);
    d = d.divide(c, 3, 2, "d/c", [](uint256_t, uint256_t) { return std::make_pair(2, 3); });

    EXPECT_FALSE(builder.check_circuit());
}

TYPED_TEST(SafeUintTest, TestDivideMethodQuotientRemainderModRFails)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // test failure when quotient and remainder are only correct mod r

    field_ct a(witness_ct(&builder, 5));
    field_ct b(witness_ct(&builder, 19));
    suint_ct c(a, 3);
    suint_ct d(b, 5);
    d = d.divide(c, 3, 1, "d/c", [](uint256_t a, uint256_t b) { return std::make_pair((fr)a / (fr)b, 0); });
    // 19 / 5 in the field is 0x1d08fbde871dc67f6e96903a4db401d17e858b5eaf6f438a5bedf9bf2999999e, so the quotient
    // should fail the range check of 3-bits.

    EXPECT_FALSE(builder.check_circuit());
}

TYPED_TEST(SafeUintTest, TestDivOperator)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    suint_ct a(witness_ct(&builder, 1000), 10, "a");
    suint_ct b(2, 2, "b");

    a = a / b;

    EXPECT_TRUE(builder.check_circuit());
}

// / OPERATOR

TYPED_TEST(SafeUintTest, TestDivideOperator)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();
    // test success cases
    {
        auto builder = Builder();
        field_ct a1(witness_ct(&builder, 2));
        field_ct b1(witness_ct(&builder, 9));
        suint_ct c1(a1, 2);
        suint_ct d1(b1, 4);
        d1 / c1;

        field_ct a2(witness_ct(&builder, engine.get_random_uint8()));
        field_ct b2(witness_ct(&builder, engine.get_random_uint32()));
        suint_ct c2(a2, 8);
        suint_ct d2(b2, 32);
        d2 / c2;

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);
    }
    // test failure when range for quotient too small
    {
        auto builder = Builder();
        field_ct a(witness_ct(&builder, 2));
        field_ct b(witness_ct(&builder, 32));
        suint_ct c(a, 2);
        suint_ct d(b, 5);
        d = d / c;
        bool result = builder.check_circuit();
        EXPECT_EQ(result, false);
    }
    // test failure when range for remainder too small
    {

        field_ct a(witness_ct(&builder, 5));
        field_ct b(witness_ct(&builder, 19));
        suint_ct c(a, 2);
        suint_ct d(b, 5);
        d = d / c;
        bool result = builder.check_circuit();
        EXPECT_EQ(result, false);
    }
    // test failure when quotient and remainder values are wrong
    {
        auto builder = Builder();
        field_ct a(witness_ct(&builder, 5));
        field_ct b(witness_ct(&builder, 19));
        suint_ct c(a, 2);
        suint_ct d(b, 5);
        d = d / c;
        bool result = builder.check_circuit();
        EXPECT_EQ(result, false);
    }
    // test failure when quotient and remainder are only correct mod r
    {
        auto builder = Builder();
        field_ct a(witness_ct(&builder, 5));
        field_ct b(witness_ct(&builder, 19));
        suint_ct c(a, 2);
        suint_ct d(b, 5);
        d = d / c;
        bool result = builder.check_circuit();
        EXPECT_EQ(result, false);
    }
}

// SLICE

TYPED_TEST(SafeUintTest, TestSlice)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    // 0b11110110101001011
    //         ^      ^
    //        msb    lsb
    //        10      3
    // hi=0x111101, lo=0x011, slice=0x10101001
    //
    suint_ct a(witness_ct(&builder, fr(126283)), 17);
    auto slice_data = a.slice(10, 3);

    EXPECT_EQ(slice_data[0].get_value(), fr(3));
    EXPECT_EQ(slice_data[1].get_value(), fr(169));
    EXPECT_EQ(slice_data[2].get_value(), fr(61));

    bool result = builder.check_circuit();
    EXPECT_TRUE(result);
}

TYPED_TEST(SafeUintTest, TestSliceEqualMsbLsb)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    // 0b11110110101001011
    //             ^
    //         msb = lsb
    //             6
    // hi=0b1111011010, lo=0b001011, slice=0b1
    //
    suint_ct a(witness_ct(&builder, fr(126283)), 17);
    auto slice_data = a.slice(6, 6);

    EXPECT_EQ(slice_data[0].get_value(), fr(11));
    EXPECT_EQ(slice_data[1].get_value(), fr(1));
    EXPECT_EQ(slice_data[2].get_value(), fr(986));

    bool result = builder.check_circuit();
    EXPECT_TRUE(result);
}

TYPED_TEST(SafeUintTest, TestSliceRandom)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    uint8_t lsb = 106;
    uint8_t msb = 189;
    fr a_ = fr(uint256_t(fr::random_element()) && ((uint256_t(1) << 252) - 1));
    suint_ct a(witness_ct(&builder, a_), 252);
    auto slice = a.slice(msb, lsb);

    const uint256_t expected0 = uint256_t(a_) & ((uint256_t(1) << uint64_t(lsb)) - 1);
    const uint256_t expected1 = (uint256_t(a_) >> lsb) & ((uint256_t(1) << (uint64_t(msb - lsb) + 1)) - 1);
    const uint256_t expected2 =
        (uint256_t(a_) >> uint64_t(msb + 1)) & ((uint256_t(1) << (uint64_t(252 - msb) - 1)) - 1);

    EXPECT_EQ(slice[0].get_value(), fr(expected0));
    EXPECT_EQ(slice[1].get_value(), fr(expected1));
    EXPECT_EQ(slice[2].get_value(), fr(expected2));

    bool result = builder.check_circuit();
    EXPECT_TRUE(result);
}

/**
 * @brief Make sure we prevent proving v / v = 0 by setting the divison remainder to be v.
 */

TYPED_TEST(SafeUintTest, TestOperatorDivRemainderConstraint)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    uint256_t val = 5;

    suint_ct a(witness_ct(&builder, val), 32);
    suint_ct b(witness_ct(&builder, val), 32);

    uint256_t quotient_val = 0;
    uint256_t remainder_val = val;
    field_ct quotient_field(witness_ct(&builder, quotient_val));
    field_ct remainder_field(witness_ct(&builder, remainder_val));
    suint_ct quotient(quotient_field, (size_t)(a.current_max.get_msb() + 1));
    suint_ct remainder(remainder_field, (size_t)(a.current_max.get_msb() + 1));
    // This line implicitly checks we are not overflowing
    suint_ct int_val = quotient * b + remainder;

    // Idiomatic constraint
    // We constrain divisor - remainder - 1 to be positive to ensure that remainder < divisor.
    suint_ct delta = b - remainder - 1;
    field_ct::from_witness_index(delta.value.context, delta.value.get_witness_index())
        .create_range_constraint(static_cast<size_t>(b.current_max.get_msb() + 1));

    // // More rudimentary constraint
    // // We constrain divisor - remainder - 1 to be positive to ensure that remainder < divisor.
    // const uint256_t delta = b.get_value() - remainder_val - 1;
    // const uint32_t delta_idx = builder.add_variable(delta);

    // // constraint: other - remainder - delta - 1 == 0
    //         const add_triple delta_gate{ .a = b.get_witness_index(),
    //                                      .b = remainder.get_witness_index(),
    //                                      .c = delta_idx,
    //                                      .a_scaling = 1,
    //                                      .b_scaling = -1,
    //                                      .c_scaling = -1,
    //                                      .const_scaling = -1 };

    // builder.create_add_gate(delta_gate);

    // // validate delta is in the correct range
    // field_ct::from_witness_index(&builder, delta_idx).create_range_constraint(b.current_max.get_msb() + 1);

    a.assert_equal(int_val);

    bool result = builder.check_circuit();
    EXPECT_EQ(result, false);
}

/**
 * @brief Make sure we prevent proving v / v = 0 with remainder set to v.
 */

TYPED_TEST(SafeUintTest, TestDivRemainderConstraint)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    uint256_t val = 5;

    suint_ct a(witness_ct(&builder, val), 32);
    suint_ct b(witness_ct(&builder, val), 32);

    // set quotient to 0 and remainder to val.
    auto supply_bad_witnesses = [](uint256_t val, uint256_t divisor) {
        ignore_unused(divisor);
        return std::make_pair(0, val);
    };

    a.divide(b, 32, 32, "", supply_bad_witnesses);

    bool result = builder.check_circuit();
    EXPECT_EQ(result, false);
}

TYPED_TEST(SafeUintTest, TestByteArrayConversion)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    field_ct elt = witness_ct(&builder, 0x7f6f5f4f00010203);
    suint_ct safe(elt, 63);
    // safe.value is a uint256_t, so we serialize to a 32-byte array
    std::string expected = { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x7f, 0x6f, 0x5f, 0x4f, 0x00, 0x01, 0x02, 0x03 };

    byte_array_ct arr(&builder);
    arr.write(static_cast<byte_array_ct>(safe));
    EXPECT_EQ(arr.get_string(), expected);
}
} // namespace test_stdlib_safe_uint
