#include "sha256.hpp"
#include "sha256_plookup.hpp"
#include <gtest/gtest.h>
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/plookup_tables.hpp>
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

TEST(stdlib_sha256_plookup, convert_into_sparse_ch_form)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t input = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> element;
        if (i == 0) {
            element = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, input);
        } else {
            element = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, input);
        }
        const auto converted = plonk::stdlib::convert_into_sparse_ch_form(element);

        const uint64_t result_normal = uint256_t(converted.normal.get_value()).data[0];
        const uint64_t result_sparse = numeric::map_from_sparse_form<7>(converted.sparse.get_value());

        const uint64_t result_rot6 = numeric::map_from_sparse_form<7>(converted.rot6.get_value());
        const uint64_t result_rot11 = numeric::map_from_sparse_form<7>(converted.rot11.get_value());
        const uint64_t result_rot25 = numeric::map_from_sparse_form<7>(converted.rot25.get_value());

        EXPECT_EQ(input, result_normal);
        EXPECT_EQ(input, result_sparse);
        EXPECT_EQ(numeric::rotate32((uint32_t)input, 6), result_rot6);
        EXPECT_EQ(numeric::rotate32((uint32_t)input, 11), result_rot11);
        EXPECT_EQ(numeric::rotate32((uint32_t)input, 25), result_rot25);
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, convert_into_sparse_maj_form)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t input = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> element;
        if (i == 0) {
            element = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, input);
        } else {
            element = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, input);
        }

        const auto converted = plonk::stdlib::convert_into_sparse_maj_form(element);

        const uint64_t result_normal = uint256_t(converted.normal.get_value()).data[0];
        const uint64_t result_sparse = numeric::map_from_sparse_form<4>(converted.sparse.get_value());

        const uint64_t result_rot2 = numeric::map_from_sparse_form<4>(converted.rot2.get_value());
        const uint64_t result_rot13 = numeric::map_from_sparse_form<4>(converted.rot13.get_value());
        const uint64_t result_rot22 = numeric::map_from_sparse_form<4>(converted.rot22.get_value());

        EXPECT_EQ(input, result_normal);
        EXPECT_EQ(input, result_sparse);
        EXPECT_EQ(numeric::rotate32((uint32_t)input, 2), result_rot2);
        EXPECT_EQ(numeric::rotate32((uint32_t)input, 13), result_rot13);
        EXPECT_EQ(numeric::rotate32((uint32_t)input, 22), result_rot22);
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, normalize_sparse_form_base7)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t in_e = engine.get_random_uint32();
        uint64_t in_f = engine.get_random_uint32();
        uint64_t in_g = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> e;
        plonk::stdlib::field_t<waffle::PLookupComposer> f;
        plonk::stdlib::field_t<waffle::PLookupComposer> g;
        if (i == 0) {
            e = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_g);
        } else {
            e = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_g);
        }

        const auto e_ch = plonk::stdlib::convert_into_sparse_ch_form(e);
        const auto f_ch = plonk::stdlib::convert_into_sparse_ch_form(f);
        const auto g_ch = plonk::stdlib::convert_into_sparse_ch_form(g);

        const auto sparse_sum = e_ch.rot6 + f_ch.rot11 + g_ch.rot25;

        const uint64_t expected = numeric::rotate32((uint32_t)in_e, 6) ^ numeric::rotate32((uint32_t)in_f, 11) ^
                                  numeric::rotate32((uint32_t)in_g, 25);

        const auto result = plonk::stdlib::normalize_sparse_form<7, 4>(sparse_sum.normalize(),
                                                                       waffle::PLookupTableId::SHA256_BASE7_NORMALIZE);

        // const uint256_t expected(in_e ^ in_f ^ in_g);

        EXPECT_EQ(uint256_t(result.get_value()), uint256_t(expected));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, normalize_sparse_form_parta)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t in_e = engine.get_random_uint32();
        uint64_t in_f = engine.get_random_uint32();
        uint64_t in_g = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> e;
        plonk::stdlib::field_t<waffle::PLookupComposer> f;
        plonk::stdlib::field_t<waffle::PLookupComposer> g;
        if (i == 0) {
            e = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_g);
        } else {
            e = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_g);
        }

        const auto e_ch = plonk::stdlib::convert_into_sparse_ch_form(e);
        const auto f_ch = plonk::stdlib::convert_into_sparse_ch_form(f);
        const auto g_ch = plonk::stdlib::convert_into_sparse_ch_form(g);

        auto sparse_sum = e_ch.sparse + f_ch.sparse + f_ch.sparse + g_ch.sparse + g_ch.sparse + g_ch.sparse;
        sparse_sum = sparse_sum.normalize();
        const uint256_t native_sparse[3]{
            numeric::map_into_sparse_form<7>(in_e),
            numeric::map_into_sparse_form<7>(in_f),
            numeric::map_into_sparse_form<7>(in_g),
        };

        auto native_add = native_sparse[0] + native_sparse[1] + native_sparse[1] + native_sparse[2] + native_sparse[2] +
                          native_sparse[2];

        const uint64_t bit_table[7]{
            0, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
            0, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
            0, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
            1, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
            0, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
            1, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
            1, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
        };

        const uint64_t expected = (in_e & in_f) ^ (~in_e & in_g);

        for (size_t i = 0; i < 7; ++i) {
            const auto foo = native_add % uint256_t(7);
            const auto bar = bit_table[foo.data[0]];
            native_add = (native_add - foo) / 7;
            EXPECT_EQ(bar, ((expected >> i) & 1));
        }

        const auto result =
            plonk::stdlib::normalize_sparse_form<7, 4>(sparse_sum, waffle::PLookupTableId::SHA256_PARTA_NORMALIZE);

        // const uint256_t expected(in_e ^ in_f ^ in_g);

        EXPECT_EQ(uint256_t(result.get_value()), uint256_t(expected));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, normalize_sparse_form_partb)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t in_e = engine.get_random_uint32();
        uint64_t in_f = engine.get_random_uint32();
        uint64_t in_g = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> e;
        plonk::stdlib::field_t<waffle::PLookupComposer> f;
        plonk::stdlib::field_t<waffle::PLookupComposer> g;
        if (i == 0) {
            e = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_g);
        } else {
            e = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_g);
        }

        const auto e_maj = plonk::stdlib::convert_into_sparse_maj_form(e);
        const auto f_maj = plonk::stdlib::convert_into_sparse_maj_form(f);
        const auto g_maj = plonk::stdlib::convert_into_sparse_maj_form(g);

        auto sparse_sum = e_maj.sparse + f_maj.sparse + g_maj.sparse;
        sparse_sum = sparse_sum.normalize();
        const uint256_t native_sparse[3]{
            numeric::map_into_sparse_form<4>(in_e),
            numeric::map_into_sparse_form<4>(in_f),
            numeric::map_into_sparse_form<4>(in_g),
        };

        EXPECT_EQ(e_maj.sparse.get_value(), native_sparse[0]);
        EXPECT_EQ(f_maj.sparse.get_value(), native_sparse[1]);
        EXPECT_EQ(g_maj.sparse.get_value(), native_sparse[2]);

        auto native_add = native_sparse[0] + native_sparse[1] + native_sparse[2];

        constexpr uint64_t bit_table[4]{
            0, // a + b + c = 0 => (a & b) ^ (a & c) ^ (b & c) = 0
            0, // a + b + c = 1 => (a & b) ^ (a & c) ^ (b & c) = 0
            1, // a + b + c = 2 => (a & b) ^ (a & c) ^ (b & c) = 1
            1, // a + b + c = 3 => (a & b) ^ (a & c) ^ (b & c) = 0
        };

        const uint64_t expected = (in_e & in_f) ^ (in_e & in_g) ^ (in_f & in_g);

        for (size_t i = 0; i < 16; ++i) {
            const auto foo = native_add % uint256_t(4);
            const auto bar = bit_table[foo.data[0]];
            native_add = (native_add - foo) / 4;
            EXPECT_EQ(bar, ((expected >> i) & 1));
        }

        const auto result =
            plonk::stdlib::normalize_sparse_form<4, 6>(sparse_sum, waffle::PLookupTableId::SHA256_PARTB_NORMALIZE);

        // const uint256_t expected(in_e ^ in_f ^ in_g);

        EXPECT_EQ(uint256_t(result.get_value()), uint256_t(expected));
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, choose)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t in_e = engine.get_random_uint32();
        uint64_t in_f = engine.get_random_uint32();
        uint64_t in_g = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> e;
        plonk::stdlib::field_t<waffle::PLookupComposer> f;
        plonk::stdlib::field_t<waffle::PLookupComposer> g;
        if (i == 0) {
            e = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_g);
        } else {
            e = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_g);
        }

        const auto e_ch = plonk::stdlib::convert_into_sparse_ch_form(e);
        const auto f_ch = plonk::stdlib::convert_into_sparse_ch_form(f);
        const auto g_ch = plonk::stdlib::convert_into_sparse_ch_form(g);

        const auto result = plonk::stdlib::choose(e_ch, f_ch, g_ch);

        const uint64_t t0 = numeric::rotate32((uint32_t)in_e, 6) ^ numeric::rotate32((uint32_t)in_e, 11) ^
                            numeric::rotate32((uint32_t)in_e, 25);
        const uint64_t t1 = (in_e & in_f) ^ (~in_e & in_g);
        const uint256_t expected(t0 + t1);

        EXPECT_EQ(uint256_t(result.get_value()), expected);
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, majority)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 2; ++i) {
        uint64_t in_e = engine.get_random_uint32();
        uint64_t in_f = engine.get_random_uint32();
        uint64_t in_g = engine.get_random_uint32();
        plonk::stdlib::field_t<waffle::PLookupComposer> e;
        plonk::stdlib::field_t<waffle::PLookupComposer> f;
        plonk::stdlib::field_t<waffle::PLookupComposer> g;
        if (i == 0) {
            e = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::field_t<waffle::PLookupComposer>(&composer, in_g);
        } else {
            e = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_e);
            f = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_f);
            g = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, in_g);
        }

        const auto e_maj = plonk::stdlib::convert_into_sparse_maj_form(e);
        const auto f_maj = plonk::stdlib::convert_into_sparse_maj_form(f);
        const auto g_maj = plonk::stdlib::convert_into_sparse_maj_form(g);

        const auto result = plonk::stdlib::majority(e_maj, f_maj, g_maj);

        const uint64_t t0 = numeric::rotate32((uint32_t)in_e, 2) ^ numeric::rotate32((uint32_t)in_e, 13) ^
                            numeric::rotate32((uint32_t)in_e, 22);
        const uint64_t t1 = (in_e & in_f) ^ (in_e & in_g) ^ (in_f & in_g);
        const uint256_t expected(t0 + t1);

        EXPECT_EQ(uint256_t(result.get_value()), expected);
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_sha256_plookup, test_round)
{

    waffle::PLookupComposer composer = waffle::PLookupComposer();

    std::array<uint64_t, 64> w_inputs;
    std::array<plonk::stdlib::field_t<waffle::PLookupComposer>, 64> w_elements;

    for (size_t i = 0; i < 64; ++i) {
        w_inputs[i] = engine.get_random_uint32();
        w_elements[i] = plonk::stdlib::witness_t<waffle::PLookupComposer>(&composer, barretenberg::fr(w_inputs[i]));
    }

    const auto expected = inner_block(w_inputs);

    const std::array<plonk::stdlib::field_t<waffle::PLookupComposer>, 8> result =
        plonk::stdlib::sha256_inner_block(w_elements);
    for (size_t i = 0; i < 8; ++i) {
        EXPECT_EQ(uint256_t(result[i].get_value()).data[0] & 0xffffffffUL,
                  uint256_t(expected[i]).data[0] & 0xffffffffUL);
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

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

TEST(stdlib_sha256, test_NIST_vector_five)
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