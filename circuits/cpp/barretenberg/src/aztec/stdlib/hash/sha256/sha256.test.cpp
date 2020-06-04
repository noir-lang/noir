#include "sha256.hpp"
#include <common/test.hpp>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <stdlib/types/turbo.hpp>

#include <numeric/random/engine.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace test_stdlib_sha256 {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

constexpr uint64_t ror(uint64_t val, uint64_t shift)
{
    return (val >> (shift & 31U)) | (val << (32U - (shift & 31U)));
}

std::array<uint64_t, 64> extend_witness(std::array<uint64_t, 16>& in)
{
    std::array<uint64_t, 64> w;

    for (size_t i = 0; i < 16; ++i) {
        w[i] = in[i];
    }

    for (size_t i = 16; i < 64; ++i) {
        uint64_t left = w[i - 15];
        uint64_t right = w[i - 2];

        uint64_t left_rot7 = numeric::rotate32((uint32_t)left, 7);
        uint64_t left_rot18 = numeric::rotate32((uint32_t)left, 18);
        uint64_t left_sh3 = left >> 3;

        uint64_t right_rot17 = numeric::rotate32((uint32_t)right, 17);
        uint64_t right_rot19 = numeric::rotate32((uint32_t)right, 19);
        uint64_t right_sh10 = right >> 10;

        uint64_t s0 = left_rot7 ^ left_rot18 ^ left_sh3;
        uint64_t s1 = right_rot17 ^ right_rot19 ^ right_sh10;

        w[i] = w[i - 16] + w[i - 7] + s0 + s1;
    }
    return w;
}
std::array<uint64_t, 8> inner_block(std::array<uint64_t, 64>& w)
{
    constexpr uint32_t init_constants[8]{ 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                                          0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19 };

    constexpr uint32_t round_constants[64]{
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    };
    uint32_t a = init_constants[0];
    uint32_t b = init_constants[1];
    uint32_t c = init_constants[2];
    uint32_t d = init_constants[3];
    uint32_t e = init_constants[4];
    uint32_t f = init_constants[5];
    uint32_t g = init_constants[6];
    uint32_t h = init_constants[7];
    for (size_t i = 0; i < 64; ++i) {
        uint32_t S1 = numeric::rotate32((uint32_t)e, 6U) ^ numeric::rotate32((uint32_t)e, 11U) ^
                      numeric::rotate32((uint32_t)e, 25U);
        uint32_t ch = (e & f) ^ (~e & g); // === (e & f) ^ (~e & g), `+` op is cheaper
        uint32_t temp1 = h + S1 + ch + round_constants[i] + (uint32_t)w[i];
        uint32_t S0 = numeric::rotate32((uint32_t)a, 2U) ^ numeric::rotate32((uint32_t)a, 13U) ^
                      numeric::rotate32((uint32_t)a, 22U);
        uint32_t maj = (a & b) ^ (a & c) ^ (b & c); // (a & (b + c - (T0 * 2))) + T0; // === (a & b) ^ (a & c) ^ (b & c)
        uint32_t temp2 = S0 + maj;

        h = g;
        g = f;
        f = e;
        e = d + temp1;
        d = c;
        c = b;
        b = a;
        a = temp1 + temp2;
    }

    /**
     * Add into previous block output and return
     **/
    std::array<uint64_t, 8> output;
    output[0] = (uint32_t)(a + init_constants[0]);
    output[1] = (uint32_t)(b + init_constants[1]);
    output[2] = (uint32_t)(c + init_constants[2]);
    output[3] = (uint32_t)(d + init_constants[3]);
    output[4] = (uint32_t)(e + init_constants[4]);
    output[5] = (uint32_t)(f + init_constants[5]);
    output[6] = (uint32_t)(g + init_constants[6]);
    output[7] = (uint32_t)(h + init_constants[7]);
    return output;
}

// TEST(stdlib_sha256_plookup, test_round)
// {

//     waffle::PLookupComposer composer = waffle::PLookupComposer();

//     std::array<uint64_t, 64> w_inputs;
//     std::array<plonk::stdlib::field_t<waffle::PLookupComposer>, 64> w_elements;

//     for (size_t i = 0; i < 64; ++i) {
//         w_inputs[i] = engine.get_random_uint32();
//         w_elements[i] = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, barretenberg::fr(w_inputs[i]));
//     }

//     const auto expected = inner_block(w_inputs);

//     const std::array<plonk::stdlib::field_t<waffle::PLookupComposer>, 8> result =
//         plonk::stdlib::sha256_inner_block(w_elements);
//     for (size_t i = 0; i < 8; ++i) {
//         EXPECT_EQ(uint256_t(result[i].get_value()).data[0] & 0xffffffffUL,
//                   uint256_t(expected[i]).data[0] & 0xffffffffUL);
//     }
//     printf("composer gates = %zu\n", composer.get_num_gates());

//     auto prover = composer.create_prover();

