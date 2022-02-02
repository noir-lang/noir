
#include <gtest/gtest.h>
#include "safe_uint.hpp"
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/plookup_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
// using namespace plonk::stdlib;
typedef plonk::stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef plonk::stdlib::field_t<waffle::TurboComposer> field_t;
typedef plonk::stdlib::safe_uint_t<waffle::TurboComposer> suint_t;
typedef plonk::stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef plonk::stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

TEST(stdlib_safeuint, add_and_multiply)
{
    // check incorrect range init in posint class causes failure
    {

        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 100));
        field_t b(witness_t(&composer, 2));
        suint_t c(a, 2);
        suint_t d(b, 2);

        c = c + d;
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    {

        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 100));
        field_t b(witness_t(&composer, 2));
        suint_t c(a, 7);
        suint_t d(b, 2);

        c = c + d;
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
}
TEST(positiveint, checkRangeBounds)
{

    waffle::TurboComposer composer = waffle::TurboComposer();
    field_t a(witness_t(&composer, 2));
    suint_t c(a, 2);
    suint_t d(a, 2);
    // since max is initally set to 3 (as bit range checks are easier than generic integer bounds), should allow largest
    // power of 3 smaller than r iterations, which is 159.
    for (auto i = 0; i < 159; i++) {
        c = c * d;
    }
    // Hence below we should exceed r, and expect a throw
    {
        try {
            waffle::TurboComposer composer = waffle::TurboComposer();
            field_t a(witness_t(&composer, 2));
            suint_t c(a, 2);
            suint_t d(a, 2);
            for (auto i = 0; i < 160; i++) {
                c = c * d;
            }
            FAIL() << "Expected out of range error";
        } catch (std::runtime_error const& err) {
            EXPECT_EQ(err.what(), std::string("exceeded modulus in positive_int class"));
        } catch (...) {
            FAIL() << "Expected std::runtime_error modulus in positive_int class";
        }
    }
    // Here we test the addition operator also causes a throw when exceeding r
    {
        try {
            waffle::TurboComposer composer = waffle::TurboComposer();
            field_t a(witness_t(&composer, 2));
            suint_t c(a, 2);
            suint_t d(a, 2);
            for (auto i = 0; i < 159; i++) {
                c = c * d;
            }
            c = c + c + c;
            FAIL() << "Expected out of range error";
        } catch (std::runtime_error const& err) {
            EXPECT_EQ(err.what(), std::string("exceeded modulus in positive_int class"));
        } catch (...) {
            FAIL() << "Expected std::runtime_error modulus in positive_int class";
        }
    }
    //  Now we check that when using constants the maximum grows more slowly - since they are bounded by themselves
    //  rather than the next 2^n-1
    {
        {
            waffle::TurboComposer composer = waffle::TurboComposer();
            field_t a(witness_t(&composer, 2));
            suint_t c(a, 2);
            suint_t d(fr(2));

            for (auto i = 0; i < 252; i++) {
                c = c * d;
            }
            // Below we should exceed r, and expect a throw
            {
                try {
                    waffle::TurboComposer composer = waffle::TurboComposer();
                    field_t a(witness_t(&composer, 2));
                    suint_t c(a, 2);
                    suint_t d(fr(2));
                    for (auto i = 0; i < 253; i++) {
                        c = c * d;
                    }
                    FAIL() << "Expected out of range error";
                } catch (std::runtime_error const& err) {
                    EXPECT_EQ(err.what(), std::string("exceeded modulus in positive_int class"));
                } catch (...) {
                    FAIL() << "Expected std::runtime_error modulus in positive_int class";
                }
            }
        }
    }
}
TEST(positiveint, subtract)
{

    waffle::TurboComposer composer = waffle::TurboComposer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 9));
    suint_t c(a, 2);
    suint_t d(b, 4);
    c = d.subtract(c, 3);
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
    // test failure when range for difference too small
    {
        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 2));
        field_t b(witness_t(&composer, 9));
        suint_t c(a, 2);
        suint_t d(b, 4);
        c = d.subtract(c, 2);
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    {

        waffle::TurboComposer composer = waffle::TurboComposer();
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
}
TEST(positiveint, minus_operator)
{

    waffle::TurboComposer composer = waffle::TurboComposer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 9));
    suint_t c(a, 2);
    suint_t d(b, 4);
    c = d - c;
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
    //
    {

        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 2));
        field_t b(witness_t(&composer, field_t::modulus / 2));
        suint_t c(a, 2);
        suint_t d(b, suint_t::MAX_BIT_NUM);
        try {
            c = c - d;
        } catch (std::runtime_error const& err) {
            EXPECT_EQ(err.what(), std::string("maximum value exceeded in positive_int minus operator"));
        } catch (...) {
            FAIL() << "Expected std::runtime_error modulus in positive_int class";
        }
    }
}
TEST(positiveint, divide)
{

    waffle::TurboComposer composer = waffle::TurboComposer();
    field_t a(witness_t(&composer, 2));
    field_t b(witness_t(&composer, 9));
    suint_t c(a, 2);
    suint_t d(b, 4);
    c = d.divide(c, 3, 1);
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
    // test failure when range for quotient too small
    {
        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 2));
        field_t b(witness_t(&composer, 32));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d.divide(c, 4, 1);
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // test failure when range for remainder too small
    {
        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 5));
        field_t b(witness_t(&composer, 19));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d.divide(c, 3, 1);
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // test failure when quotient and remainder values are wrong
    {
        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 5));
        field_t b(witness_t(&composer, 19));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d.divide(c, 3, 1, [](uint256_t, uint256_t) { return std::make_pair(2, 8); });
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // test failure when quotient and remainder are only correct mod r
    {
        waffle::TurboComposer composer = waffle::TurboComposer();
        field_t a(witness_t(&composer, 5));
        field_t b(witness_t(&composer, 19));
        suint_t c(a, 2);
        suint_t d(b, 5);
        d = d.divide(c, 3, 1, [](uint256_t a, uint256_t b) { return std::make_pair((fr)a / (fr)b, 0); });
        waffle::TurboProver prover = composer.create_prover();
        waffle::TurboVerifier verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    // {

    //     waffle::TurboComposer composer = waffle::TurboComposer();
    //     field_t a(witness_t(&composer, 2));
    //     field_t b(witness_t(&composer, field_t::modulus / 2));
    //     suint_t c(a, 2);
    //     suint_t d(b, suint_t::MAX_BIT_NUM);
    //     try {
    //         suint_t e = c.divide(d, suint_t::MAX_BIT_NUM, suint_t::MAX_BIT_NUM);
    //         FAIL() << "Expected out of range error";
    //     } catch (std::runtime_error const& err) {
    //         EXPECT_EQ(err.what(), std::string("maximum value exceeded in positive_int subtract"));
    //     } catch (...) {
    //         FAIL() << "Expected std::runtime_error modulus in positive_int class";
    //     }
    // }
}
TEST(stdlib_safeuint, test_slice)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
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

    waffle::TurboProver prover = composer.create_prover();

    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_safeuint, test_slice_equal_msb_lsb)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
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

    waffle::TurboProver prover = composer.create_prover();

    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_safeuint, test_slice_random)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

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

    waffle::TurboProver prover = composer.create_prover();

    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
