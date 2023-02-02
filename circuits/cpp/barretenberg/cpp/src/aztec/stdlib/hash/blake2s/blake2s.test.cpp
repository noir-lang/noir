#include "blake2s.hpp"
#include "blake2s_plookup.hpp"
#include <crypto/blake2s/blake2s.hpp>
#include <gtest/gtest.h>
#include <stdlib/types/types.hpp>

using namespace barretenberg;
using namespace plonk;

using namespace plonk::stdlib::types;
typedef stdlib::byte_array<Composer> byte_array;
typedef stdlib::byte_array<waffle::UltraComposer> byte_array_plookup;
typedef stdlib::public_witness_t<Composer> public_witness_t;
typedef stdlib::public_witness_t<waffle::UltraComposer> public_witness_t_plookup;

TEST(stdlib_blake2s, test_single_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_ct input_arr(&composer, input_v);
    byte_array_ct output = stdlib::blake2s(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake2s, test_single_block_plookup)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&composer, input_v);
    byte_array_plookup output = stdlib::blake2s<waffle::UltraComposer>(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake2s, test_double_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_ct input_arr(&composer, input_v);
    byte_array_ct output = stdlib::blake2s(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake2s, test_double_block_plookup)
{
    waffle::UltraComposer composer = waffle::UltraComposer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&composer, input_v);
    byte_array_plookup output = stdlib::blake2s(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
