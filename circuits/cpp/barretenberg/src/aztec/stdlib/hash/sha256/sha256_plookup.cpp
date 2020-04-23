#include "sha256_plookup.hpp"
#include <plonk/composer/plookup_tables.hpp>
#include <plonk/composer/plookup_composer.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>

using namespace barretenberg;

namespace plonk {
namespace stdlib {
namespace internal {

constexpr size_t get_num_blocks(const size_t num_bits)
{
    constexpr size_t extra_bits = 65UL;

    return ((num_bits + extra_bits) / 512UL) + ((num_bits + extra_bits) % 512UL > 0);
}
} // namespace internal

template <uint64_t base, uint64_t num_bits>
field_t<waffle::PLookupComposer> normalize_sparse_form(const field_t<waffle::PLookupComposer>& input,
                                                       waffle::LookupTableId table_id)
{
    waffle::PLookupComposer* ctx = input.get_context();

    uint256_t sparse(input.get_value());

    bool is_constant = input.witness_index == UINT32_MAX;

    if (is_constant) {
        if (table_id == waffle::LookupTableId::SHA256_PARTA_NORMALIZE) {
            fr output = waffle::sha256_tables::get_sha256_part_a_output_values_from_key(sparse);
            return field_t<waffle::PLookupComposer>(ctx, output);
        } else if (table_id == waffle::LookupTableId::SHA256_PARTB_NORMALIZE) {
            fr output = waffle::sha256_tables::get_sha256_part_b_output_values_from_key(sparse);
            return field_t<waffle::PLookupComposer>(ctx, output);
        } else {
            uint64_t output = waffle::sha256_tables::map_from_sparse_form<base>(sparse);
            return field_t<waffle::PLookupComposer>(ctx, output);
        }
    }

    uint64_t base_product = 1;
    uint64_t binary_product = 1 << num_bits;
    for (size_t i = 0; i < num_bits; ++i) {
        base_product *= base;
    }
    const uint256_t slice_maximum(base_product);

    constexpr size_t num_slices = (32 / num_bits) + ((num_bits % num_bits) == 0);
    std::array<field_t<waffle::PLookupComposer>, num_slices> input_slices;
    for (auto& slice : input_slices) {
        uint64_t witness = (sparse % slice_maximum).data[0];
        slice = witness_t<waffle::PLookupComposer>(ctx, barretenberg::fr(witness));
        sparse /= slice_maximum;
    }

    std::array<field_t<waffle::PLookupComposer>, num_slices> output_slices;
    for (size_t i = 0; i < num_slices; ++i) {
        output_slices[i] = field_t<waffle::PLookupComposer>(ctx);
        output_slices[i].witness_index = ctx->read_from_table(table_id, input_slices[i].witness_index);
    }

    field_t<waffle::PLookupComposer> input_sum = input_slices[0];
    field_t<waffle::PLookupComposer> output_sum = output_slices[0];

    field_t<waffle::PLookupComposer> sparse_base(ctx, base_product);
    field_t<waffle::PLookupComposer> sparse_base_accumulator = sparse_base;

    field_t<waffle::PLookupComposer> base2(ctx, binary_product);
    field_t<waffle::PLookupComposer> base2_accumulator = base2;

    for (size_t i = 1; i < num_slices - 1; i += 2) {
        const auto t1 = sparse_base_accumulator * sparse_base;
        input_sum = input_sum.add_two(input_slices[i] * sparse_base_accumulator, input_slices[i + 1] * t1);
        sparse_base_accumulator = t1 * sparse_base;

        const auto t2 = base2_accumulator * base2;
        output_sum = output_sum.add_two(output_slices[i] * base2_accumulator, output_slices[i + 1] * t2);
        base2_accumulator = t2 * base2;
    }
    if ((num_slices & 1) == 0) {
        input_sum += input_slices[num_slices - 1] * sparse_base_accumulator;
        output_sum += output_slices[num_slices - 1] * base2_accumulator;
    }

    return output_sum;
}

field_t<waffle::PLookupComposer> choose(const sparse_ch_value& e, const sparse_ch_value& f, const sparse_ch_value& g)
{
    const auto t0 = e.sparse.add_two(f.sparse + f.sparse, g.sparse + g.sparse + g.sparse);
    const auto t1 = e.rot6.add_two(e.rot11, e.rot25);

    const auto r0 = normalize_sparse_form<7, 4>(t0, waffle::LookupTableId::SHA256_PARTA_NORMALIZE);
    const auto r1 = normalize_sparse_form<7, 4>(t1, waffle::LookupTableId::SHA256_BASE7_NORMALIZE);

    return r0 + r1;
}

field_t<waffle::PLookupComposer> majority(const sparse_maj_value& a,
                                          const sparse_maj_value& b,
                                          const sparse_maj_value& c)
{
    const auto t0 = a.sparse.add_two(b.sparse, c.sparse);
    const auto t1 = a.rot2.add_two(a.rot13, a.rot22);

    const auto r0 = normalize_sparse_form<4, 6>(t0, waffle::LookupTableId::SHA256_PARTB_NORMALIZE);
    const auto r1 = normalize_sparse_form<4, 6>(t1, waffle::LookupTableId::SHA256_BASE4_NORMALIZE);

    return r0 + r1;
}

sparse_maj_value convert_into_sparse_maj_form(const field_t<waffle::PLookupComposer>& a)
{
    waffle::PLookupComposer* ctx = a.get_context();

    sparse_maj_value result;

    const uint64_t input_full = uint256_t(a.get_value()).data[0];

    // TODO: USE RANGE PROOF TO CONSTRAIN INPUT
    const uint64_t input = (input_full & 0xffffffffUL);

    const uint64_t slice_maximum = (1 << 11) - 1;
    const uint64_t slice_values[3]{
        input & slice_maximum,
        (input >> 11) & slice_maximum,
        (input >> 22) & slice_maximum,
    };

    if (a.witness_index == UINT32_MAX) {
        result.normal = field_t<waffle::PLookupComposer>(ctx, input);
        result.sparse =
            field_t<waffle::PLookupComposer>(ctx, fr(waffle::sha256_tables::map_into_sparse_form<4>(input)));
        result.rot2 = field_t<waffle::PLookupComposer>(
            ctx, fr(waffle::sha256_tables::map_into_sparse_form<4>(numeric::rotate32((uint32_t)input, 2))));
        result.rot13 = field_t<waffle::PLookupComposer>(
            ctx, fr(waffle::sha256_tables::map_into_sparse_form<4>(numeric::rotate32((uint32_t)input, 13))));
        result.rot22 = field_t<waffle::PLookupComposer>(
            ctx, fr(waffle::sha256_tables::map_into_sparse_form<4>(numeric::rotate32((uint32_t)input, 22))));
        return result;
    }

    std::array<field_t<waffle::PLookupComposer>, 3> input_slices;
    for (size_t i = 0; i < 3; ++i) {
        input_slices[i] = field_t<waffle::PLookupComposer>(ctx);
        input_slices[i].witness_index = ctx->add_variable(barretenberg::fr(slice_values[i]));
    }

    std::array<field_t<waffle::PLookupComposer>, 3> s{
        witness_t<waffle::PLookupComposer>(ctx, waffle::sha256_tables::map_into_sparse_form<4>(slice_values[0])),
        witness_t<waffle::PLookupComposer>(ctx, waffle::sha256_tables::map_into_sparse_form<4>(slice_values[1])),
        witness_t<waffle::PLookupComposer>(ctx, waffle::sha256_tables::map_into_sparse_form<4>(slice_values[2])),
    };
    std::array<field_t<waffle::PLookupComposer>, 3> s_rot{
        field_t<waffle::PLookupComposer>(ctx),
        field_t<waffle::PLookupComposer>(ctx),
        field_t<waffle::PLookupComposer>(ctx),
    };

    const std::array<uint32_t, 3> slice_indices{
        ctx->read_from_table(
            waffle::LookupTableId::SHA256_BASE4_ROTATE2, input_slices[0].witness_index, s[0].witness_index),
        ctx->read_from_table(
            waffle::LookupTableId::SHA256_BASE4_ROTATE2, input_slices[1].witness_index, s[1].witness_index),
        ctx->read_from_table(
            waffle::LookupTableId::SHA256_BASE4_ROTATE2, input_slices[2].witness_index, s[2].witness_index),
    };

    s_rot[0].witness_index = slice_indices[0];
    s_rot[1].witness_index = slice_indices[1];
    s_rot[2].witness_index = slice_indices[2];

    constexpr fr limb_1_shift = uint256_t(4).pow(11);
    constexpr fr limb_2_shift = uint256_t(4).pow(22);

    constexpr fr rot2_limb_1_shift = uint256_t(4).pow(11 - 2);
    constexpr fr rot2_limb_2_shift = uint256_t(4).pow(22 - 2);

    constexpr fr rot13_limb_0_shift = uint256_t(4).pow(32 - 11 - 2);
    constexpr fr rot13_limb_2_shift = uint256_t(4).pow(22 - 11 - 2);

    constexpr fr rot22_limb_0_shift = uint256_t(4).pow(32 - 22);
    constexpr fr rot22_limb_1_shift = uint256_t(4).pow(32 - 22 + 11);

    result.normal = input_slices[0].add_two(input_slices[1] * field_t<waffle::PLookupComposer>(ctx, fr(1 << 11)),
                                            input_slices[2] * field_t<waffle::PLookupComposer>(ctx, fr(1 << 22)));

    // TODO: USE RANGE PROOF TO CONSTRAIN INPUT
    // result.normal.assert_equal(a);

    result.sparse = s[0].add_two(s[1] * limb_1_shift, s[2] * limb_2_shift);

    // a >>> 6 = (s0 + s1 * (2^11-6) + s2 * 2^(22 - 6))
    result.rot2 = s_rot[0].add_two(s[1] * rot2_limb_1_shift, s[2] * rot2_limb_2_shift);

    result.rot13 = s_rot[1].add_two(s[0] * rot13_limb_0_shift, s[2] * rot13_limb_2_shift);

    result.rot22 = s[2].add_two(s[0] * rot22_limb_0_shift, s[1] * rot22_limb_1_shift);

    return result;
}

sparse_ch_value convert_into_sparse_ch_form(const field_t<waffle::PLookupComposer>& e)
{
    waffle::PLookupComposer* ctx = e.get_context();

    sparse_ch_value result;

    const uint64_t input_full = uint256_t(e.get_value()).data[0];

    // TODO: USE NEW RANGE PROOF TO CONSTRAINT INPUT
    const uint64_t input = (input_full & 0xffffffffUL);

    const uint64_t slice_maximum = (1 << 11) - 1;
    const uint64_t slice_values[3]{
        input & slice_maximum,
        (input >> 11) & slice_maximum,
        (input >> 22) & slice_maximum,
    };

    if (e.witness_index == UINT32_MAX) {
        result.normal = field_t<waffle::PLookupComposer>(ctx, input);
        result.sparse =
            field_t<waffle::PLookupComposer>(ctx, fr(waffle::sha256_tables::map_into_sparse_form<7>(input)));
        result.rot6 = field_t<waffle::PLookupComposer>(
            ctx, fr(waffle::sha256_tables::map_into_sparse_form<7>(numeric::rotate32((uint32_t)input, 6))));
        result.rot11 = field_t<waffle::PLookupComposer>(
            ctx, fr(waffle::sha256_tables::map_into_sparse_form<7>(numeric::rotate32((uint32_t)input, 11))));
        result.rot25 = field_t<waffle::PLookupComposer>(
            ctx, fr(waffle::sha256_tables::map_into_sparse_form<7>(numeric::rotate32((uint32_t)input, 25))));
        return result;
    }

    std::array<field_t<waffle::PLookupComposer>, 3> input_slices;
    for (size_t i = 0; i < 3; ++i) {
        input_slices[i] = field_t<waffle::PLookupComposer>(ctx);
        input_slices[i].witness_index = ctx->add_variable(barretenberg::fr(slice_values[i]));
    }

    std::array<field_t<waffle::PLookupComposer>, 3> s{
        witness_t<waffle::PLookupComposer>(ctx, waffle::sha256_tables::map_into_sparse_form<7>(slice_values[0])),
        witness_t<waffle::PLookupComposer>(ctx, waffle::sha256_tables::map_into_sparse_form<7>(slice_values[1])),
        witness_t<waffle::PLookupComposer>(ctx, waffle::sha256_tables::map_into_sparse_form<7>(slice_values[2])),
    };

    std::array<field_t<waffle::PLookupComposer>, 3> s_rot{
        field_t<waffle::PLookupComposer>(ctx),
        field_t<waffle::PLookupComposer>(ctx),
        field_t<waffle::PLookupComposer>(ctx),
    };

    const std::array<uint32_t, 3> slice_indices{
        ctx->read_from_table(
            waffle::LookupTableId::SHA256_BASE7_ROTATE6, input_slices[0].witness_index, s[0].witness_index),
        ctx->read_from_table(
            waffle::LookupTableId::SHA256_BASE7_ROTATE6, input_slices[1].witness_index, s[1].witness_index),
        ctx->read_from_table(
            waffle::LookupTableId::SHA256_BASE7_ROTATE3, input_slices[2].witness_index, s[2].witness_index),
    };

    s_rot[0].witness_index = slice_indices[0];
    s_rot[1].witness_index = slice_indices[1];
    s_rot[2].witness_index = slice_indices[2];

    constexpr fr limb_1_shift = uint256_t(7).pow(11);
    constexpr fr limb_2_shift = uint256_t(7).pow(22);

    constexpr fr rot6_limb_1_shift = uint256_t(7).pow(11 - 6);
    constexpr fr rot6_limb_2_shift = uint256_t(7).pow(22 - 6);

    constexpr fr rot11_limb_0_shift = uint256_t(7).pow(32 - 11);
    constexpr fr rot11_limb_2_shift = uint256_t(7).pow(22 - 11);

    constexpr fr rot25_limb_0_shift = uint256_t(7).pow(32 - 25);
    constexpr fr rot25_limb_1_shift = uint256_t(7).pow(32 - 25 + 11);

    result.normal = input_slices[0].add_two(input_slices[1] * field_t<waffle::PLookupComposer>(ctx, fr(1 << 11)),
                                            input_slices[2] * field_t<waffle::PLookupComposer>(ctx, fr(1 << 22)));

    // TODO: USE RANGE PROOF TO CONSTRAIN INPUT
    // result.normal.assert_equal(e);

    result.sparse = s[0].add_two(s[1] * limb_1_shift, s[2] * limb_2_shift);

    // a >>> 6 = (s0 + s1 * (2^11-6) + s2 * 2^(22 - 6))
    result.rot6 = s_rot[0].add_two(s[1] * rot6_limb_1_shift, s[2] * rot6_limb_2_shift);

    result.rot11 = s[1].add_two(s[0] * rot11_limb_0_shift, s[2] * rot11_limb_2_shift);

    result.rot25 = s_rot[2].add_two(s[0] * rot25_limb_0_shift, s[1] * rot25_limb_1_shift);

    return result;
}

std::array<field_t<waffle::PLookupComposer>, 8> sha256_inner_block(
    const std::array<field_t<waffle::PLookupComposer>, 64>& w)
{
    typedef field_t<waffle::PLookupComposer> field_t;

    constexpr uint64_t init_constants[8]{ 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                                          0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19 };

    constexpr uint64_t round_constants[64]{
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    };
    /**
     * Initialize round variables with previous block output
     **/
    auto a = convert_into_sparse_maj_form(fr(init_constants[0]));
    auto b = convert_into_sparse_maj_form(fr(init_constants[1]));
    auto c = convert_into_sparse_maj_form(fr(init_constants[2]));
    auto d = convert_into_sparse_maj_form(fr(init_constants[3]));
    auto e = convert_into_sparse_ch_form(fr(init_constants[4]));
    auto f = convert_into_sparse_ch_form(fr(init_constants[5]));
    auto g = convert_into_sparse_ch_form(fr(init_constants[6]));
    auto h = convert_into_sparse_ch_form(fr(init_constants[7]));

    /**
     * Apply SHA-256 compression function to the message schedule
     **/
    for (size_t i = 0; i < 64; ++i) {
        auto ch = choose(e, f, g);
        auto maj = majority(a, b, c);
        auto temp1 = h.normal.add_two(ch, w[i] + fr(round_constants[i]));

        h = g;
        g = f;
        f = e;
        e = convert_into_sparse_ch_form(d.normal + temp1);
        d = c;
        c = b;
        b = a;
        a = convert_into_sparse_maj_form(temp1 + maj);
    }

    /**
     * Add into previous block output and return
     **/
    std::array<field_t, 8> output;
    output[0] = a.normal + fr(init_constants[0]);
    output[1] = b.normal + fr(init_constants[1]);
    output[2] = c.normal + fr(init_constants[2]);
    output[3] = d.normal + fr(init_constants[3]);
    output[4] = e.normal + fr(init_constants[4]);
    output[5] = f.normal + fr(init_constants[5]);
    output[6] = g.normal + fr(init_constants[6]);
    output[7] = h.normal + fr(init_constants[7]);
    return output;
}

// std::array<uint32<waffle::PLookupComposer>, 8> sha256_block(const std::array<uint32<waffle::PLookupComposer>, 8>&
// h_init,
//                                                     const std::array<uint32<waffle::PLookupComposer>, 16>& input)
// {
//     typedef uint32<waffle::PLookupComposer> uint32;
//     std::array<uint32, 64> w;

//     /**
//      * Fill first 16 words with the message schedule
//      **/
//     for (size_t i = 0; i < 16; ++i) {
//         w[i] = input[i];
//     }

//     /**
//      * Extend the input data into the remaining 48 words
//      **/
//     for (size_t i = 16; i < 64; ++i) {
//         uint32 s0 = w[i - 15].ror(7) ^ w[i - 15].ror(18) ^ (w[i - 15] >> 3);
//         uint32 s1 = w[i - 2].ror(17) ^ w[i - 2].ror(19) ^ (w[i - 2] >> 10);
//         w[i] = w[i - 16] + w[i - 7] + s0 + s1;
//     }

//     /**
//      * Initialize round variables with previous block output
//      **/
//     uint32 a = h_init[0];
//     uint32 b = h_init[1];
//     uint32 c = h_init[2];
//     uint32 d = h_init[3];
//     uint32 e = h_init[4];
//     uint32 f = h_init[5];
//     uint32 g = h_init[6];
//     uint32 h = h_init[7];

//     /**
//      * Apply SHA-256 compression function to the message schedule
//      **/
//     for (size_t i = 0; i < 64; ++i) {
//         uint32 S1 = e.ror(6U) ^ e.ror(11U) ^ e.ror(25U);
//         uint32 ch = (e & f) + (~e & g); // === (e & f) ^ (~e & g), `+` op is cheaper
//         uint32 temp1 = h + S1 + ch + internal::round_constants[i] + w[i];
//         uint32 S0 = a.ror(2U) ^ a.ror(13U) ^ a.ror(22U);
//         uint32 T0 = (b & c);
//         uint32 maj = (a & (b + c - (T0 + T0))) + T0; // === (a & b) ^ (a & c) ^ (b & c)
//         uint32 temp2 = S0 + maj;

//         h = g;
//         g = f;
//         f = e;
//         e = d + temp1;
//         d = c;
//         c = b;
//         b = a;
//         a = temp1 + temp2;
//     }

//     /**
//      * Add into previous block output and return
//      **/
//     std::array<uint32, 8> output;
//     output[0] = a + h_init[0];
//     output[1] = b + h_init[1];
//     output[2] = c + h_init[2];
//     output[3] = d + h_init[3];
//     output[4] = e + h_init[4];
//     output[5] = f + h_init[5];
//     output[6] = g + h_init[6];
//     output[7] = h + h_init[7];
//     return output;
// }

// byte_array<waffle::PLookupComposer> sha256_block(const byte_array<waffle::PLookupComposer>& input)
// {
//     typedef uint32<waffle::PLookupComposer> uint32;

//     ASSERT(input.size() == 64);

//     std::array<uint32, 8> hash;
//     prepare_constants(hash);

//     std::array<uint32, 16> hash_input;
//     for (size_t i = 0; i < 16; ++i) {
//         hash_input[i] = uint32(input.slice(i * 4, 4));
//     }
//     hash = sha256_block(hash, hash_input);

//     byte_array<waffle::PLookupComposer> result(input.get_context());
//     for (size_t i = 0; i < 8; ++i) {
//         result.write(static_cast<byte_array<waffle::PLookupComposer>>(hash[i]));
//     }

//     return result;
// }

// bit_array<waffle::PLookupComposer> sha256(const bit_array<waffle::PLookupComposer>& input)
// {
//     typedef uint32<waffle::PLookupComposer> uint32;
//     typedef bit_array<waffle::PLookupComposer> bit_array;

//     size_t num_bits = input.size();
//     size_t num_blocks = internal::get_num_blocks(num_bits);

//     bit_array message_schedule = bit_array(input.get_context(), num_blocks * 512UL);

//     // begin filling message schedule from most significant to least significant
//     size_t offset = message_schedule.size() - input.size();

//     for (size_t i = input.size() - 1; i < input.size(); --i) {
//         size_t idx = offset + i;
//         message_schedule[idx] = input[i];
//     }
//     message_schedule[offset - 1] = true;
//     for (size_t i = 0; i < 32; ++i) {
//         message_schedule[i] = static_cast<bool>((num_bits >> i) & 1);
//     }

//     std::array<uint32, 8> rolling_hash;
//     prepare_constants(rolling_hash);
//     for (size_t i = 0; i < num_blocks; ++i) {
//         std::array<uint32, 16> hash_input;
//         message_schedule.populate_uint32_array(i * 512, hash_input);
//         rolling_hash = sha256_block(rolling_hash, hash_input);
//     }
//     return bit_array(rolling_hash);
// }

// template byte_array<waffle::TurboComposer> sha256_block(const byte_array<waffle::TurboComposer>& input);
// template bit_array<waffle::StandardComposer> sha256(const bit_array<waffle::StandardComposer>& input);
// template bit_array<waffle::TurboComposer> sha256(const bit_array<waffle::TurboComposer>& input);

} // namespace stdlib
} // namespace plonk