//     auto verifier = composer.create_verifier();
//     waffle::plonk_proof proof = prover.construct_proof();
//     bool proof_result = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_result, true);
// }

TEST(stdlib_sha256, test_plookup_55_bytes)
{
    typedef plonk::stdlib::field_t<waffle::PLookupComposer> field_pt;
    typedef plonk::stdlib::packed_byte_array<waffle::PLookupComposer> packed_byte_array_pt;

    // 55 bytes is the largest number of bytes that can be hashed in a single block,
    // accounting for the single padding bit, and the 64 size bits required by the SHA-256 standard.
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    packed_byte_array_pt input(&composer, "An 8 character password? Snow White and the 7 Dwarves..");

    packed_byte_array_pt output_bits = plonk::stdlib::sha256(input);

    std::vector<field_pt> output = output_bits.to_unverified_byte_slices(4);

    EXPECT_EQ(uint256_t(output[0].get_value()), 0x51b2529fU);
    EXPECT_EQ(uint256_t(output[1].get_value()), 0x872e839aU);
    EXPECT_EQ(uint256_t(output[2].get_value()), 0xb686c3c2U);
    EXPECT_EQ(uint256_t(output[3].get_value()), 0x483c872eU);
    EXPECT_EQ(uint256_t(output[4].get_value()), 0x975bd672U);
    EXPECT_EQ(uint256_t(output[5].get_value()), 0xbde22ab0U);
    EXPECT_EQ(uint256_t(output[6].get_value()), 0x54a8fac7U);
    EXPECT_EQ(uint256_t(output[7].get_value()), 0x93791fc7U);
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_55_bytes)
{
    // 55 bytes is the largest number of bytes that can be hashed in a single block,
    // accounting for the single padding bit, and the 64 size bits required by the SHA-256 standard.
    Composer composer = Composer();
    packed_byte_array_ct input(&composer, "An 8 character password? Snow White and the 7 Dwarves..");

    packed_byte_array_ct output_bits = plonk::stdlib::sha256(input);

    std::vector<field_ct> output = output_bits.to_unverified_byte_slices(4);

    EXPECT_EQ(output[0].get_value(), fr(0x51b2529fULL));
    EXPECT_EQ(output[1].get_value(), fr(0x872e839aULL));
    EXPECT_EQ(output[2].get_value(), fr(0xb686c3c2ULL));
    EXPECT_EQ(output[3].get_value(), fr(0x483c872eULL));
    EXPECT_EQ(output[4].get_value(), fr(0x975bd672ULL));
    EXPECT_EQ(output[5].get_value(), fr(0xbde22ab0ULL));
    EXPECT_EQ(output[6].get_value(), fr(0x54a8fac7ULL));
    EXPECT_EQ(output[7].get_value(), fr(0x93791fc7ULL));
    printf("composer gates = %zu\n", composer.get_num_gates());

    Prover prover = composer.create_prover();

    Verifier verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_one_packed_byte_array)
{
    typedef plonk::stdlib::field_t<waffle::PLookupComposer> field_pt;
    typedef plonk::stdlib::packed_byte_array<waffle::PLookupComposer> packed_byte_array_pt;

    waffle::PLookupComposer composer = waffle::PLookupComposer();

    packed_byte_array_pt input(&composer, "abc");
    packed_byte_array_pt output_bytes = plonk::stdlib::sha256(input);
    std::vector<field_pt> output = output_bytes.to_unverified_byte_slices(4);
    EXPECT_EQ(uint256_t(output[0].get_value()).data[0], (uint64_t)0xBA7816BFU);
    EXPECT_EQ(uint256_t(output[1].get_value()).data[0], (uint64_t)0x8F01CFEAU);
    EXPECT_EQ(uint256_t(output[2].get_value()).data[0], (uint64_t)0x414140DEU);
    EXPECT_EQ(uint256_t(output[3].get_value()).data[0], (uint64_t)0x5DAE2223U);
    EXPECT_EQ(uint256_t(output[4].get_value()).data[0], (uint64_t)0xB00361A3U);
    EXPECT_EQ(uint256_t(output[5].get_value()).data[0], (uint64_t)0x96177A9CU);
    EXPECT_EQ(uint256_t(output[6].get_value()).data[0], (uint64_t)0xB410FF61U);
    EXPECT_EQ(uint256_t(output[7].get_value()).data[0], (uint64_t)0xF20015ADU);
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_one)
{
    typedef plonk::stdlib::field_t<waffle::PLookupComposer> field_pt;
    typedef plonk::stdlib::packed_byte_array<waffle::PLookupComposer> packed_byte_array_pt;

    waffle::PLookupComposer composer = waffle::PLookupComposer();

    packed_byte_array_pt input(&composer, "abc");

    packed_byte_array_pt output_bits = plonk::stdlib::sha256(input);

    std::vector<field_pt> output = output_bits.to_unverified_byte_slices(4);

    EXPECT_EQ(output[0].get_value(), fr(0xBA7816BFULL));
    EXPECT_EQ(output[1].get_value(), fr(0x8F01CFEAULL));
    EXPECT_EQ(output[2].get_value(), fr(0x414140DEULL));
    EXPECT_EQ(output[3].get_value(), fr(0x5DAE2223ULL));
    EXPECT_EQ(output[4].get_value(), fr(0xB00361A3ULL));
    EXPECT_EQ(output[5].get_value(), fr(0x96177A9CULL));
    EXPECT_EQ(output[6].get_value(), fr(0xB410FF61ULL));
    EXPECT_EQ(output[7].get_value(), fr(0xF20015ADULL));
    printf("composer gates = %zu\n", composer.get_num_gates());

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    printf("constructing proof \n");
    waffle::plonk_proof proof = prover.construct_proof();
    printf("constructed proof \n");

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256, test_NIST_vector_two)
{
    Composer composer = Composer();

    byte_array_ct input(&composer, "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");

    byte_array_ct output_bits = plonk::stdlib::sha256<Composer>(input);

    std::vector<field_ct> output = packed_byte_array_ct(output_bits).to_unverified_byte_slices(4);

    EXPECT_EQ(output[0].get_value(), 0x248D6A61ULL);
    EXPECT_EQ(output[1].get_value(), 0xD20638B8ULL);
    EXPECT_EQ(output[2].get_value(), 0xE5C02693ULL);
    EXPECT_EQ(output[3].get_value(), 0x0C3E6039ULL);
    EXPECT_EQ(output[4].get_value(), 0xA33CE459ULL);
    EXPECT_EQ(output[5].get_value(), 0x64FF2167ULL);
    EXPECT_EQ(output[6].get_value(), 0xF6ECEDD4ULL);
    EXPECT_EQ(output[7].get_value(), 0x19DB06C1ULL);
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
    byte_array_ct input(&composer, std::vector<uint8_t>{ 0xbd });

    byte_array_ct output_bits = plonk::stdlib::sha256<Composer>(input);

    std::vector<field_ct> output = packed_byte_array_ct(output_bits).to_unverified_byte_slices(4);

    EXPECT_EQ(output[0].get_value(), 0x68325720ULL);
    EXPECT_EQ(output[1].get_value(), 0xaabd7c82ULL);
    EXPECT_EQ(output[2].get_value(), 0xf30f554bULL);
    EXPECT_EQ(output[3].get_value(), 0x313d0570ULL);
    EXPECT_EQ(output[4].get_value(), 0xc95accbbULL);
    EXPECT_EQ(output[5].get_value(), 0x7dc4b5aaULL);
    EXPECT_EQ(output[6].get_value(), 0xe11204c0ULL);
    EXPECT_EQ(output[7].get_value(), 0x8ffe732bULL);
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
    byte_array_ct input(&composer, std::vector<uint8_t>{ 0xc9, 0x8c, 0x8e, 0x55 });

    byte_array_ct output_bits = plonk::stdlib::sha256<Composer>(input);

    std::vector<field_ct> output = packed_byte_array_ct(output_bits).to_unverified_byte_slices(4);

    EXPECT_EQ(output[0].get_value(), 0x7abc22c0ULL);
    EXPECT_EQ(output[1].get_value(), 0xae5af26cULL);
    EXPECT_EQ(output[2].get_value(), 0xe93dbb94ULL);
    EXPECT_EQ(output[3].get_value(), 0x433a0e0bULL);
    EXPECT_EQ(output[4].get_value(), 0x2e119d01ULL);
    EXPECT_EQ(output[5].get_value(), 0x4f8e7f65ULL);
    EXPECT_EQ(output[6].get_value(), 0xbd56c61cULL);
    EXPECT_EQ(output[7].get_value(), 0xcccd9504ULL);

    Prover prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

HEAVY_TEST(stdlib_sha256, test_NIST_vector_five)
{
    typedef plonk::stdlib::field_t<waffle::PLookupComposer> field_pt;
    typedef plonk::stdlib::packed_byte_array<waffle::PLookupComposer> packed_byte_array_pt;

    waffle::PLookupComposer composer = waffle::PLookupComposer();

    packed_byte_array_pt input(
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

    packed_byte_array_pt output_bits = plonk::stdlib::sha256<waffle::PLookupComposer>(input);

    std::vector<field_pt> output = output_bits.to_unverified_byte_slices(4);

    EXPECT_EQ(output[0].get_value(), 0xc2e68682ULL);
    EXPECT_EQ(output[1].get_value(), 0x3489ced2ULL);
    EXPECT_EQ(output[2].get_value(), 0x017f6059ULL);
    EXPECT_EQ(output[3].get_value(), 0xb8b23931ULL);
    EXPECT_EQ(output[4].get_value(), 0x8b6364f6ULL);
    EXPECT_EQ(output[5].get_value(), 0xdcd835d0ULL);
    EXPECT_EQ(output[6].get_value(), 0xa519105aULL);
    EXPECT_EQ(output[7].get_value(), 0x1eadd6e4ULL);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace test_stdlib_sha256