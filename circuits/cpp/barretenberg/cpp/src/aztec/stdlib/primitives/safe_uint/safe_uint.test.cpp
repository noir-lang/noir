
#include <cstddef>
#include <gtest/gtest.h>
#include "safe_uint.hpp"
#include <numeric/random/engine.hpp>
#include "../byte_array/byte_array.hpp"
#include <honk/composer/standard_honk_composer.hpp>
// #include <plonk/composer/standard_composer.hpp>
// #include <plonk/composer/ultra_composer.hpp>
// #include <plonk/composer/turbo_composer.hpp>
#include <stdlib/types/types.hpp>

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using namespace plonk::stdlib::types;

namespace test_stdlib_safe_uint {
template <class T> void ignore_unused(T&) {} // use to ignore unused variables in lambdas

using Composer = honk::StandardHonkComposer;
typedef plonk::stdlib::bool_t<Composer> bool_t;
typedef plonk::stdlib::field_t<Composer> field_t;
typedef plonk::stdlib::safe_uint_t<Composer> suint_t;
typedef plonk::stdlib::witness_t<Composer> witness_t;
typedef plonk::stdlib::public_witness_t<Composer> public_witness_t;
typedef plonk::stdlib::byte_array<Composer> byte_array_t;

struct verify_logic_result {
    bool valid;
    std::string err;
};

verify_logic_result verify_logic(Composer& composer)
{
    if (composer.failed()) {
        info("Circuit logic failed: " + composer.err());
    }
    return { !composer.failed(), composer.err() };
}

// CONSTRUCTOR

TEST(stdlib_safeuint, test_constructor_with_value_out_of_range_fails)
{
    // check incorrect range init causes failure

    Composer composer = Composer();
    field_t a(witness_t(&composer, 100));
    suint_t b(a, 2, "b");

    auto result = verify_logic(composer);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: b");
}

TEST(stdlib_safeuint, test_constructor_with_value_in_range)
{
    Composer composer = Composer();
    field_t a(witness_t(&composer, 100));
    suint_t b(a, 7);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

// * OPERATOR

#if !defined(__wasm__)
TEST(stdlib_safeuint, test_multiply_operation_out_of_range_fails)
{
    // Since max is initally set to (1 << 2) - 1 = 3 (as bit range checks are easier than generic integer bounds),
    // should allow largest power of 3 smaller than r iterations, which is 159. Hence below we should exceed r, and
    // expect a throw
    try {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 2));
        suint_t c(a, 2);
        suint_t d(a, 2);
        for (auto i = 0; i < 160; i++) {
            c = c * d;
        }
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_EQ(err.what(), std::string("exceeded modulus in safe_uint class"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}

TEST(stdlib_safeuint, test_multiply_operation_on_constants_out_of_range_fails)
{
    //  Now we check that when using constants the maximum grows more slowly - since they are bounded by themselves
    //  rather than the next 2^n-1
    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    suint_t c(a, 2);
    suint_t d(fr(2));

    for (auto i = 0; i < 252; i++) {
        c = c * d;
    }
    // Below we should exceed r, and expect a throw

    try {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 2));
        suint_t c(a, 2);
        suint_t d(fr(2));
        for (auto i = 0; i < 253; i++) {
            c = c * d;
        }
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_EQ(err.what(), std::string("exceeded modulus in safe_uint class"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
// + OPERATOR

TEST(stdlib_safeuint, test_add_operation_out_of_range_fails)
{
    // Here we test the addition operator also causes a throw when exceeding r
    try {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 2));
        suint_t c(a, 2);
        suint_t d(a, 2);
        for (auto i = 0; i < 159; i++) {
            c = c * d;
        }
        c = c + c + c;
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_EQ(err.what(), std::string("exceeded modulus in safe_uint class"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
#endif
// SUBTRACT METHOD

TEST(stdlib_safeuint, test_subtract_method)
{

    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 9));
    suint_t c(a, 2);
    suint_t d(b, 4);
    c = d.subtract(c, 3);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

TEST(stdlib_safeuint, test_subtract_method_minued_gt_lhs_fails)
{
    // test failure when range for difference too small
    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 9));
    suint_t c(a, 2, "c");
    suint_t d(b, 4, "d");
    c = d.subtract(c, 2, "d - c"); // we can't be sure that 4-bits minus 2-bits is 2-bits.

    auto result = verify_logic(composer);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: d - c");
}

#if !defined(__wasm__)
TEST(stdlib_safeuint, test_subtract_method_underflow_fails)
{

    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, field_t::modulus / 2));
    suint_t c(a, 2);
    suint_t d(b, suint_t::MAX_BIT_NUM);
    try {
        c = c.subtract(d, suint_t::MAX_BIT_NUM);
        FAIL() << "Expected out of range error";
    } catch (std::runtime_error const& err) {
        EXPECT_EQ(err.what(), std::string("maximum value exceeded in safe_uint subtract"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
#endif
// - OPERATOR

TEST(stdlib_safeuint, test_minus_operator)
{
    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 9));
    suint_t c(a, 2);
    suint_t d(b, 4);
    c = d - c;

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

#if !defined(__wasm__)
TEST(stdlib_safeuint, test_minus_operator_underflow_fails)
{

    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, field_t::modulus / 2));
    suint_t c(a, 2);
    suint_t d(b, suint_t::MAX_BIT_NUM);
    try {
        c = c - d;
    } catch (std::runtime_error const& err) {
        EXPECT_EQ(err.what(), std::string("maximum value exceeded in safe_uint minus operator"));
    } catch (...) {
        FAIL() << "Expected std::runtime_error modulus in safe_uint class";
    }
}
#endif

// DIVIDE METHOD

TEST(stdlib_safeuint, test_divide_method)
{

    Composer composer = Composer();

    field_t a1(witness_t(&composer, 2));
    field_t b1(witness_t(&composer, 9));
    suint_t c1(a1, 2);
    suint_t d1(b1, 4);
    c1 = d1.divide(c1, 3, 1);

    field_t a2(witness_t(&composer, engine.get_random_uint8()));
    field_t b2(witness_t(&composer, engine.get_random_uint32()));
    suint_t c2(a2, 8);
    suint_t d2(b2, 32);
    c2 = d2.divide(c2, 32, 8);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

TEST(stdlib_safeuint, test_divide_method_quotient_range_too_small_fails)
{
    Composer composer = Composer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 32));
    suint_t c(a, 2);
    suint_t d(b, 6);
    d = d.divide(c, 4, 1, "d/c");

    auto result = verify_logic(composer);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: divide method quotient: d/c");
}

#if !defined(__wasm__)
TEST(stdlib_safeuint, test_divide_remainder_too_large)
{
    // test failure when range for remainder too small
    Composer composer = Composer();
    field_t a(witness_t(&composer, 5));
    suint_t c(a, 3);
    suint_t d((fr::modulus - 1) / 3);
    suint_t b;
    EXPECT_ANY_THROW(b = c / d);
}
#endif

TEST(stdlib_safeuint, test_divide_method_quotient_remainder_incorrect_fails)
{
    // test failure when quotient and remainder values are wrong
    Composer composer = Composer();
    field_t a(witness_t(&composer, 5));
    field_t b(witness_t(&composer, 19));
    suint_t c(a, 3);
    suint_t d(b, 5);
    d = d.divide(c, 3, 2, "d/c", [](uint256_t, uint256_t) { return std::make_pair(2, 3); });

    auto result = verify_logic(composer);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "divide method quotient and/or remainder incorrect");
}

TEST(stdlib_safeuint, test_divide_method_quotient_remainder_mod_r_fails)
{
    // test failure when quotient and remainder are only correct mod r
    Composer composer = Composer();
    field_t a(witness_t(&composer, 5));
    field_t b(witness_t(&composer, 19));
    suint_t c(a, 3);
    suint_t d(b, 5);
    d = d.divide(c, 3, 1, "d/c", [](uint256_t a, uint256_t b) { return std::make_pair((fr)a / (fr)b, 0); });
    // 19 / 5 in the field is 0x1d08fbde871dc67f6e96903a4db401d17e858b5eaf6f438a5bedf9bf2999999e, so the quotient
    // should fail the range check of 3-bits.

    auto result = verify_logic(composer);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: divide method quotient: d/c");
}

TEST(stdlib_safeuint, test_div_operator)
{
    Composer composer = Composer();

    suint_t a(witness_t(&composer, 1000), 10, "a");
    suint_t b(2, 2, "b");

    a = a / b;

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    EXPECT_TRUE(verifier.verify_proof(proof));
}

// / OPERATOR

TEST(stdlib_safeuint, test_divide_operator)
{
    // test success cases
    {
        Composer composer = Composer();

        field_t a1(witness_t(&composer, 2));
        field_t b1(witness_t(&composer, 9));
        suint_t c1(a1, 2);
        suint_t d1(b1, 4);
        d1 / c1;

        field_t a2(witness_t(&composer, engine.get_random_uint8()));
        field_t b2(witness_t(&composer, engine.get_random_uint32()));
        suint_t c2(a2, 8);
        suint_t d2(b2, 32);
        d2 / c2;

        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    // test failure when range for quotient too small
    {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 2));
        field_t b(witness_t(&composer, 32));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d / c;
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // test failure when range for remainder too small
    {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 5));
        field_t b(witness_t(&composer, 19));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d / c;
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // test failure when quotient and remainder values are wrong
    {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 5));
        field_t b(witness_t(&composer, 19));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d / c;
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // test failure when quotient and remainder are only correct mod r
    {
        Composer composer = Composer();
        field_t a(witness_t(&composer, 5));
        field_t b(witness_t(&composer, 19));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d / c;
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
}

