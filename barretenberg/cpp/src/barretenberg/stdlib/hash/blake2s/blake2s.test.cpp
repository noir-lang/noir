#include "barretenberg/crypto/blake2s/blake2s.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "blake2s.hpp"
#include "blake2s_plookup.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::stdlib;

using Builder = bb::UltraCircuitBuilder;

using field_ct = field_t<Builder>;
using witness_ct = witness_t<Builder>;
using byte_array_ct = byte_array<Builder>;
using byte_array_plookup = byte_array<Builder>;
using public_witness_t = public_witness_t<Builder>;

// TEST(stdlib_blake2s, test_single_block)
// {
//     auto builder = Builder();
//     std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
//     std::vector<uint8_t> input_v(input.begin(), input.end());

//     byte_array_ct input_arr(&builder, input_v);
//     byte_array_ct output = blake2s(input_arr);

//     std::vector<uint8_t> expected = blake2::blake2s(input_v);

//     EXPECT_EQ(output.get_value(), expected);

//     info("num gates = %zu\n", builder.get_num_gates());

//     bool proof_result = builder.check_circuit();
//     EXPECT_EQ(proof_result, true);
// }

TEST(stdlib_blake2s, test_single_block_plookup)
{
    Builder builder;
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&builder, input_v);
    byte_array_plookup output = blake2s<Builder>(input_arr);

    auto expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), std::vector<uint8_t>(expected.begin(), expected.end()));

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

// TEST(stdlib_blake2s, test_double_block)
// {
//     auto builder = Builder();
//     std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
//     std::vector<uint8_t> input_v(input.begin(), input.end());

//     byte_array_ct input_arr(&builder, input_v);
//     byte_array_ct output = blake2s(input_arr);

//     std::vector<uint8_t> expected = blake2::blake2s(input_v);

//     EXPECT_EQ(output.get_value(), expected);

//     info("num gates = %zu\n", builder.get_num_gates());

//     bool proof_result = builder.check_circuit();
//     EXPECT_EQ(proof_result, true);
// }

TEST(stdlib_blake2s, test_double_block_plookup)
{
    Builder builder;
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&builder, input_v);
    byte_array_plookup output = blake2s<Builder>(input_arr);

    auto expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), std::vector<uint8_t>(expected.begin(), expected.end()));

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}
