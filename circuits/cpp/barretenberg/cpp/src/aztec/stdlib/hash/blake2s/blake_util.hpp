#pragma once
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <stdlib/primitives/plookup/plookup.hpp>

namespace plonk {
namespace stdlib {

namespace blake_util {

// constants
enum blake_constant { BLAKE3_STATE_SIZE = 16 };

constexpr uint8_t MSG_SCHEDULE_BLAKE3[7][16] = {
    { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }, { 2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8 },
    { 3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1 }, { 10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6 },
    { 12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4 }, { 9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7 },
    { 11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13 },
};

constexpr uint8_t MSG_SCHEDULE_BLAKE2[10][16] = {
    { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }, { 14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3 },
    { 11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4 }, { 7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8 },
    { 9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13 }, { 2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9 },
    { 12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11 }, { 13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10 },
    { 6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5 }, { 10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0 },
};

/**
 * Addition with normalisation (to ensure the addition is in the scalar field.)
 * Given two field_t elements a and b, this function computes ((a + b) % 2^{32}).
 * Additionally, it checks if the overflow of the addition is a maximum of 3 bits.
 * This is to ascertain that the additions of two 32-bit scalars in blake2s and blake3s do not exceed 35 bits.
 */
template <typename Composer> field_t<Composer> add_normalize(const field_t<Composer>& a, const field_t<Composer>& b)
{
    typedef field_t<Composer> field_pt;
    typedef witness_t<Composer> witness_pt;

    Composer* ctx = a.get_context() ? a.get_context() : b.get_context();

    uint256_t sum = a.get_value() + b.get_value();

    uint256_t normalized_sum = static_cast<uint32_t>(sum.data[0]);

    if (a.witness_index == IS_CONSTANT && b.witness_index == IS_CONSTANT) {
        return field_pt(ctx, normalized_sum);
    }

    field_pt overflow = witness_pt(ctx, fr((sum - normalized_sum) >> 32));

    // The overflow here could be of 2 bits because we allow that much overflow in the Blake rounds.
    overflow.create_range_constraint(3);

    // a + b - (overflow * 2^{32})
    field_pt result = a.add_two(b, overflow * field_pt(ctx, -fr((uint64_t)(1ULL << 32ULL))));

    return result;
}

/**
 *
 * Function `G' in the Blake2s and Blake3s algorithm which is the core
 * mixing step with additions, xors and right-rotates. This function is
 * used in both TurboPlonk (without lookup tables).
 *
 * Inputs: - A pointer to a 16-word `state`,
 *         - indices a, b, c, d,
 *         - addition messages x and y
 *
 **/
template <typename Composer>
void g(uint32<Composer> state[BLAKE3_STATE_SIZE],
       size_t a,
       size_t b,
       size_t c,
       size_t d,
       uint32<Composer> x,
       uint32<Composer> y)
{
    state[a] = state[a] + state[b] + x;
    state[d] = (state[d] ^ state[a]).ror(16);
    state[c] = state[c] + state[d];
    state[b] = (state[b] ^ state[c]).ror(12);
    state[a] = state[a] + state[b] + y;
    state[d] = (state[d] ^ state[a]).ror(8);
    state[c] = state[c] + state[d];
    state[b] = (state[b] ^ state[c]).ror(7);
}

/**
 *
 * Function `G' in the Blake2s and Blake3s algorithm which is the core
 * mixing step with additions, xors and right-rotates. This function is
 * used in  UltraPlonk version (with lookup tables).
 *
 * Inputs: - A pointer to a 16-word `state`,
 *         - indices a, b, c, d,
 *         - addition messages x and y
 *         - boolean `last_update` to make sure addition is normalised only in
 *           last update of the state
 *
 * Gate costs per call to function G in lookup case:
 *
 * Read sequence from table = 6 gates per read => 6 * 4 = 24
 * Addition gates = 4 gates
 * Range gates = 2 gates
 * Addition gate for correct output of XOR rotate 12 = 1 gate
 * Normalizing scaling factors = 2 gates
 *
 * Subtotal = 33 gates
 * Outside rounds, each of Blake2s and Blake3s needs 20 and 24 lookup reads respectively.
 *
 * +-----------+--------------+-----------------------+---------------------------+--------------+
 * |           |  calls to G  | gate count for rounds | gate count outside rounds |    total     |
 * |-----------|--------------|-----------------------|---------------------------|--------------|
 * |  Blake2s  |      80      |        80 * 33        |          20 * 6           |     2760     |
 * |  Blake3s  |      56      |        56 * 33        |          24 * 6           |     1992     |
 * +-----------+--------------+-----------------------+---------------------------+--------------+
 *
 * P.S. This doesn't include some more addition gates required after the rounds.
 *      This cost would be negligible as compared to the above gate counts.
 *
 *
 * TODO: Idea for getting rid of extra addition and multiplication gates by tweaking gate structure.
 *       To be implemented later.
 *
 *   q_plookup = 1        | d0 | a0 | d'0 | --  |
 *   q_plookup = 1        | d1 | a1 | d'1 | d2  | <--- set q_arith = 1 and validate d2 - d'5 * scale_factor = 0
 *   q_plookup = 1        | d2 | a2 | d'2 | d'5 |
 *   q_plookup = 1        | d3 | a3 | d'3 | --  |
 *   q_plookup = 1        | d4 | a4 | d'4 | --  |
 *   q_plookup = 1        | d5 | a5 | d'5 | c   |  <---- set q_arith = 1 and validate d'5 * scale_factor + c - c2 =
 * 0. |               | c2  |  <---- this row is start of another lookup table (b ^ c)
 *
 *
 **/
template <typename Composer>
void g_lookup(field_t<Composer> state[BLAKE3_STATE_SIZE],
              size_t a,
              size_t b,
              size_t c,
              size_t d,
              field_t<Composer> x,
              field_t<Composer> y,
              const bool last_update = false)
{
    typedef field_t<Composer> field_pt;

    // For simplicity, state[a] is written as `a' in comments.
    // a = a + b + x
    state[a] = state[a].add_two(state[b], x);

    // d = (d ^ a).ror(16)
    const auto lookup_1 = plookup_read::get_lookup_accumulators(BLAKE_XOR_ROTATE_16, state[d], state[a], true);
    field_pt scaling_factor_1 = (1 << (32 - 16));
    state[d] = lookup_1[ColumnIdx::C3][0] * scaling_factor_1;

    // c = c + d
    state[c] = state[c] + state[d];

    // b = (b ^ c).ror(12)
    const auto lookup_2 = plookup_read::get_lookup_accumulators(BLAKE_XOR, state[b], state[c], true);
    field_pt lookup_output = lookup_2[ColumnIdx::C3][2];
    field_pt t2_term = field_pt(1 << 12) * lookup_2[ColumnIdx::C3][2];
    lookup_output += (lookup_2[ColumnIdx::C3][0] - t2_term) * field_pt(1 << 20);
    state[b] = lookup_output;

    // a = a + b + y
    if (!last_update) {
        state[a] = state[a].add_two(state[b], y);
    } else {
        state[a] = add_normalize(state[a], state[b] + y);
    }

    // d = (d ^ a).ror(8)
    const auto lookup_3 = plookup_read::get_lookup_accumulators(BLAKE_XOR_ROTATE_8, state[d], state[a], true);
    field_pt scaling_factor_3 = (1 << (32 - 8));
    state[d] = lookup_3[ColumnIdx::C3][0] * scaling_factor_3;

    // c = c + d
    if (!last_update) {
        state[c] = state[c] + state[d];
    } else {
        state[c] = add_normalize(state[c], state[d]);
    }

    // b = (b ^ c).ror(7)
    const auto lookup_4 = plookup_read::get_lookup_accumulators(BLAKE_XOR_ROTATE_7, state[b], state[c], true);
    field_pt scaling_factor_4 = (1 << (32 - 7));
    state[b] = lookup_4[ColumnIdx::C3][0] * scaling_factor_4;
}

/*
 * This is the round function used in Blake2s and Blake3s for TurboPlonk.
 * Inputs: - 16-word state
 *         - 16-word msg
 *         - round numbe
 *         - which_blake to choose Blake2 or Blake3 (false -> Blake2)
 */
template <typename Composer>
void round_fn(uint32<Composer> state[BLAKE3_STATE_SIZE],
              uint32<Composer> msg[BLAKE3_STATE_SIZE],
              size_t round,
              const bool which_blake = false)
{
    // Select the message schedule based on the round.
    const uint8_t* schedule = which_blake ? MSG_SCHEDULE_BLAKE3[round] : MSG_SCHEDULE_BLAKE2[round];

    // Mix the columns.
    g<Composer>(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]]);
    g<Composer>(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]]);
    g<Composer>(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]]);
    g<Composer>(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]]);

    // Mix the rows.
    g<Composer>(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]]);
    g<Composer>(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]]);
    g<Composer>(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]]);
    g<Composer>(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]]);
}

/*
 * This is the round function used in Blake2s and Blake3s for UltraPlonk.
 * Inputs: - 16-word state
 *         - 16-word msg
 *         - round numbe
 *         - which_blake to choose Blake2 or Blake3 (false -> Blake2)
 */
template <typename Composer>
void round_fn_lookup(field_t<Composer> state[BLAKE3_STATE_SIZE],
                     field_t<Composer> msg[BLAKE3_STATE_SIZE],
                     size_t round,
                     const bool which_blake = false)
{
    // Select the message schedule based on the round.
    const uint8_t* schedule = which_blake ? MSG_SCHEDULE_BLAKE3[round] : MSG_SCHEDULE_BLAKE2[round];

    // Mix the columns.
    g_lookup<Composer>(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]]);
    g_lookup<Composer>(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]]);
    g_lookup<Composer>(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]]);
    g_lookup<Composer>(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]]);

    // Mix the rows.
    g_lookup<Composer>(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]], true);
    g_lookup<Composer>(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]], true);
    g_lookup<Composer>(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]], true);
    g_lookup<Composer>(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]], true);
}

} // namespace blake_util

} // namespace stdlib
} // namespace plonk