// SLICE

TEST(stdlib_safeuint, test_slice)
{
    Composer composer = Composer();
    // 0b11110110101001011
    //         ^      ^
    //        msb    lsb
    //        10      3
    // hi=0x111101, lo=0x011, slice=0x10101001
    //
    suint_t a(witness_t(&composer, fr(126283)), 17);
    auto slice_data = a.slice(10, 3);

    EXPECT_EQ(slice_data[0].get_value(), fr(3));
    EXPECT_EQ(slice_data[1].get_value(), fr(169));
    EXPECT_EQ(slice_data[2].get_value(), fr(61));

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_TRUE(result);
}

TEST(stdlib_safeuint, test_slice_equal_msb_lsb)
{
    Composer composer = Composer();
    // 0b11110110101001011
    //             ^
    //         msb = lsb
    //             6
    // hi=0b1111011010, lo=0b001011, slice=0b1
    //
    suint_t a(witness_t(&composer, fr(126283)), 17);
    auto slice_data = a.slice(6, 6);

    EXPECT_EQ(slice_data[0].get_value(), fr(11));
    EXPECT_EQ(slice_data[1].get_value(), fr(1));
    EXPECT_EQ(slice_data[2].get_value(), fr(986));

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_TRUE(result);
}

TEST(stdlib_safeuint, test_slice_random)
{
    Composer composer = Composer();

    uint8_t lsb = 106;
    uint8_t msb = 189;
    fr a_ = fr(uint256_t(fr::random_element()) && ((uint256_t(1) << 252) - 1));
    suint_t a(witness_t(&composer, a_), 252);
    auto slice = a.slice(msb, lsb);

    const uint256_t expected0 = uint256_t(a_) & ((uint256_t(1) << uint64_t(lsb)) - 1);
    const uint256_t expected1 = (uint256_t(a_) >> lsb) & ((uint256_t(1) << (uint64_t(msb - lsb) + 1)) - 1);
    const uint256_t expected2 = (uint256_t(a_) >> (msb + 1)) & ((uint256_t(1) << (uint64_t(252 - msb) - 1)) - 1);

    EXPECT_EQ(slice[0].get_value(), fr(expected0));
    EXPECT_EQ(slice[1].get_value(), fr(expected1));
    EXPECT_EQ(slice[2].get_value(), fr(expected2));

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_TRUE(result);
}

