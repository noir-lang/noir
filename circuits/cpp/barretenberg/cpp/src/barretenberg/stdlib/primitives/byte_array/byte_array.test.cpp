#include "byte_array.hpp"
#include <gtest/gtest.h>
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

#pragma GCC diagnostic ignored "-Wunused-local-typedefs"

namespace test_stdlib_byte_array {
using namespace barretenberg;
using namespace plonk;

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Composer = TypeParam;                                                                                        \
    using witness_ct = stdlib::witness_t<Composer>;                                                                    \
    using byte_array_ct = stdlib::byte_array<Composer>;                                                                \
    using field_ct = stdlib::field_t<Composer>;                                                                        \
    using bool_ct = stdlib::bool_t<Composer>;

template <class Composer> class ByteArrayTest : public ::testing::Test {};

template <class Composer> using byte_array_ct = stdlib::byte_array<Composer>;

using ComposerTypes =
    ::testing::Types<honk::StandardHonkComposer, plonk::StandardComposer, plonk::TurboComposer, plonk::UltraComposer>;
TYPED_TEST_SUITE(ByteArrayTest, ComposerTypes);

TYPED_TEST(ByteArrayTest, test_reverse)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    std::vector<uint8_t> expected = { 0x04, 0x03, 0x02, 0x01 };
    byte_array_ct arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

    EXPECT_EQ(arr.size(), 4UL);
    EXPECT_EQ(arr.reverse().get_value(), expected);
}

TYPED_TEST(ByteArrayTest, test_string_constructor)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    std::string a = "ascii";
    byte_array_ct arr(&composer, a);
    EXPECT_EQ(arr.get_string(), a);
}

TYPED_TEST(ByteArrayTest, test_ostream_operator)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    std::string a = "\1\2\3a";
    byte_array_ct arr(&composer, a);
    std::ostringstream os;
    os << arr;
    EXPECT_EQ(os.str(), "[ 01 02 03 61 ]");
}

TYPED_TEST(ByteArrayTest, test_byte_array_input_output_consistency)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    fr a_expected = fr::random_element();
    fr b_expected = fr::random_element();

    field_ct a = witness_ct(&composer, a_expected);
    field_ct b = witness_ct(&composer, b_expected);

    byte_array_ct arr(&composer);

    arr.write(static_cast<byte_array_ct>(a));
    arr.write(static_cast<byte_array_ct>(b));

    EXPECT_EQ(arr.size(), 64UL);

    field_ct a_result(arr.slice(0, 32));
    field_ct b_result(arr.slice(32));

    EXPECT_EQ(a_result.get_value(), a_expected);
    EXPECT_EQ(b_result.get_value(), b_expected);

    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool verified = verifier.verify_proof(proof);
    EXPECT_EQ(verified, true);
}

TYPED_TEST(ByteArrayTest, get_bit)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    byte_array_ct arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

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

TYPED_TEST(ByteArrayTest, set_bit)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    byte_array_ct arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

    arr.set_bit(16, bool_ct(witness_ct(&composer, true)));
    arr.set_bit(18, bool_ct(witness_ct(&composer, true)));
    arr.set_bit(24, bool_ct(witness_ct(&composer, false)));
    arr.set_bit(0, bool_ct(witness_ct(&composer, true)));

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
