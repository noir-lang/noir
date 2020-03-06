#include <gtest/gtest.h>
#include <memory>

#include <barretenberg/waffle/composer/turbo_composer.hpp>
#include <barretenberg/waffle/proof_system/preprocess.hpp>
#include <barretenberg/waffle/proof_system/prover/prover.hpp>
#include <barretenberg/waffle/proof_system/verifier/verifier.hpp>
#include <barretenberg/waffle/proof_system/widgets/arithmetic_widget.hpp>

#include <barretenberg/polynomials/polynomial_arithmetic.hpp>

#include <barretenberg/waffle/stdlib/bitarray/bitarray.hpp>
#include <barretenberg/waffle/stdlib/byte_array/byte_array.hpp>
#include <barretenberg/waffle/stdlib/common.hpp>
#include <barretenberg/waffle/stdlib/field/field.hpp>
#include <barretenberg/waffle/stdlib/uint32/uint32.hpp>

namespace test_stdlib_byte_array {
using namespace barretenberg;
using namespace plonk;

typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::uint32<waffle::TurboComposer> uint32;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::byte_array<waffle::TurboComposer> byte_array;

TEST(stdlib_byte_array, test_uint32_byte_array_conversion)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    uint32 a = witness_t(&composer, 0x10000002);
    std::string expected = { 0x10, 0x00, 0x00, 0x02 };
    byte_array arr(&composer);
    arr.write(a);

    EXPECT_EQ(arr.size(), 4UL);
    EXPECT_EQ(arr.get_value(), expected);
}

TEST(stdlib_byte_array, test_reverse)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    uint32 a = witness_t(&composer, 1UL);
    std::string expected = { 0x04, 0x03, 0x02, 0x01 };
    byte_array arr(&composer, std::vector<uint8_t>{ 0x01, 0x02, 0x03, 0x04 });

    EXPECT_EQ(arr.size(), 4UL);
    EXPECT_EQ(arr.reverse().get_value(), expected);
}

TEST(stdlib_byte_array, test_uint32_input_output_consistency)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    for (size_t i = 1; i < 1024; i *= 2) {
        uint32_t a_expected = (uint32_t)i;
        uint32_t b_expected = (uint32_t)i;

        uint32 a = witness_t(&composer, a_expected);
        uint32 b = witness_t(&composer, b_expected);

        byte_array arr(&composer);

        arr.write(a);
        arr.write(b);

        EXPECT_EQ(arr.size(), 8UL);

        uint32 a_result(arr.slice(0, 4));
        uint32 b_result(arr.slice(4));

        EXPECT_EQ(a_result.get_value(), a_expected);
        EXPECT_EQ(b_result.get_value(), b_expected);
    }
}

TEST(stdlib_byte_array, test_field_t_input_output_consistency)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    fr a_expected = fr::random_element();
    fr b_expected = fr::random_element();

    field_t a = witness_t(&composer, a_expected);
    field_t b = witness_t(&composer, b_expected);

    byte_array arr(&composer);

    arr.write(a);
    arr.write(b);

    EXPECT_EQ(arr.size(), 64UL);

    field_t a_result(arr.slice(0, 32));
    field_t b_result(arr.slice(32));

    EXPECT_EQ(a_result.get_value(), a_expected);
    EXPECT_EQ(b_result.get_value(), b_expected);
}

TEST(stdlib_byte_array, test_string_constructor)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::string a = "ascii";
    byte_array arr(&composer, a);
    EXPECT_EQ(arr.get_value(), a);
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

} // namespace test_stdlib_byte_array