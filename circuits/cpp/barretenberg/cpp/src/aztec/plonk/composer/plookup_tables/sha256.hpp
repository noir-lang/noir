#pragma once

#include <crypto/aes128/aes128.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>
#include <numeric/bitop/pow.hpp>

#include "types.hpp"
#include "sparse.hpp"

namespace plookup {
namespace sha256_tables {

static constexpr uint64_t choose_normalization_table[28]{
    /* xor result = 0 */
    0, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
    0, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
    0, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
    1, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
    0, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
    1, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
    1, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
    /* xor result = 1 */
    1, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
    1, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
    1, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
    2, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
    1, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
    2, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
    2, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
    /* xor result = 2 */
    0, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
    0, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
    0, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
    1, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
    0, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
    1, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
    1, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
    1, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
    /* xor result = 3 */
    1, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
    1, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
    2, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
    1, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
    2, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
    2, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
};

static constexpr uint64_t majority_normalization_table[16]{
    /* xor result = 0 */
    0, // a + b + c = 0 => (a & b) ^ (a & c) ^ (b & c) = 0
    0, // a + b + c = 1 => (a & b) ^ (a & c) ^ (b & c) = 0
    1, // a + b + c = 2 => (a & b) ^ (a & c) ^ (b & c) = 1
    1, // a + b + c = 3 => (a & b) ^ (a & c) ^ (b & c) = 1
    /* xor result = 1 */
    1,
    1,
    2,
    2,
    /* xor result = 2 */
    0,
    0,
    1,
    1,
    /* xor result = 3 */
    1,
    1,
    2,
    2,
};

static constexpr uint64_t witness_extension_normalization_table[16]{
    /* xor result = 0 */
    0,
    1,
    0,
    1,
    /* xor result = 1 */
    1,
    2,
    1,
    2,
    /* xor result = 2 */
    0,
    1,
    0,
    1,
    /* xor result = 3 */
    1,
    2,
    1,
    2,
};

inline BasicTable generate_witness_extension_normalization_table(BasicTableId id, const size_t table_index)
{
    return sparse_tables::generate_sparse_normalization_table<16, 3, witness_extension_normalization_table>(
        id, table_index);
}

inline BasicTable generate_choose_normalization_table(BasicTableId id, const size_t table_index)
{
    return sparse_tables::generate_sparse_normalization_table<28, 2, choose_normalization_table>(id, table_index);
}

inline BasicTable generate_majority_normalization_table(BasicTableId id, const size_t table_index)
{
    return sparse_tables::generate_sparse_normalization_table<16, 3, majority_normalization_table>(id, table_index);
}

inline MultiTable get_witness_extension_output_table(const MultiTableId id = SHA256_WITNESS_OUTPUT)
{
    const size_t num_entries = 11;

    MultiTable table(numeric::pow64(16, 3), 1 << 3, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(numeric::pow64(16, 3));
        table.lookup_ids.emplace_back(SHA256_WITNESS_NORMALIZE);
        table.get_table_values.emplace_back(
            &sparse_tables::get_sparse_normalization_values<16, witness_extension_normalization_table>);
    }
    return table;
}

inline MultiTable get_choose_output_table(const MultiTableId id = SHA256_CH_OUTPUT)
{
    const size_t num_entries = 16;

    MultiTable table(numeric::pow64(28, 2), 1 << 2, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(numeric::pow64(28, 2));
        table.lookup_ids.emplace_back(SHA256_CH_NORMALIZE);
        table.get_table_values.emplace_back(
            &sparse_tables::get_sparse_normalization_values<28, choose_normalization_table>);
    }
    return table;
}

inline MultiTable get_majority_output_table(const MultiTableId id = SHA256_MAJ_OUTPUT)
{
    const size_t num_entries = 11;

    MultiTable table(numeric::pow64(16, 3), 1 << 3, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(numeric::pow64(16, 3));
        table.lookup_ids.emplace_back(SHA256_MAJ_NORMALIZE);
        table.get_table_values.emplace_back(
            &sparse_tables::get_sparse_normalization_values<16, majority_normalization_table>);
    }
    return table;
}

inline std::array<barretenberg::fr, 3> get_majority_rotation_multipliers()
{
    constexpr uint64_t base_temp = 16;
    auto base = barretenberg::fr(base_temp);
    // scaling factors applied to a's sparse limbs, excluding the rotated limb
    const std::array<barretenberg::fr, 3> rot2_coefficients{ 0, base.pow(11 - 2), base.pow(22 - 2) };
    const std::array<barretenberg::fr, 3> rot13_coefficients{ base.pow(32 - 13), 0, base.pow(22 - 13) };
    const std::array<barretenberg::fr, 3> rot22_coefficients{ base.pow(32 - 22), base.pow(32 - 22 + 11), 0 };

    // these are the coefficients that we want
    const std::array<barretenberg::fr, 3> target_rotation_coefficients{
        rot2_coefficients[0] + rot13_coefficients[0] + rot22_coefficients[0],
        rot2_coefficients[1] + rot13_coefficients[1] + rot22_coefficients[1],
        rot2_coefficients[2] + rot13_coefficients[2] + rot22_coefficients[2],
    };

    barretenberg::fr column_2_row_1_multiplier = target_rotation_coefficients[0];
    barretenberg::fr column_2_row_2_multiplier =
        target_rotation_coefficients[0] * (-barretenberg::fr(base).pow(11)) + target_rotation_coefficients[1];

    std::array<barretenberg::fr, 3> rotation_multipliers = { column_2_row_1_multiplier,
                                                             column_2_row_2_multiplier,
                                                             barretenberg::fr(0) };
    return rotation_multipliers;
}

// template <uint64_t rot_a, uint64_t rot_b, uint64_t rot_c>
inline std::array<barretenberg::fr, 3> get_choose_rotation_multipliers()
{
    const std::array<barretenberg::fr, 3> column_2_row_3_coefficients{
        barretenberg::fr(1),
        barretenberg::fr(28).pow(11),
        barretenberg::fr(28).pow(22),
    };

    // scaling factors applied to a's sparse limbs, excluding the rotated limb
    const std::array<barretenberg::fr, 3> rot6_coefficients{ barretenberg::fr(0),
                                                             barretenberg::fr(28).pow(11 - 6),
                                                             barretenberg::fr(28).pow(22 - 6) };
    const std::array<barretenberg::fr, 3> rot11_coefficients{ barretenberg::fr(28).pow(32 - 11),
                                                              barretenberg::fr(0),
                                                              barretenberg::fr(28).pow(22 - 11) };
    const std::array<barretenberg::fr, 3> rot25_coefficients{ barretenberg::fr(28).pow(32 - 25),
                                                              barretenberg::fr(28).pow(32 - 25 + 11),
                                                              barretenberg::fr(0) };

    // these are the coefficients that we want
    const std::array<barretenberg::fr, 3> target_rotation_coefficients{
        rot6_coefficients[0] + rot11_coefficients[0] + rot25_coefficients[0],
        rot6_coefficients[1] + rot11_coefficients[1] + rot25_coefficients[1],
        rot6_coefficients[2] + rot11_coefficients[2] + rot25_coefficients[2],
    };

    barretenberg::fr column_2_row_1_multiplier =
        barretenberg::fr(1) * target_rotation_coefficients[0]; // why multiply by one?

    // this gives us the correct scaling factor for a0's 1st limb
    std::array<barretenberg::fr, 3> current_coefficients{
        column_2_row_3_coefficients[0] * column_2_row_1_multiplier,
        column_2_row_3_coefficients[1] * column_2_row_1_multiplier,
        column_2_row_3_coefficients[2] * column_2_row_1_multiplier,
    };

    barretenberg::fr column_2_row_3_multiplier = -(current_coefficients[2]) + target_rotation_coefficients[2];

    std::array<barretenberg::fr, 3> rotation_multipliers = { column_2_row_1_multiplier,
                                                             barretenberg::fr(0),
                                                             column_2_row_3_multiplier };
    return rotation_multipliers;
}

inline MultiTable get_witness_extension_input_table(const MultiTableId id = SHA256_WITNESS_INPUT)
{
    std::vector<barretenberg::fr> column_1_coefficients{ 1, 1 << 3, 1 << 10, 1 << 18 };
    std::vector<barretenberg::fr> column_2_coefficients{ 0, 0, 0, 0 };
    std::vector<barretenberg::fr> column_3_coefficients{ 0, 0, 0, 0 };
    MultiTable table(column_1_coefficients, column_2_coefficients, column_3_coefficients);
    table.id = id;
    table.slice_sizes = { (1 << 3), (1 << 7), (1 << 8), (1 << 18) };
    table.lookup_ids = { SHA256_WITNESS_SLICE_3,
                         SHA256_WITNESS_SLICE_7_ROTATE_4,
                         SHA256_WITNESS_SLICE_8_ROTATE_7,
                         SHA256_WITNESS_SLICE_14_ROTATE_1 };

    table.get_table_values = {
        &sparse_tables::get_sparse_table_with_rotation_values<16, 0>,
        &sparse_tables::get_sparse_table_with_rotation_values<16, 4>,
        &sparse_tables::get_sparse_table_with_rotation_values<16, 7>,
        &sparse_tables::get_sparse_table_with_rotation_values<16, 1>,
    };
    return table;
}

inline MultiTable get_choose_input_table(const MultiTableId id = SHA256_CH_INPUT)
{
    /**
     * When reading from our lookup tables, we can read from the differences between adjacent rows in program memory,
     *instead of taking absolute values
     *
     * For example, if our layout in memory is:
     *
     * |  1  |  2  |  3  |
     * |  -  |  -  |  -  |
     * | a_1 | b_1 | c_1 |
     * | a_2 | b_2 | c_2 |
     * | ... | ... | ... |
     *
     * We can valdiate that (a_1 + q_0 * a_2) is a table key and (c_1 + q_1 * c_2), (b_1 + q_2 * b_2) are table values,
     * where q_0, q_1, q_2 are precomputed constants
     *
     * This allows us to assemble accumulating sums out of multiple table reads, without requiring extra addition gates.
     *
     * We can also use this feature to evaluate our sha256 rotations more efficiently, when converting into sparse form.
     *
     * Let column 1 represents our 'normal' scalar, column 2 represents our scalar in sparse form
     *
     * It's simple enough to make columns 1 and 2 track the accumulating sum of our scalar in normal and sparse form.
     *
     * Column 3 contains terms we can combine with our accumulated sparse scalar, to obtain our rotated scalar.
     *
     * Each lookup table will be of size 2^11. as that allows us to decompose a 32-bit scalar into sparse form in 3
     *reads (2^16 is too expensive for small circuits)
     *
     * For example, if we want to rotate `a` by 6 bits, we make the first lookup access the table that rotates `b` by 6
     *bits. Subsequent table reads do not need to be rotated, as the 11-bit limbs will not cross 32-bit boundary and can
     *be scaled by constants
     *
     * With this in mind, we want to tackle the SHA256 `ch` sub-algorithm
     *
     * This requires us to compute ((a >>> 6) ^ (a >>> 11) ^ (a >>> 25)) + ((a ^ b) ^ (~a ^ c))
     *
     * In sparse form, we can represent this as:
     *
     *      7 * (a >>> 6) + (a >>> 11) + (a >>> 25) + (a + 2 * b + 3 * c)
     *
     * When decomposing a into sparse form, we would therefore like to obtain the following:
     *
     *      7 * (a >>> 6) + (a >>> 11) + (a >>> 25) + (a)
     *
     * We need to determine the values of the constants (q_1, q_2, q_3) that we will be scaling our lookup values by,
     *when assembling our accumulated sums.
     *
     * We need the sparse representation of `a` elsewhere in the algorithm, so the constants in columns 1 and 2 are
     *fixed.
     *
     **/

    // scaling factors applied to a's sparse limbs, excluding the rotated limb
    const std::array<barretenberg::fr, 3> rot6_coefficients{ barretenberg::fr(0),
                                                             barretenberg::fr(28).pow(11 - 6),
                                                             barretenberg::fr(28).pow(22 - 6) };
    const std::array<barretenberg::fr, 3> rot11_coefficients{ barretenberg::fr(28).pow(32 - 11),
                                                              barretenberg::fr(0),
                                                              barretenberg::fr(28).pow(22 - 11) };
    const std::array<barretenberg::fr, 3> rot25_coefficients{ barretenberg::fr(28).pow(32 - 25),
                                                              barretenberg::fr(28).pow(32 - 25 + 11),
                                                              barretenberg::fr(0) };

    // these are the coefficients that we want
    const std::array<barretenberg::fr, 3> target_rotation_coefficients{
        rot6_coefficients[0] + rot11_coefficients[0] + rot25_coefficients[0],
        rot6_coefficients[1] + rot11_coefficients[1] + rot25_coefficients[1],
        rot6_coefficients[2] + rot11_coefficients[2] + rot25_coefficients[2],
    };

    barretenberg::fr column_2_row_1_multiplier = target_rotation_coefficients[0];

    // this gives us the correct scaling factor for a0's 1st limb
    std::array<barretenberg::fr, 3> current_coefficients{
        column_2_row_1_multiplier,
        barretenberg::fr(28).pow(11) * column_2_row_1_multiplier,
        barretenberg::fr(28).pow(22) * column_2_row_1_multiplier,
    };

    // barretenberg::fr column_2_row_3_multiplier = -(current_coefficients[2]) + target_rotation_coefficients[2];
    barretenberg::fr column_3_row_2_multiplier = -(current_coefficients[1]) + target_rotation_coefficients[1];

    std::vector<barretenberg::fr> column_1_coefficients{ barretenberg::fr(1),
                                                         barretenberg::fr(1 << 11),
                                                         barretenberg::fr(1 << 22) };
    std::vector<barretenberg::fr> column_2_coefficients{ barretenberg::fr(1),
                                                         barretenberg::fr(28).pow(11),
                                                         barretenberg::fr(28).pow(22) };
    std::vector<barretenberg::fr> column_3_coefficients{ barretenberg::fr(1),
                                                         column_3_row_2_multiplier + barretenberg::fr(1),
                                                         barretenberg::fr(1) };
    MultiTable table(column_1_coefficients, column_2_coefficients, column_3_coefficients);
    table.id = id;
    table.slice_sizes = { (1 << 11), (1 << 11), (1 << 10) };
    table.lookup_ids = { SHA256_BASE28_ROTATE6, SHA256_BASE28, SHA256_BASE28_ROTATE3 };

    table.get_table_values.push_back(&sparse_tables::get_sparse_table_with_rotation_values<28, 6>);
    table.get_table_values.push_back(&sparse_tables::get_sparse_table_with_rotation_values<28, 0>);
    table.get_table_values.push_back(&sparse_tables::get_sparse_table_with_rotation_values<28, 3>);
    // table.get_table_values = std::vector<MultiTable::table_out (*)(MultiTable::table_in)>{

    //     &get_sha256_sparse_map_values<28, 0, 0>,
    //     &get_sha256_sparse_map_values<28, 3, 0>,
    // };
    return table;
}

// This table (at third row and column) returns the sum of roations that "non-trivially wrap"
inline MultiTable get_majority_input_table(const MultiTableId id = SHA256_MAJ_INPUT)
{
    /**
     * We want to tackle the SHA256 `maj` sub-algorithm
     *
     * This requires us to compute ((a >>> 2) ^ (a >>> 13) ^ (a >>> 22)) + ((a & b) ^ (a & c) ^ (b & c))
     *
     * In sparse form, we can represent this as:
     *
     *      4 * (a >>> 2) + (a >>> 13) + (a >>> 22) +  (a + b + c)
     *
     *
     * We need to determine the values of the constants (q_1, q_2, q_3) that we will be scaling our lookup values by,
     *when assembling our accumulated sums.
     *
     * We need the sparse representation of `a` elsewhere in the algorithm, so the constants in columns 1 and 2 are
     *fixed.
     *
     **/
    constexpr uint64_t base = 16;

    // scaling factors applied to a's sparse limbs, excluding the rotated limb
    const std::array<barretenberg::fr, 3> rot2_coefficients{ barretenberg::fr(0),
                                                             barretenberg::fr(base).pow(11 - 2),
                                                             barretenberg::fr(base).pow(22 - 2) };
    const std::array<barretenberg::fr, 3> rot13_coefficients{ barretenberg::fr(base).pow(32 - 13),
                                                              barretenberg::fr(0),
                                                              barretenberg::fr(base).pow(22 - 13) };
    const std::array<barretenberg::fr, 3> rot22_coefficients{ barretenberg::fr(base).pow(32 - 22),
                                                              barretenberg::fr(base).pow(32 - 22 + 11),
                                                              barretenberg::fr(0) };

    // these are the coefficients that we want
    const std::array<barretenberg::fr, 3> target_rotation_coefficients{
        rot2_coefficients[0] + rot13_coefficients[0] + rot22_coefficients[0],
        rot2_coefficients[1] + rot13_coefficients[1] + rot22_coefficients[1],
        rot2_coefficients[2] + rot13_coefficients[2] + rot22_coefficients[2],
    };

    barretenberg::fr column_2_row_3_multiplier =
        target_rotation_coefficients[1] * (-barretenberg::fr(base).pow(11)) + target_rotation_coefficients[2];

    std::vector<barretenberg::fr> column_1_coefficients{ barretenberg::fr(1),
                                                         barretenberg::fr(1 << 11),
                                                         barretenberg::fr(1 << 22) };
    std::vector<barretenberg::fr> column_2_coefficients{ barretenberg::fr(1),
                                                         barretenberg::fr(base).pow(11),
                                                         barretenberg::fr(base).pow(22) };
    std::vector<barretenberg::fr> column_3_coefficients{ barretenberg::fr(1),
                                                         barretenberg::fr(1),
                                                         barretenberg::fr(1) + column_2_row_3_multiplier };

    MultiTable table(column_1_coefficients, column_2_coefficients, column_3_coefficients);
    table.id = id;
    table.slice_sizes = { (1 << 11), (1 << 11), (1 << 10) };
    table.lookup_ids = { SHA256_BASE16_ROTATE2, SHA256_BASE16_ROTATE2, SHA256_BASE16 };
    table.get_table_values = {
        &sparse_tables::get_sparse_table_with_rotation_values<16, 2>,
        &sparse_tables::get_sparse_table_with_rotation_values<16, 2>,
        &sparse_tables::get_sparse_table_with_rotation_values<16, 0>,
    };
    return table;
}

} // namespace sha256_tables
} // namespace plookup