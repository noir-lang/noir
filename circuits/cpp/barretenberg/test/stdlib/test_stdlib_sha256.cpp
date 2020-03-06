#include <gtest/gtest.h>

#include <barretenberg/waffle/composer/turbo_composer.hpp>
#include <barretenberg/waffle/proof_system/preprocess.hpp>
#include <barretenberg/waffle/proof_system/prover/prover.hpp>
#include <barretenberg/waffle/proof_system/verifier/verifier.hpp>
#include <barretenberg/waffle/proof_system/widgets/arithmetic_widget.hpp>

#include <barretenberg/waffle/stdlib/bitarray/bitarray.hpp>
#include <barretenberg/waffle/stdlib/common.hpp>
#include <barretenberg/waffle/stdlib/crypto/hash/sha256.hpp>
#include <barretenberg/waffle/stdlib/uint32/uint32.hpp>

#include <iostream>
#include <memory>

#include "../test_helpers.hpp"

namespace test_stdlib_sha256 {
using namespace barretenberg;
using namespace plonk;

typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::uint32<waffle::TurboComposer> uint32;
typedef stdlib::bitarray<waffle::TurboComposer> bitarray;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

// TEST(stdlib_sha256, test_sha256)
// {
//     waffle::TurboComposer composer = waffle::TurboComposer();
//     std::array<uint32, 16> inputs;
//     for (size_t i = 0; i < 16; ++i) {
//         inputs[i] = uint32(public_witness_t(&composer, get_random_int()));
//     }

//     std::array<uint32, 8> init_constants;
//     plonk::stdlib::prepare_constants(init_constants);
//     plonk::stdlib::sha256_block(init_constants, inputs);

//     waffle::TurboProver prover = composer.create_prover();

//     printf("composer gates = %zu\n", composer.get_num_gates());
//     waffle::TurboVerifier verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool result = verifier.verify_proof(proof);
//     EXPECT_EQ(result, true);
// }

TEST(stdlib_sha256, test_55_bytes)
{
    // 55 bytes is the largest number of bytes that can be hashed in a single block,
    // accounting for the single padding bit, and the 64 size bits required by the SHA-256 standard.
    waffle::TurboComposer composer = waffle::TurboComposer();
    bitarray input(&composer, "An 8 character password? Snow White and the 7 Dwarves..");

    bitarray output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x51b2529fU);
    EXPECT_EQ(output[1].get_value(), 0x872e839aU);
    EXPECT_EQ(output[2].get_value(), 0xb686c3c2U);
    EXPECT_EQ(output[3].get_value(), 0x483c872eU);
    EXPECT_EQ(output[4].get_value(), 0x975bd672U);
    EXPECT_EQ(output[5].get_value(), 0xbde22ab0U);
    EXPECT_EQ(output[6].get_value(), 0x54a8fac7U);
    EXPECT_EQ(output[7].get_value(), 0x93791fc7U);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_one)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    bitarray input(&composer, "abc");

    bitarray output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0xBA7816BFU);
    EXPECT_EQ(output[1].get_value(), 0x8F01CFEAU);
    EXPECT_EQ(output[2].get_value(), 0x414140DEU);
    EXPECT_EQ(output[3].get_value(), 0x5DAE2223U);
    EXPECT_EQ(output[4].get_value(), 0xB00361A3U);
    EXPECT_EQ(output[5].get_value(), 0x96177A9CU);
    EXPECT_EQ(output[6].get_value(), 0xB410FF61U);
    EXPECT_EQ(output[7].get_value(), 0xF20015ADU);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_two)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    bitarray input(&composer, "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");

    bitarray output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x248D6A61U);
    EXPECT_EQ(output[1].get_value(), 0xD20638B8U);
    EXPECT_EQ(output[2].get_value(), 0xE5C02693U);
    EXPECT_EQ(output[3].get_value(), 0x0C3E6039U);
    EXPECT_EQ(output[4].get_value(), 0xA33CE459U);
    EXPECT_EQ(output[5].get_value(), 0x64FF2167U);
    EXPECT_EQ(output[6].get_value(), 0xF6ECEDD4U);
    EXPECT_EQ(output[7].get_value(), 0x19DB06C1U);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_three)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    // one byte, 0xbd
    bitarray input(&composer, 8);
    input[0] = witness_t(&composer, true);
    input[1] = witness_t(&composer, false);
    input[2] = witness_t(&composer, true);
    input[3] = witness_t(&composer, true);
    input[4] = witness_t(&composer, true);
    input[5] = witness_t(&composer, true);
    input[6] = witness_t(&composer, false);
    input[7] = witness_t(&composer, true);

    bitarray output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x68325720U);
    EXPECT_EQ(output[1].get_value(), 0xaabd7c82U);
    EXPECT_EQ(output[2].get_value(), 0xf30f554bU);
    EXPECT_EQ(output[3].get_value(), 0x313d0570U);
    EXPECT_EQ(output[4].get_value(), 0xc95accbbU);
    EXPECT_EQ(output[5].get_value(), 0x7dc4b5aaU);
    EXPECT_EQ(output[6].get_value(), 0xe11204c0U);
    EXPECT_EQ(output[7].get_value(), 0x8ffe732bU);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_four)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    // 4 bytes, 0xc98c8e55
    std::array<uint32, 1> data;
    data[0] = witness_t(&composer, 0xc98c8e55);
    bitarray input(data);

    bitarray output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x7abc22c0U);
    EXPECT_EQ(output[1].get_value(), 0xae5af26cU);
    EXPECT_EQ(output[2].get_value(), 0xe93dbb94U);
    EXPECT_EQ(output[3].get_value(), 0x433a0e0bU);
    EXPECT_EQ(output[4].get_value(), 0x2e119d01U);
    EXPECT_EQ(output[5].get_value(), 0x4f8e7f65U);
    EXPECT_EQ(output[6].get_value(), 0xbd56c61cU);
    EXPECT_EQ(output[7].get_value(), 0xcccd9504U);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_five)
{
    waffle::TurboComposer composer = waffle::TurboComposer();

    bitarray input(
        &composer,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        "AAAAAAAAAA");

    bitarray output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0xc2e68682U);
    EXPECT_EQ(output[1].get_value(), 0x3489ced2U);
    EXPECT_EQ(output[2].get_value(), 0x017f6059U);
    EXPECT_EQ(output[3].get_value(), 0xb8b23931U);
    EXPECT_EQ(output[4].get_value(), 0x8b6364f6U);
    EXPECT_EQ(output[5].get_value(), 0xdcd835d0U);
    EXPECT_EQ(output[6].get_value(), 0xa519105aU);
    EXPECT_EQ(output[7].get_value(), 0x1eadd6e4U);

    waffle::TurboProver prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stdlib_sha256