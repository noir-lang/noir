#include "./aes128.hpp"

#include <plonk/composer/plookup_composer.hpp>

#include <numeric/uint256/uint256.hpp>
#include <numeric/bitop/sparse_form.hpp>
#include <crypto/aes128/aes128.hpp>

#include <stdlib/primitives/plookup/plookup.hpp>

using namespace crypto::aes128;
using namespace barretenberg;

namespace plonk {
namespace stdlib {
namespace aes128 {

constexpr uint32_t AES128_BASE = 9;
typedef plonk::stdlib::field_t<waffle::PlookupComposer> field_t;
typedef plonk::stdlib::witness_t<waffle::PlookupComposer> witness_t;

typedef std::pair<field_t, field_t> byte_pair;

field_t normalize_sparse_form(waffle::PlookupComposer*, field_t& byte)
{
    byte = byte.normalize();
    auto result = plookup::read_from_1_to_2_table(waffle::AES_NORMALIZE, byte);
    return result;
}

byte_pair apply_aes_sbox_map(waffle::PlookupComposer*, field_t& input)
{
    input = input.normalize();

    return plookup::read_pair_from_table(waffle::AES_SBOX, input);
}

std::array<field_t, 16> convert_into_sparse_bytes(waffle::PlookupComposer*, const field_t& block_data)
{
    // `block_data` must be a 128 bit variable
    std::array<field_t, 16> sparse_bytes;

    auto sequence = plookup::read_sequence_from_table(waffle::AES_INPUT, block_data);

    for (size_t i = 0; i < 16; ++i) {
        sparse_bytes[15 - i] = sequence[1][i];
    }

    return sparse_bytes;
}

field_t convert_from_sparse_bytes(waffle::PlookupComposer* ctx, field_t* sparse_bytes)
{
    std::array<field_t, 16> bytes;

    uint256_t accumulator = 0;
    for (size_t i = 0; i < 16; ++i) {
        uint64_t sparse_byte = uint256_t(sparse_bytes[i].get_value()).data[0];
        uint256_t byte = numeric::map_from_sparse_form<AES128_BASE>(sparse_byte);
        accumulator <<= 8;
        accumulator += (byte);
    }

    field_t result = witness_t(ctx, fr(accumulator));

    const auto indices = plookup::read_sequence_from_table(waffle::AES_INPUT, result);

    for (size_t i = 0; i < 16; ++i) {
        ctx->assert_equal(sparse_bytes[15 - i].witness_index, indices[1][i].witness_index);
    }

    return result;
}

std::array<field_t, 176> expand_key(waffle::PlookupComposer* ctx, const field_t& key)
{
    constexpr uint8_t round_constants[11] = { 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36 };
    std::array<field_t, 11> sparse_round_constants{
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[0]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[1]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[2]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[3]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[4]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[5]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[6]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[7]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[8]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[9]))),
        field_t(ctx, fr(numeric::map_into_sparse_form<AES128_BASE>(round_constants[10]))),
    };

    std::array<field_t, 176> round_key{};
    const auto sparse_key = convert_into_sparse_bytes(ctx, key);

    field_t temp[4]{};
    uint64_t temp_add_counts[4]{};
    uint64_t add_counts[176]{};
    for (size_t i = 0; i < 176; ++i) {
        add_counts[i] = 1;
    }
    for (size_t i = 0; i < 16; ++i) {
        round_key[i] = sparse_key[i];
    }

    for (size_t i = 4; i < 44; ++i) {
        size_t k = (i - 1) * 4;

        temp_add_counts[0] = add_counts[k + 0];
        temp_add_counts[1] = add_counts[k + 1];
        temp_add_counts[2] = add_counts[k + 2];
        temp_add_counts[3] = add_counts[k + 3];

        temp[0] = round_key[k];
        temp[1] = round_key[k + 1];
        temp[2] = round_key[k + 2];
        temp[3] = round_key[k + 3];

        if ((i & 0x03) == 0) {
            const auto t = temp[0];
            temp[0] = temp[1];
            temp[1] = temp[2];
            temp[2] = temp[3];
            temp[3] = t;

            temp[0] = apply_aes_sbox_map(ctx, temp[0]).first;
            temp[1] = apply_aes_sbox_map(ctx, temp[1]).first;
            temp[2] = apply_aes_sbox_map(ctx, temp[2]).first;
            temp[3] = apply_aes_sbox_map(ctx, temp[3]).first;

            temp[0] = temp[0] + sparse_round_constants[i >> 2];
            temp[0] = temp[0].normalize();
            ++temp_add_counts[0];
        }

        size_t j = i * 4;
        k = (i - 4) * 4;
        round_key[j] = round_key[k] + temp[0];
        round_key[j + 1] = round_key[k + 1] + temp[1];
        round_key[j + 2] = round_key[k + 2] + temp[2];
        round_key[j + 3] = round_key[k + 3] + temp[3];

        add_counts[j] = add_counts[k] + temp_add_counts[0];
        add_counts[j + 1] = add_counts[k + 1] + temp_add_counts[1];
        add_counts[j + 2] = add_counts[k + 2] + temp_add_counts[2];
        add_counts[j + 3] = add_counts[k + 3] + temp_add_counts[3];

        constexpr uint64_t target = 3;
        if (add_counts[j] > target || (add_counts[j] > 1 && (j & 12) == 12)) {
            round_key[j] = normalize_sparse_form(ctx, round_key[j]);
            add_counts[j] = 1;
        }
        if (add_counts[j + 1] > target || (add_counts[j + 1] > 1 && ((j + 1) & 12) == 12)) {
            round_key[j + 1] = normalize_sparse_form(ctx, round_key[j + 1]);
            add_counts[j + 1] = 1;
        }
        if (add_counts[j + 2] > target || (add_counts[j + 2] > 1 && ((j + 2) & 12) == 12)) {
            round_key[j + 2] = normalize_sparse_form(ctx, round_key[j + 2]);
            add_counts[j + 2] = 1;
        }
        if (add_counts[j + 3] > target || (add_counts[j + 3] > 1 && ((j + 3) & 12) == 12)) {
            round_key[j + 3] = normalize_sparse_form(ctx, round_key[j + 3]);
            add_counts[j + 3] = 1;
        }
    }

