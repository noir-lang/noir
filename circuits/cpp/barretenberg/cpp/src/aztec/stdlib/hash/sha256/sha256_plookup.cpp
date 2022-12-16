#include "sha256_plookup.hpp"

#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <plonk/composer/plookup_tables/sha256.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/plookup/plookup.hpp>

using namespace barretenberg;

namespace plonk {
namespace stdlib {
namespace sha256_plookup {

namespace internal {

constexpr size_t get_num_blocks(const size_t num_bits)
{
    constexpr size_t extra_bits = 65UL;

    return ((num_bits + extra_bits) / 512UL) + ((num_bits + extra_bits) % 512UL > 0);
}
} // namespace internal

void prepare_constants(std::array<field_t<waffle::UltraComposer>, 8>& input)
{
    constexpr uint64_t init_constants[8]{ 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                                          0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19 };

    input[0] = init_constants[0];
    input[1] = init_constants[1];
    input[2] = init_constants[2];
    input[3] = init_constants[3];
    input[4] = init_constants[4];
    input[5] = init_constants[5];
    input[6] = init_constants[6];
    input[7] = init_constants[7];
}

sparse_witness_limbs convert_witness(const field_t<waffle::UltraComposer>& w)
{
    typedef field_t<waffle::UltraComposer> field_pt;

    sparse_witness_limbs result(w);

    const auto lookup = plookup_read::get_lookup_accumulators(MultiTableId::SHA256_WITNESS_INPUT, w);

    result.sparse_limbs = std::array<field_pt, 4>{
        lookup[ColumnIdx::C2][0],
        lookup[ColumnIdx::C2][1],
        lookup[ColumnIdx::C2][2],
        lookup[ColumnIdx::C2][3],
    };
    result.rotated_limbs = std::array<field_pt, 4>{
        lookup[ColumnIdx::C3][0],
        lookup[ColumnIdx::C3][1],
        lookup[ColumnIdx::C3][2],
        lookup[ColumnIdx::C3][3],
    };
    result.has_sparse_limbs = true;

    return result;
}

std::array<field_t<waffle::UltraComposer>, 64> extend_witness(
    const std::array<field_t<waffle::UltraComposer>, 16>& w_in)
{
    typedef field_t<waffle::UltraComposer> field_pt;

    waffle::UltraComposer* ctx = w_in[0].get_context();

    std::array<sparse_witness_limbs, 64> w_sparse;
    for (size_t i = 0; i < 16; ++i) {
        w_sparse[i] = sparse_witness_limbs(w_in[i]);
        if (!ctx && w_in[i].get_context()) {
            ctx = w_in[i].get_context();
        }
    }

    for (size_t i = 16; i < 64; ++i) {
        auto& w_left = w_sparse[i - 15];
        auto& w_right = w_sparse[i - 2];

        if (!w_left.has_sparse_limbs) {
            w_left = convert_witness(w_left.normal);
        }
        if (!w_right.has_sparse_limbs) {
            w_right = convert_witness(w_right.normal);
        }

        constexpr fr base(16);
        constexpr fr left_multipliers[4]{
            (base.pow(32 - 7) + base.pow(32 - 18)),
            (base.pow(32 - 18 + 3) + 1),
            (base.pow(32 - 18 + 10) + base.pow(10 - 7) + base.pow(10 - 3)),
            (base.pow(18 - 7) + base.pow(18 - 3) + 1),
        };

        constexpr fr right_multipliers[4]{
            base.pow(32 - 17) + base.pow(32 - 19),
            base.pow(32 - 17 + 3) + base.pow(32 - 19 + 3),
            base.pow(32 - 19 + 10) + fr(1),
            base.pow(18 - 17) + base.pow(18 - 10),
        };

        field_pt left[4]{
            w_left.sparse_limbs[0] * left_multipliers[0],
            w_left.sparse_limbs[1] * left_multipliers[1],
            w_left.sparse_limbs[2] * left_multipliers[2],
            w_left.sparse_limbs[3] * left_multipliers[3],
        };

        field_pt right[4]{
            w_right.sparse_limbs[0] * right_multipliers[0],
            w_right.sparse_limbs[1] * right_multipliers[1],
            w_right.sparse_limbs[2] * right_multipliers[2],
            w_right.sparse_limbs[3] * right_multipliers[3],
        };

        const auto left_xor_sparse =
            left[0].add_two(left[1], left[2]).add_two(left[3], w_left.rotated_limbs[1]) * fr(4);

        const auto xor_result_sparse = right[0]
                                           .add_two(right[1], right[2])
                                           .add_two(right[3], w_right.rotated_limbs[2])
                                           .add_two(w_right.rotated_limbs[3], left_xor_sparse)
                                           .normalize();

        field_pt xor_result = plookup_read::read_from_1_to_2_table(SHA256_WITNESS_OUTPUT, xor_result_sparse);

        // TODO NORMALIZE WITH RANGE CHECK

        field_pt w_out_raw = xor_result.add_two(w_sparse[i - 16].normal, w_sparse[i - 7].normal);
        field_pt w_out;
        if (w_out_raw.witness_index == IS_CONSTANT) {
            w_out = field_pt(ctx, fr(w_out_raw.get_value().from_montgomery_form().data[0] & (uint64_t)0xffffffffULL));

        } else {
            w_out = witness_t<waffle::UltraComposer>(
                ctx, fr(w_out_raw.get_value().from_montgomery_form().data[0] & (uint64_t)0xffffffffULL));
        }
        w_sparse[i] = sparse_witness_limbs(w_out);
    }

    std::array<field_pt, 64> w_extended;

    for (size_t i = 0; i < 64; ++i) {
        w_extended[i] = w_sparse[i].normal;
    }
    return w_extended;
}

sparse_value map_into_choose_sparse_form(const field_t<waffle::UltraComposer>& e)
{
    sparse_value result;
    result.normal = e;
    result.sparse = plookup_read::read_from_1_to_2_table(SHA256_CH_INPUT, e);

    return result;
}

sparse_value map_into_maj_sparse_form(const field_t<waffle::UltraComposer>& e)
{
    sparse_value result;
    result.normal = e;
    result.sparse = plookup_read::read_from_1_to_2_table(SHA256_MAJ_INPUT, e);

    return result;
}

field_t<waffle::UltraComposer> choose(sparse_value& e, const sparse_value& f, const sparse_value& g)
{
    typedef field_t<waffle::UltraComposer> field_pt;

    const auto lookup = plookup_read::get_lookup_accumulators(SHA256_CH_INPUT, e.normal);
    const auto rotation_coefficients = sha256_tables::get_choose_rotation_multipliers();

    field_pt rotation_result = lookup[ColumnIdx::C3][0];

    e.sparse = lookup[ColumnIdx::C2][0];

    field_pt sparse_limb_3 = lookup[ColumnIdx::C2][2];

    // where is the middle limb used
    field_pt xor_result = (rotation_result * fr(7))
                              .add_two(e.sparse * (rotation_coefficients[0] * fr(7) + fr(1)),
                                       sparse_limb_3 * (rotation_coefficients[2] * fr(7)));

    field_pt choose_result_sparse = xor_result.add_two(f.sparse + f.sparse, g.sparse + g.sparse + g.sparse).normalize();

    field_pt choose_result = plookup_read::read_from_1_to_2_table(SHA256_CH_OUTPUT, choose_result_sparse);

    return choose_result;
}

field_t<waffle::UltraComposer> majority(sparse_value& a, const sparse_value& b, const sparse_value& c)
{
    typedef field_t<waffle::UltraComposer> field_pt;

    const auto lookup = plookup_read::get_lookup_accumulators(SHA256_MAJ_INPUT, a.normal);
    const auto rotation_coefficients = sha256_tables::get_majority_rotation_multipliers();

    field_pt rotation_result =
        lookup[ColumnIdx::C3][0]; // last index of first row gives accumulating sum of "non-trival" wraps
    a.sparse = lookup[ColumnIdx::C2][0];
    // use these values to compute trivial wraps somehow
    field_pt sparse_accumulator_2 = lookup[ColumnIdx::C2][1];

    field_pt xor_result = (rotation_result * fr(4))
                              .add_two(a.sparse * (rotation_coefficients[0] * fr(4) + fr(1)),
                                       sparse_accumulator_2 * (rotation_coefficients[1] * fr(4)));

    field_pt majority_result_sparse = xor_result.add_two(b.sparse, c.sparse).normalize();

    field_pt majority_result = plookup_read::read_from_1_to_2_table(SHA256_MAJ_OUTPUT, majority_result_sparse);

    return majority_result;
}

field_t<waffle::UltraComposer> add_normalize(const field_t<waffle::UltraComposer>& a,
                                             const field_t<waffle::UltraComposer>& b)
{
    typedef field_t<waffle::UltraComposer> field_pt;
    typedef witness_t<waffle::UltraComposer> witness_pt;

    waffle::UltraComposer* ctx = a.get_context() ? a.get_context() : b.get_context();

    uint256_t sum = a.get_value() + b.get_value();

    uint256_t normalized_sum = static_cast<uint32_t>(sum.data[0]);

    if (a.witness_index == IS_CONSTANT && b.witness_index == IS_CONSTANT) {
        return field_pt(ctx, normalized_sum);
    }

    field_pt overflow = witness_pt(ctx, fr((sum - normalized_sum) >> 32));

    field_pt result = a.add_two(b, overflow * field_pt(ctx, -fr((uint64_t)(1ULL << 32ULL))));

    overflow.create_range_constraint(3);
    return result;
}

std::array<field_t<waffle::UltraComposer>, 8> sha256_block(const std::array<field_t<waffle::UltraComposer>, 8>& h_init,
                                                           const std::array<field_t<waffle::UltraComposer>, 16>& input)
{
    typedef field_t<waffle::UltraComposer> field_pt;

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
    auto a = map_into_maj_sparse_form(h_init[0]);
    auto b = map_into_maj_sparse_form(h_init[1]);
    auto c = map_into_maj_sparse_form(h_init[2]);
    auto d = map_into_maj_sparse_form(h_init[3]);
    auto e = map_into_choose_sparse_form(h_init[4]);
    auto f = map_into_choose_sparse_form(h_init[5]);
    auto g = map_into_choose_sparse_form(h_init[6]);
    auto h = map_into_choose_sparse_form(h_init[7]);

    /**
     * Extend witness
     **/
    const auto w = extend_witness(input);

    /**
     * Apply SHA-256 compression function to the message schedule
     **/
    // As opposed to standard sha description - Maj and Choose functions also include required rotations for round
    for (size_t i = 0; i < 64; ++i) {
        auto ch = choose(e, f, g);
        auto maj = majority(a, b, c);
        auto temp1 = ch.add_two(h.normal, w[i] + fr(round_constants[i]));

        h = g;
        g = f;
        f = e;
        e.normal = add_normalize(d.normal, temp1);
        d = c;
        c = b;
        b = a;
        a.normal = add_normalize(temp1, maj);
    }

    /**
     * Add into previous block output and return
     **/
    std::array<field_pt, 8> output;
    output[0] = add_normalize(a.normal, h_init[0]);
    output[1] = add_normalize(b.normal, h_init[1]);
    output[2] = add_normalize(c.normal, h_init[2]);
    output[3] = add_normalize(d.normal, h_init[3]);
    output[4] = add_normalize(e.normal, h_init[4]);
    output[5] = add_normalize(f.normal, h_init[5]);
    output[6] = add_normalize(g.normal, h_init[6]);
    output[7] = add_normalize(h.normal, h_init[7]);

    /**
     * At this point, a malicilous prover could tweak the add_normalise function and the result could be 'overflowed'.
     * Thus, we need 32-bit range checks on the outputs. Note that we won't need range checks while applying the SHA-256
     * compression function because the outputs of the lookup table ensures that the output is contrained to 32 bits.
     */
    for (size_t i = 0; i < 8; i++) {
        output[i].create_range_constraint(32);
    }

    return output;
}

packed_byte_array<waffle::UltraComposer> sha256(const packed_byte_array<waffle::UltraComposer>& input)
{
    typedef field_t<waffle::UltraComposer> field_pt;

    waffle::UltraComposer* ctx = input.get_context();

    auto message_schedule(input);

    const size_t message_bits = message_schedule.size() * 8;
    message_schedule.append(field_t(ctx, 128), 1);

    constexpr size_t bytes_per_block = 64;
    const size_t num_bytes = message_schedule.size() + 8;
    const size_t num_blocks = num_bytes / bytes_per_block + (num_bytes % bytes_per_block != 0);

    const size_t num_total_bytes = num_blocks * bytes_per_block;
    for (size_t i = num_bytes; i < num_total_bytes; ++i) {
        message_schedule.append(field_t(ctx, 0), 1);
    }

    message_schedule.append(field_t(ctx, message_bits), 8);

    const auto slices = message_schedule.to_unverified_byte_slices(4);

    constexpr size_t slices_per_block = 16;

    std::array<field_pt, 8> rolling_hash;
    prepare_constants(rolling_hash);
    for (size_t i = 0; i < num_blocks; ++i) {
        std::array<field_pt, 16> hash_input;
        for (size_t j = 0; j < 16; ++j) {
            hash_input[j] = slices[i * slices_per_block + j];
        }
        rolling_hash = sha256_block(rolling_hash, hash_input);
    }

    std::vector<field_pt> output(rolling_hash.begin(), rolling_hash.end());
    return packed_byte_array<waffle::UltraComposer>(output, 4);
}

} // namespace sha256_plookup
} // namespace stdlib
} // namespace plonk
