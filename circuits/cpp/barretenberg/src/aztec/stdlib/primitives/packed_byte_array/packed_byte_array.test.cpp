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

TEST(packed_byte_array, check_append_uint8)
{
    std::vector<uint8_t> bytes;
    const size_t initial_size = 100;
    auto floor = 1UL << numeric::get_msb(initial_size);
    auto next_pow_2 = floor << (initial_size != floor);

    for (size_t i = 0; i < initial_size; ++i) {
        bytes.push_back(engine.get_random_uint8());
    }

    waffle::TurboComposer composer = waffle::TurboComposer();
    packed_byte_array arr(&composer, bytes);

    // append upto size (16x)
    size_t num_bytes_to_append = next_pow_2 - initial_size;
    for (size_t i = 0; i < num_bytes_to_append; ++i) {
        uint8_t byte_to_append = engine.get_random_uint8();
        bytes.push_back(byte_to_append);
        arr.append(byte_to_append, 1);
    }

    // append over size (16x) (this creates new limb internally)
    num_bytes_to_append = 20;
    for (size_t i = 0; i < num_bytes_to_append; ++i) {
        uint8_t byte_to_append = engine.get_random_uint8();
        bytes.push_back(byte_to_append);
        arr.append(byte_to_append, 1);
    }

    // append two bytes at once, example: 0x004c
    num_bytes_to_append = 26;
    for (size_t i = 0; i < num_bytes_to_append; ++i) {
        uint8_t byte_to_append = engine.get_random_uint8();
        bytes.push_back(0);
        bytes.push_back(byte_to_append);
        arr.append(byte_to_append, 2);
    }

    EXPECT_EQ(bytes.size(), arr.size());
    const auto result_elements = arr.to_unverified_byte_slices(1);
    EXPECT_EQ(bytes.size(), result_elements.size());
    for (size_t i = 0; i < bytes.size(); ++i) {
        uint8_t result = static_cast<uint8_t>(uint256_t(result_elements[i].get_value()).data[0]);
        EXPECT_EQ(result, bytes[i]);
    }
}

TEST(packed_byte_array, check_append_uint32)
{
    std::vector<uint8_t> bytes;
    const size_t initial_size = 100;
    auto floor = 1UL << numeric::get_msb(initial_size);
    auto next_pow_2 = floor << (initial_size != floor);

    for (size_t i = 0; i < next_pow_2; ++i) {
        bytes.push_back(engine.get_random_uint8());
    }

    std::vector<uint32_t> uint32s;
    for (size_t i = 0; i < (next_pow_2 >> 2); ++i) {
        uint32_t result = ((uint32_t)bytes[i * 4] << 24) + ((uint32_t)bytes[i * 4 + 1] << 16) +
                          ((uint32_t)bytes[i * 4 + 2] << 8) + ((uint32_t)bytes[i * 4 + 3]);
        uint32s.push_back(result);
    }

    waffle::TurboComposer composer = waffle::TurboComposer();
    packed_byte_array arr(&composer, bytes);

    // append over size (16x) (this creates new limb internally)
    size_t num_bytes_to_append = 20;
    for (size_t i = 0; i < num_bytes_to_append; ++i) {
        uint32_t word_to_append = engine.get_random_uint32();
        uint32s.push_back(word_to_append);
        arr.append(uint256_t(word_to_append), 4);
    }

    // append eight bytes at once, example: 0x000000001e087f4c
    num_bytes_to_append = 26;
    for (size_t i = 0; i < num_bytes_to_append; ++i) {
        uint32_t word_to_append = engine.get_random_uint32();
        uint32s.push_back(0);
        uint32s.push_back(word_to_append);
        arr.append(uint256_t(word_to_append), 8);
    }

    EXPECT_EQ(uint32s.size() * 4, arr.size());
    const auto result_elements = arr.to_unverified_byte_slices(4);
    EXPECT_EQ(uint32s.size(), result_elements.size());
    for (size_t i = 0; i < uint32s.size(); ++i) {
        uint32_t result = static_cast<uint32_t>(uint256_t(result_elements[i].get_value()).data[0]);
        EXPECT_EQ(result, uint32s[i]);
    }
}

} // namespace test_stdlib_packed_byte_array