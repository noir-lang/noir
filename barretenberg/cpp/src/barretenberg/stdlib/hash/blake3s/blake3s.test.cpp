#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include "barretenberg/common/streams.hpp"
#include "blake3s.hpp"
#include "blake3s_plookup.hpp"
#include <gtest/gtest.h>

using namespace bb;

using byte_array = stdlib::byte_array<bb::StandardCircuitBuilder>;
using public_witness_t = stdlib::public_witness_t<bb::StandardCircuitBuilder>;
using byte_array_plookup = stdlib::byte_array<bb::UltraCircuitBuilder>;
using public_witness_t_plookup = stdlib::public_witness_t<bb::UltraCircuitBuilder>;
using StandardBuilder = StandardCircuitBuilder;
using UltraBuilder = UltraCircuitBuilder;

TEST(stdlib_blake3s, test_single_block)
{
    auto builder = StandardBuilder();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&builder, input_v);
    byte_array output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_single_block_plookup)
{
    auto builder = UltraBuilder();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&builder, input_v);
    byte_array_plookup output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_double_block)
{
    auto builder = StandardBuilder();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&builder, input_v);
    byte_array output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_double_block_plookup)
{
    auto builder = UltraBuilder();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&builder, input_v);
    byte_array_plookup output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}