    return round_key;
}

void shift_rows(byte_pair* state)
{
    byte_pair temp = state[1];
    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = temp;

    temp = state[2];
    state[2] = state[10];
    state[10] = temp;
    temp = state[6];
    state[6] = state[14];
    state[14] = temp;

    temp = state[3];
    state[3] = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = temp;
}

void mix_column_and_add_round_key(byte_pair* column_pairs, field_t* round_key, uint64_t round)
{

    auto t0 = column_pairs[0].first.add_two(column_pairs[3].first, column_pairs[1].second);
    auto t1 = column_pairs[1].first.add_two(column_pairs[2].first, column_pairs[3].second);

    auto r0 = t0.add_two(column_pairs[2].first, column_pairs[0].second);
    auto r1 = t0.add_two(column_pairs[1].first, column_pairs[2].second);
    auto r2 = t1.add_two(column_pairs[0].first, column_pairs[2].second);
    auto r3 = t1.add_two(column_pairs[0].second, column_pairs[3].first);

    column_pairs[0].first = r0 + round_key[(round * 16U)];
    column_pairs[1].first = r1 + round_key[(round * 16U) + 1];
    column_pairs[2].first = r2 + round_key[(round * 16U) + 2];
    column_pairs[3].first = r3 + round_key[(round * 16U) + 3];
}

void mix_columns_and_add_round_key(byte_pair* state_pairs, field_t* round_key, uint64_t round)
{
    mix_column_and_add_round_key(state_pairs, round_key, round);
    mix_column_and_add_round_key(state_pairs + 4, round_key + 4, round);
    mix_column_and_add_round_key(state_pairs + 8, round_key + 8, round);
    mix_column_and_add_round_key(state_pairs + 12, round_key + 12, round);
}

void sub_bytes(waffle::PlookupComposer* ctx, byte_pair* state_pairs)
{
    for (size_t i = 0; i < 16; ++i) {
        state_pairs[i] = apply_aes_sbox_map(ctx, state_pairs[i].first);
    }
}

void add_round_key(byte_pair* sparse_state, field_t* sparse_round_key, uint64_t round)
{
    for (size_t i = 0; i < 16; i += 4) {
        for (size_t j = 0; j < 4; ++j) {
            sparse_state[i + j].first += sparse_round_key[(round * 16U) + i + j];
        }
    }
}

void xor_with_iv(byte_pair* state, field_t* iv)
{
    for (size_t i = 0; i < 16; ++i) {
        state[i].first += iv[i];
    }
}

void aes128_cipher(waffle::PlookupComposer* ctx, byte_pair* state, field_t* sparse_round_key)
{
    add_round_key(state, sparse_round_key, 0);
    for (size_t i = 0; i < 16; ++i) {
        state[i].first = normalize_sparse_form(ctx, state[i].first);
    }

    for (size_t round = 1; round < 10; ++round) {
        sub_bytes(ctx, state);
        shift_rows(state);
        mix_columns_and_add_round_key(state, sparse_round_key, round);
        for (size_t i = 0; i < 16; ++i) {
            state[i].first = normalize_sparse_form(ctx, state[i].first);
        }
    }

    sub_bytes(ctx, state);
    shift_rows(state);
    add_round_key(state, sparse_round_key, 10);
}

std::vector<field_t> encrypt_buffer_cbc(const std::vector<field_t>& input, const field_t& iv, const field_t& key)
{
    waffle::PlookupComposer* ctx = key.get_context();

    auto round_key = expand_key(ctx, key);

    const size_t num_blocks = input.size();

    std::vector<byte_pair> sparse_state;
    for (size_t i = 0; i < num_blocks; ++i) {
        auto bytes = convert_into_sparse_bytes(ctx, input[i]);
        for (const auto& byte : bytes) {
            sparse_state.push_back({ byte, field_t(ctx, fr(0)) });
        }
    }

    auto sparse_iv = convert_into_sparse_bytes(ctx, iv);

    for (size_t i = 0; i < num_blocks; ++i) {
        byte_pair* round_state = &sparse_state[i * 16];
        xor_with_iv(round_state, &sparse_iv[0]);
        aes128_cipher(ctx, round_state, &round_key[0]);

        for (size_t j = 0; j < 16; ++j) {
            sparse_iv[j] = round_state[j].first;
        }
    }

    std::vector<field_t> sparse_output;
    for (auto& element : sparse_state) {
        sparse_output.push_back(normalize_sparse_form(ctx, element.first));
    }

    std::vector<field_t> output;
    for (size_t i = 0; i < num_blocks; ++i) {
        output.push_back(convert_from_sparse_bytes(ctx, &sparse_output[i * 16]));
    }
    return output;
}

} // namespace aes128
} // namespace stdlib
} // namespace plonk
