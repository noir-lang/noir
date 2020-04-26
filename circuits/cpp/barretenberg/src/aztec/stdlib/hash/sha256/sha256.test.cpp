#include "sha256.hpp"
#include <common/test.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <stdlib/types/turbo.hpp>

namespace test_stdlib_sha256 {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

TEST(stdlib_sha256, test_55_bytes)
{
    // 55 bytes is the largest number of bytes that can be hashed in a single block,
    // accounting for the single padding bit, and the 64 size bits required by the SHA-256 standard.
    Composer composer = Composer();
    bit_array_ct input(&composer, "An 8 character password? Snow White and the 7 Dwarves..");

    bit_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32_ct> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x51b2529fU);
    EXPECT_EQ(output[1].get_value(), 0x872e839aU);
    EXPECT_EQ(output[2].get_value(), 0xb686c3c2U);
    EXPECT_EQ(output[3].get_value(), 0x483c872eU);
    EXPECT_EQ(output[4].get_value(), 0x975bd672U);
    EXPECT_EQ(output[5].get_value(), 0xbde22ab0U);
    EXPECT_EQ(output[6].get_value(), 0x54a8fac7U);
    EXPECT_EQ(output[7].get_value(), 0x93791fc7U);
    printf("composer gates = %zu\n", composer.get_num_gates());

    Prover prover = composer.create_prover();

    Verifier verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_one)
{
    Composer composer = Composer();

    bit_array_ct input(&composer, "abc");

    bit_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32_ct> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0xBA7816BFU);
    EXPECT_EQ(output[1].get_value(), 0x8F01CFEAU);
    EXPECT_EQ(output[2].get_value(), 0x414140DEU);
    EXPECT_EQ(output[3].get_value(), 0x5DAE2223U);
    EXPECT_EQ(output[4].get_value(), 0xB00361A3U);
    EXPECT_EQ(output[5].get_value(), 0x96177A9CU);
    EXPECT_EQ(output[6].get_value(), 0xB410FF61U);
    EXPECT_EQ(output[7].get_value(), 0xF20015ADU);
    printf("composer gates = %zu\n", composer.get_num_gates());

    Prover prover = composer.create_prover();

    Verifier verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_two)
{
    Composer composer = Composer();

    bit_array_ct input(&composer, "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");

    bit_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32_ct> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x248D6A61U);
    EXPECT_EQ(output[1].get_value(), 0xD20638B8U);
    EXPECT_EQ(output[2].get_value(), 0xE5C02693U);
    EXPECT_EQ(output[3].get_value(), 0x0C3E6039U);
    EXPECT_EQ(output[4].get_value(), 0xA33CE459U);
    EXPECT_EQ(output[5].get_value(), 0x64FF2167U);
    EXPECT_EQ(output[6].get_value(), 0xF6ECEDD4U);
    EXPECT_EQ(output[7].get_value(), 0x19DB06C1U);
    printf("composer gates = %zu\n", composer.get_num_gates());

    Prover prover = composer.create_prover();

    Verifier verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_three)
{
    Composer composer = Composer();

    // one byte, 0xbd
    bit_array_ct input(&composer, 8);
    input[0] = witness_ct(&composer, true);
    input[1] = witness_ct(&composer, false);
    input[2] = witness_ct(&composer, true);
    input[3] = witness_ct(&composer, true);
    input[4] = witness_ct(&composer, true);
    input[5] = witness_ct(&composer, true);
    input[6] = witness_ct(&composer, false);
    input[7] = witness_ct(&composer, true);

    bit_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32_ct> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x68325720U);
    EXPECT_EQ(output[1].get_value(), 0xaabd7c82U);
    EXPECT_EQ(output[2].get_value(), 0xf30f554bU);
    EXPECT_EQ(output[3].get_value(), 0x313d0570U);
    EXPECT_EQ(output[4].get_value(), 0xc95accbbU);
    EXPECT_EQ(output[5].get_value(), 0x7dc4b5aaU);
    EXPECT_EQ(output[6].get_value(), 0xe11204c0U);
    EXPECT_EQ(output[7].get_value(), 0x8ffe732bU);
    printf("composer gates = %zu\n", composer.get_num_gates());

    Prover prover = composer.create_prover();

    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_four)
{
    Composer composer = Composer();

    // 4 bytes, 0xc98c8e55
    std::array<uint32_ct, 1> data;
    data[0] = witness_ct(&composer, 0xc98c8e55);
    bit_array_ct input(data);

    bit_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32_ct> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0x7abc22c0U);
    EXPECT_EQ(output[1].get_value(), 0xae5af26cU);
    EXPECT_EQ(output[2].get_value(), 0xe93dbb94U);
    EXPECT_EQ(output[3].get_value(), 0x433a0e0bU);
    EXPECT_EQ(output[4].get_value(), 0x2e119d01U);
    EXPECT_EQ(output[5].get_value(), 0x4f8e7f65U);
    EXPECT_EQ(output[6].get_value(), 0xbd56c61cU);
    EXPECT_EQ(output[7].get_value(), 0xcccd9504U);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

HEAVY_TEST(stdlib_sha256, test_NIST_vector_five)
{
    Composer composer = Composer();

    bit_array_ct input(
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

    bit_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<uint32_ct> output = output_bits.to_uint32_vector();

    EXPECT_EQ(output[0].get_value(), 0xc2e68682U);
    EXPECT_EQ(output[1].get_value(), 0x3489ced2U);
    EXPECT_EQ(output[2].get_value(), 0x017f6059U);
    EXPECT_EQ(output[3].get_value(), 0xb8b23931U);
    EXPECT_EQ(output[4].get_value(), 0x8b6364f6U);
    EXPECT_EQ(output[5].get_value(), 0xdcd835d0U);
    EXPECT_EQ(output[6].get_value(), 0xa519105aU);
    EXPECT_EQ(output[7].get_value(), 0x1eadd6e4U);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stdlib_sha256