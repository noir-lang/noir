#include "blake2s.hpp"
#include "blake2s_plookup.hpp"
#include <gtest/gtest.h>
#include "barretenberg/crypto/blake2s/blake2s.hpp"

using namespace barretenberg;
using namespace proof_system::plonk;

using namespace plonk::stdlib;

using Composer = proof_system::UltraCircuitConstructor;

using field_ct = field_t<Composer>;
using witness_ct = witness_t<Composer>;
using byte_array_ct = stdlib::byte_array<Composer>;
using byte_array_plookup = stdlib::byte_array<Composer>;
using public_witness_t = stdlib::public_witness_t<Composer>;
using public_witness_t_plookup = stdlib::public_witness_t<Composer>;

// TEST(stdlib_blake2s, test_single_block)
// {
//     auto composer = Composer();
//     std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
//     std::vector<uint8_t> input_v(input.begin(), input.end());

//     byte_array_ct input_arr(&composer, input_v);
//     byte_array_ct output = stdlib::blake2s(input_arr);

//     std::vector<uint8_t> expected = blake2::blake2s(input_v);

//     EXPECT_EQ(output.get_value(), expected);

//     info("composer gates = %zu\n", composer.get_num_gates());

//     bool proof_result = composer.check_circuit();
//     EXPECT_EQ(proof_result, true);
// }

TEST(stdlib_blake2s, test_single_block_plookup)
{
    proof_system::UltraCircuitConstructor composer = UltraCircuitConstructor();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&composer, input_v);
    byte_array_plookup output = stdlib::blake2s<proof_system::UltraCircuitConstructor>(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("composer gates = ", composer.get_num_gates());

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

// TEST(stdlib_blake2s, test_double_block)
// {
//     auto composer = Composer();
//     std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
//     std::vector<uint8_t> input_v(input.begin(), input.end());

//     byte_array_ct input_arr(&composer, input_v);
//     byte_array_ct output = stdlib::blake2s(input_arr);

//     std::vector<uint8_t> expected = blake2::blake2s(input_v);

//     EXPECT_EQ(output.get_value(), expected);

//     info("composer gates = %zu\n", composer.get_num_gates());

//     bool proof_result = composer.check_circuit();
//     EXPECT_EQ(proof_result, true);
// }

TEST(stdlib_blake2s, test_double_block_plookup)
{
    proof_system::UltraCircuitConstructor composer = UltraCircuitConstructor();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&composer, input_v);
    byte_array_plookup output = stdlib::blake2s<proof_system::UltraCircuitConstructor>(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    info("composer gates = ", composer.get_num_gates());

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}
