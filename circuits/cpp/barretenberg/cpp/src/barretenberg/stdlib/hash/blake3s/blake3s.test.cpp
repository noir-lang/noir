#include "blake3s.hpp"
#include "blake3s_plookup.hpp"
#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include "barretenberg/common/streams.hpp"
#include <gtest/gtest.h>
#include "barretenberg/plonk/composer/turbo_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"

using namespace barretenberg;
using namespace proof_system::plonk;

typedef proof_system::plonk::TurboComposer Composer;
typedef stdlib::byte_array<Composer> byte_array;
typedef stdlib::byte_array<plonk::UltraComposer> byte_array_plookup;
typedef stdlib::public_witness_t<Composer> public_witness_t;
typedef stdlib::public_witness_t<plonk::UltraComposer> public_witness_t_plookup;

TEST(stdlib_blake3s, test_single_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&composer, input_v);
    byte_array output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_single_block_plookup)
{
    plonk::UltraComposer composer = proof_system::plonk::UltraComposer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&composer, input_v);
    byte_array_plookup output = stdlib::blake3s<plonk::UltraComposer>(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_double_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&composer, input_v);
    byte_array output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake3s, test_double_block_plookup)
{
    plonk::UltraComposer composer = proof_system::plonk::UltraComposer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array_plookup input_arr(&composer, input_v);
    byte_array_plookup output = stdlib::blake3s(input_arr);

    std::vector<uint8_t> expected = blake3::blake3s(input_v);

    EXPECT_EQ(output.get_value(), expected);

    auto prover = composer.create_prover();
    std::cout << "prover gates = " << prover.circuit_size << std::endl;
    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
