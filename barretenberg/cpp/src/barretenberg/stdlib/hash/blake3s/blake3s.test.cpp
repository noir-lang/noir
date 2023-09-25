#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include "barretenberg/common/streams.hpp"
#include "blake3s.hpp"
#include "blake3s_plookup.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace proof_system::plonk;

typedef proof_system::TurboCircuitBuilder Builder;
typedef stdlib::byte_array<Builder> byte_array;
typedef stdlib::byte_array<proof_system::UltraCircuitBuilder> byte_array_plookup;
typedef stdlib::public_witness_t<Builder> public_witness_t;
typedef stdlib::public_witness_t<proof_system::UltraCircuitBuilder> public_witness_t_plookup;

TEST(stdlib_blake3s, test_single_block)
{
    Builder builder = Builder();
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
    proof_system::UltraCircuitBuilder builder = proof_system::UltraCircuitBuilder();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&builder, input_v);
    byte_array_plookup output = stdlib::blake3s<proof_system::UltraCircuitBuilder>(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("builder gates = ", builder.get_num_gates());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_double_block)
{
    Builder builder = Builder();
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
    proof_system::UltraCircuitBuilder builder = proof_system::UltraCircuitBuilder();
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