/**
 * @brief Make sure we prevent proving v / v = 0 by setting the divison remainder to be v.
 */

TEST(stdlib_safeuint, operator_div_remainder_constraint)
{
    Composer composer = Composer();

    uint256_t val = 5;

    suint_t a(witness_t(&composer, val), 32);
    suint_t b(witness_t(&composer, val), 32);

    uint256_t quotient_val = 0;
    uint256_t remainder_val = val;
    field_t quotient_field(witness_t(&composer, quotient_val));
    field_t remainder_field(witness_t(&composer, remainder_val));
    suint_t quotient(quotient_field, (size_t)(a.current_max.get_msb() + 1));
    suint_t remainder(remainder_field, (size_t)(a.current_max.get_msb() + 1));
    // This line implicitly checks we are not overflowing
    suint_t int_val = quotient * b + remainder;

    // Idiomatic constraint
    // We constrain divisor - remainder - 1 to be positive to ensure that remainder < divisor.
    suint_t delta = b - remainder - 1;
    field_t::from_witness_index(delta.value.context, delta.value.get_witness_index())
        .create_range_constraint(static_cast<size_t>(b.current_max.get_msb() + 1));

    // // More rudimentary constraint
    // // We constrain divisor - remainder - 1 to be positive to ensure that remainder < divisor.
    // const uint256_t delta = b.get_value() - remainder_val - 1;
    // const uint32_t delta_idx = composer.add_variable(delta);

    // // constraint: other - remainder - delta - 1 == 0
    //         const add_triple delta_gate{ .a = b.get_witness_index(),
    //                                      .b = remainder.get_witness_index(),
    //                                      .c = delta_idx,
    //                                      .a_scaling = 1,
    //                                      .b_scaling = -1,
    //                                      .c_scaling = -1,
    //                                      .const_scaling = -1 };

    // composer.create_add_gate(delta_gate);

    // // validate delta is in the correct range
    // field_t::from_witness_index(&composer, delta_idx).create_range_constraint(b.current_max.get_msb() + 1);

    a.assert_equal(int_val);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, false);
}

/**
 * @brief Make sure we prevent proving v / v = 0 with remainder set to v.
 */

TEST(stdlib_safeuint, div_remainder_constraint)
{
    Composer composer = Composer();

    uint256_t val = 5;

    suint_t a(witness_t(&composer, val), 32);
    suint_t b(witness_t(&composer, val), 32);

    // set quotient to 0 and remainder to val.
    auto supply_bad_witnesses = [](uint256_t val, uint256_t divisor) {
        ignore_unused(divisor);
        return std::make_pair(0, val);
    };

    a.divide(b, 32, 32, "", supply_bad_witnesses);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, false);
}

TEST(stdlib_safeuint, test_byte_array_conversion)
{
    Composer composer = Composer();
    field_t elt = witness_t(&composer, 0x7f6f5f4f00010203);
    suint_t safe(elt, 63);
    // safe.value is a uint256_t, so we serialize to a 32-byte array
    std::string expected = { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                             0x00, 0x00, 0x7f, 0x6f, 0x5f, 0x4f, 0x00, 0x01, 0x02, 0x03 };

    byte_array_t arr(&composer);
    arr.write(static_cast<byte_array_t>(safe));
    EXPECT_EQ(arr.get_string(), expected);
}
} // namespace test_stdlib_safe_uint