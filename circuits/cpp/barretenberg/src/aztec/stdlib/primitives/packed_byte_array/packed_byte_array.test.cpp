#include "packed_byte_array.hpp"
#include "../byte_array/byte_array.hpp"

#include <gtest/gtest.h>
#include <plonk/composer/turbo_composer.hpp>

#include <numeric/random/engine.hpp>

namespace test_stdlib_packed_byte_array {
using namespace barretenberg;
using namespace plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}
typedef stdlib::packed_byte_array<waffle::TurboComposer> packed_byte_array;
typedef stdlib::byte_array<waffle::TurboComposer> byte_array;

TEST(packed_byte_array, string_constructor_and_get_value_consistency)
{
    std::string input = "the quick brown fox jumped over the lazy dog.";

    waffle::TurboComposer composer = waffle::TurboComposer();

    packed_byte_array arr(&composer, input);

    std::string output = arr.get_value();

    EXPECT_EQ(input, output);
}

TEST(packed_byte_array, byte_array_constructor_consistency)
{
    std::string input = "the quick brown fox jumped over the lazy dog.";

    waffle::TurboComposer composer = waffle::TurboComposer();

    byte_array arr(&composer, input);
    packed_byte_array converted(arr);
    std::string output = converted.get_value();

    EXPECT_EQ(input, output);
}

TEST(packed_byte_array, byte_array_cast_consistency)
{
    std::string input = "the quick brown fox jumped over the lazy dog.";

    waffle::TurboComposer composer = waffle::TurboComposer();

    packed_byte_array arr(&composer, input);
    byte_array converted(arr);
    std::string output = converted.get_string();

    EXPECT_EQ(input, output);
}

TEST(packed_byte_array, unverified_byte_slices)
{

    std::vector<uint8_t> bytes;
    for (size_t i = 0; i < 256; ++i) {
        bytes.push_back(engine.get_random_uint8());
    }

    std::vector<uint32_t> uint32s;
    for (size_t i = 0; i < 64; ++i) {
        uint32_t result = ((uint32_t)bytes[i * 4] << 24) + ((uint32_t)bytes[i * 4 + 1] << 16) +
                          ((uint32_t)bytes[i * 4 + 2] << 8) + ((uint32_t)bytes[i * 4 + 3]);
        uint32s.push_back(result);
    }

    waffle::TurboComposer composer = waffle::TurboComposer();

    packed_byte_array arr(&composer, bytes);

    const auto result_elements = arr.to_unverified_byte_slices(4);

    for (size_t i = 0; i < 64; ++i) {
        uint32_t result = static_cast<uint32_t>(uint256_t(result_elements[i].get_value()).data[0]);
        EXPECT_EQ(result, uint32s[i]);
    }
}
} // namespace test_stdlib_packed_byte_array