#include "byte_array.hpp"
#include <gtest/gtest.h>
#include <plonk/composer/turbo_composer.hpp>

namespace test_stdlib_byte_array {
using namespace barretenberg;
using namespace plonk;

typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::byte_array<waffle::TurboComposer> byte_array;

TEST(stdlib_byte_array, test_reverse)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::vector<uint8_t> expected = { 0x04, 0x03, 0x02, 0x01 };
    byte_array arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

    EXPECT_EQ(arr.size(), 4UL);
    EXPECT_EQ(arr.reverse().get_value(), expected);
}

TEST(stdlib_byte_array, test_string_constructor)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::string a = "ascii";
    byte_array arr(&composer, a);
    EXPECT_EQ(arr.get_string(), a);
}

TEST(stdlib_byte_array, test_ostream_operator)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::string a = "\1\2\3a";
    byte_array arr(&composer, a);
    std::ostringstream os;
    os << arr;
    EXPECT_EQ(os.str(), "[ 01 02 03 61 ]");
}

TEST(stdlib_byte_array, test_byte_array_input_output_consistency)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    fr a_expected = fr::random_element();
    fr b_expected = fr::random_element();

    field_t a = witness_t(&composer, a_expected);
    field_t b = witness_t(&composer, b_expected);

    byte_array arr(&composer);

    arr.write(static_cast<byte_array>(a));
    arr.write(static_cast<byte_array>(b));

    EXPECT_EQ(arr.size(), 64UL);

    field_t a_result(arr.slice(0, 32));
    field_t b_result(arr.slice(32));

    EXPECT_EQ(a_result.get_value(), a_expected);
    EXPECT_EQ(b_result.get_value(), b_expected);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool verified = verifier.verify_proof(proof);
    EXPECT_EQ(verified, true);
}

TEST(stdlib_byte_array, get_bit)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    byte_array arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

    EXPECT_EQ(arr.get_bit(0).get_value(), false);
    EXPECT_EQ(arr.get_bit(1).get_value(), false);
    EXPECT_EQ(arr.get_bit(2).get_value(), true);
    EXPECT_EQ(arr.get_bit(3).get_value(), false);
    EXPECT_EQ(arr.get_bit(4).get_value(), false);
    EXPECT_EQ(arr.get_bit(5).get_value(), false);
    EXPECT_EQ(arr.get_bit(6).get_value(), false);
    EXPECT_EQ(arr.get_bit(7).get_value(), false);

    EXPECT_EQ(arr.get_bit(8).get_value(), true);
    EXPECT_EQ(arr.get_bit(9).get_value(), true);
    EXPECT_EQ(arr.get_bit(10).get_value(), false);
    EXPECT_EQ(arr.get_bit(11).get_value(), false);
    EXPECT_EQ(arr.get_bit(12).get_value(), false);
    EXPECT_EQ(arr.get_bit(13).get_value(), false);
    EXPECT_EQ(arr.get_bit(14).get_value(), false);
    EXPECT_EQ(arr.get_bit(15).get_value(), false);

    EXPECT_EQ(arr.size(), 4UL);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_byte_array, set_bit)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    byte_array arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

    arr.set_bit(16, bool_t(witness_t(&composer, true)));
    arr.set_bit(18, bool_t(witness_t(&composer, true)));
    arr.set_bit(24, bool_t(witness_t(&composer, false)));
    arr.set_bit(0, bool_t(witness_t(&composer, true)));

    const auto out = arr.get_value();
    EXPECT_EQ(out[0], uint8_t(0));
    EXPECT_EQ(out[1], uint8_t(7));
    EXPECT_EQ(out[3], uint8_t(5));

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stdlib_byte_array