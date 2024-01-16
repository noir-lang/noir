#pragma once

#include "../types.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"

namespace plookup {
namespace keccak_tables {

/**
 * @brief Generate the plookup tables used for the RHO round of the Keccak hash algorithm
 *
 * Keccak has 25 hash lanes, each represented as 64-bit integers.
 * The RHO round performs a left-rotation on each lane, by a fixed rotation value defined by the ROTATIONS arrray
 *
 * We evaluate Keccak in-circuit using a base-11 sparse integer representation of each hash lane:
 *
 * P = \sum_{j=0}^63 b_i * 11^i
 *
 * This allows us to replace binary operations (XOR, AND) with algebraic ones, combined with plookup tables.
 * (see keccak_chi.hpp for comments on this).
 *
 * At this point in the algorithm, each hash lane 'quasi-bit' is in the range [0, 1, 2].
 *
 * The RHO lookup tables are used to perform the following:
 *
 * 1. Normalize each quasi-bit so that P_out = \sum_{j=0}^63 (b_i mod 2) * 11^i
 * 2. Perform a left-rotation whose value is defined by a value in the ROTATIONS array
 * 3. Extract the most significant bit of the non-rotated normalized output
 *
 * The most significant bit component is required because we use this table to
 * normalize XOR operations in the IOTA round and the `sponge_absorb` phase of the algorighm.
 * (Both IOTA and `sponge_absorb` are followed by the THETA round which requires the msb of each hash lane)
 *
 * Rotations are performed by splitting the input into 'left' and 'right' bit slices
 * (left slice = bits that wrap around the 11^64 modulus of our base-11 integers)
 * (right slice = bits that don't wrap)
 *
 * Both slices are fed into a Rho plookup table. The outputs are stitched together to produce the rotated value.
 *
 * We need multiple Rho tables in order to efficiently range-constrain our input slices.
 *
 * The maximum number of bits we can accommodate in this lookup table is MAXIMUM_MULTITABLE_BITS (assume this is 8)
 * For example take a left-rotation by 1 bit. The right-slice will be a 63-bit integer.
 * 63 does not evenly divide 8. i.e. an 8-bit table cannot correctly range-constrain the input slice and we would need
 * additional range constraints.
 * We solve this problem by creating multiple Rho lookup tables each of different sizes (1 bit, 2 bits, ..., 8 bits).
 *
 * We can stitch together a lookup table sequence that correctly range constrains the left/right slices for any of our
 * 25 rotation values
 *
 * @tparam TABLE_BITS The number of bits each lookup table can accommodate
 * @tparam LANE_INDEX Required by get_rho_output_table to produce the correct MultiTable
 */
template <size_t TABLE_BITS = 0, size_t LANE_INDEX = 0> class Rho {

  public:
    static constexpr uint64_t BASE = 11;

    // EFFECTIVE_BASE = maximum value each input 'quasi-bit' can reach at this stage in the Keccak algo
    // (we use base11 as it's a bit more efficient to use a consistent base across the entire Keccak hash algorithm.
    //  The THETA round requires base-11 in order to most efficiently convert XOR operations into algebraic operations)
    static constexpr uint64_t EFFECTIVE_BASE = 3;

    // The maximum number of bits of a Rho lookup table (TABLE_BITS <= MAXIMUM_MULTITABLE_BITS).
    // Used in get_rho_output_table
    static constexpr size_t MAXIMUM_MULTITABLE_BITS = 8;

    // Rotation offsets, y vertically, x horizontally: r[y * 5 + x]
    static constexpr std::array<size_t, 25> ROTATIONS = {
        0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14,
    };

    static constexpr uint64_t RHO_NORMALIZATION_TABLE[3]{
        0,
        1,
        0,
    };

    /**
     * @brief Given a table input value, return the table output value
     *
     * Used by the Plookup code to precompute lookup tables and generate witness values
     *
     * @param key (first element = table input. Second element is unused as this lookup does not have 2 keys per value)
     * @return std::array<bb::fr, 2> table output (normalized input and normalized input / 11^TABLE_BITS - 1)
     */
    static std::array<bb::fr, 2> get_rho_renormalization_values(const std::array<uint64_t, 2> key)
    {
        uint64_t accumulator = 0;
        uint64_t input = key[0];
        uint64_t base_shift = 1;
        constexpr uint64_t divisor_exponent = TABLE_BITS - 1;
        constexpr uint64_t divisor = numeric::pow64(BASE, divisor_exponent);
        while (input > 0) {
            uint64_t slice = input % BASE;
            uint64_t bit = RHO_NORMALIZATION_TABLE[static_cast<size_t>(slice)];
            accumulator += (bit * base_shift);
            input /= BASE;
            base_shift *= BASE;
        }

        return { bb::fr(accumulator), bb::fr(accumulator / divisor) };
    }

    /**
     * @brief Precompute an array of base multipliers (11^i for i = [0, ..., TABLE_BITS - 1])
     * Code is slightly faster at runtime if we compute this at compile time
     *
     * @return constexpr std::array<uint64_t, TABLE_BITS>
     */
    static constexpr std::array<uint64_t, TABLE_BITS> get_scaled_bases()
    {
        std::array<uint64_t, TABLE_BITS> result;
        uint64_t acc = 1;
        for (size_t i = 0; i < TABLE_BITS; ++i) {
            result[i] = acc;
            acc *= BASE;
        }
        return result;
    }

    /**
     * @brief Get column values for next row of plookup table. Used to generate plookup table row values
     *
     * Input `counts` is an array of quasi-bits that represent the current row.
     * Method increases `counts` by 1 and returns the plookup table column values.
     *
     * (a bit tricky to compute because each quasi-bit ranges from [0, 1, 2],
     *  but we're working with base-11 numbers.
     *  i.e. unlike most of our lookup tables, the 1st column is not uniformly increasing by a constant value!)
     *
     * @param counts The current row value represented as an array of quasi-bits
     * @return std::array<uint64_t, 3> the columns of the plookup table
     */
    static std::array<uint64_t, 3> get_column_values_for_next_row(std::array<size_t, TABLE_BITS>& counts)
    {
        static constexpr auto scaled_bases = get_scaled_bases();

        // want the most significant bit of the 64-bit integer, when this table is used on the most significant slice
        constexpr uint64_t divisor = numeric::pow64(static_cast<uint64_t>(BASE), TABLE_BITS - 1);

        for (size_t i = 0; i < TABLE_BITS; ++i) {
            if (counts[i] == EFFECTIVE_BASE - 1) {
                counts[i] = 0;
            } else {
                counts[i] += 1;
                break;
            }
        }

        uint64_t value = 0;
        uint64_t normalized_value = 0;
        for (size_t i = 0; i < TABLE_BITS; ++i) {
            value += counts[i] * scaled_bases[i];
            normalized_value += (RHO_NORMALIZATION_TABLE[counts[i]]) * scaled_bases[i];
        }
        return { value, normalized_value, normalized_value / divisor };
    }

    /**
     * @brief Generate plookup table that normalizes a TABLE_BITS-slice of a base-11 integer and extracts the msb
     *
     * @param id
     * @param table_index
     * @return BasicTable
     */
    static BasicTable generate_rho_renormalization_table(BasicTableId id, const size_t table_index)
    {
        BasicTable table;
        table.id = id;
        table.table_index = table_index;
        table.use_twin_keys = false;
        table.size = numeric::pow64(static_cast<uint64_t>(EFFECTIVE_BASE), TABLE_BITS);

        std::array<size_t, TABLE_BITS> counts{};
        std::array<uint64_t, 3> column_values{ 0, 0, 0 };

        for (size_t i = 0; i < table.size; ++i) {
            table.column_1.emplace_back(column_values[0]);
            table.column_2.emplace_back(column_values[1]);
            table.column_3.emplace_back(column_values[2]);
            column_values = get_column_values_for_next_row(counts);
        }

        table.get_values_from_key = &get_rho_renormalization_values;

        uint64_t step_size = numeric::pow64(static_cast<uint64_t>(BASE), TABLE_BITS);
        table.column_1_step_size = bb::fr(step_size);
        table.column_2_step_size = bb::fr(step_size);
        table.column_3_step_size = bb::fr(0);
        return table;
    }

    /**
     * @brief Create the Rho MultiTable used by plookup to generate a sequence of lookups
     *
     * Keccak operates on 64-bit integers, but the lookup table only indexes TABLE_BITS bits.
     *
     * Representing the RHO lookup as a sequence of accumulating sums is tricky because of rotations.
     *
     * For example, imagine our input is a 32-bit integer A represented as: A = A3.11^24 + A2.11^16 + A1.11^8 + A0,
     *              and our output is a 32-bit integer B = B3.11^24 + B2.11^16 + B1.11^8 + B0
     *
     * In this example, we want to normalize A and left-rotate by 16 bits.
     *
     * Our lookup gate wire values will look like the following:
     *
     * | Row | C0                                       | C1           | C2       |
     * | --- | -----------------------------------------| ------------ | -------- |
     * |  0  | A3.11^24 + A2.11^16 + A1.11^8  + A0      | B1.11^8 + B0 | A0.msb() |
     * |  1  |            A3.11^16 + A2.11^8  + A1      |           B1 | A1.msb() |
     * |  2  |                       A1311^8  + A2      | B3.11^8 + B2 | A2.msb() |
     * |  3  |                                  A3      |           B3 | A3.msb() |
     *
     * Finally, an addition gate is used to extract B = 11^32 * C1[0] + C1[2]
     *
     * We use the above structure because the plookup protocol derives lookup entries by taking:
     *
     *      Colunn1 = W1[i] + Q1 . W1[i + 1]
     *      Colunn2 = W2[i] + Q2 . W2[i + 1]
     *
     * Where Q1, Q2 are constants encoded in selector polynomials.
     * We do not have any spare selector polynomials to apply to W1[i] and W2[i] :(
     *
     * i.e. we cannot represent the column C1 as a sequence of accumulating sums whilst performing a bit rotation!
     * The part of A that wraps around past 11^64 must be represented separately vs the part that does not.
     *
     * @param id
     * @return MultiTable
     */
    static MultiTable get_rho_output_table(const MultiTableId id = KECCAK_NORMALIZE_AND_ROTATE)
    {
        constexpr size_t left_bits = ROTATIONS[LANE_INDEX];
        constexpr size_t right_bits = 64 - ROTATIONS[LANE_INDEX];
        constexpr size_t num_left_tables =
            left_bits / MAXIMUM_MULTITABLE_BITS + (left_bits % MAXIMUM_MULTITABLE_BITS > 0 ? 1 : 0);
        constexpr size_t num_right_tables =
            right_bits / MAXIMUM_MULTITABLE_BITS + (right_bits % MAXIMUM_MULTITABLE_BITS > 0 ? 1 : 0);

        MultiTable table;
        table.id = id;

        table.column_1_step_sizes.push_back(1);
        table.column_2_step_sizes.push_back(1);
        table.column_3_step_sizes.push_back(1);

        // generate table selector values for the 'right' slice
        bb::constexpr_for<0, num_right_tables, 1>([&]<size_t i> {
            constexpr size_t num_bits_processed = (i * MAXIMUM_MULTITABLE_BITS);
            constexpr size_t bit_slice = (num_bits_processed + MAXIMUM_MULTITABLE_BITS > right_bits)
                                             ? right_bits % MAXIMUM_MULTITABLE_BITS
                                             : MAXIMUM_MULTITABLE_BITS;

            constexpr uint64_t scaled_base = numeric::pow64(BASE, bit_slice);
            if (i == num_right_tables - 1) {
                table.column_1_step_sizes.push_back(scaled_base);
                table.column_2_step_sizes.push_back(0);
                table.column_3_step_sizes.push_back(0);
            } else {
                table.column_1_step_sizes.push_back(scaled_base);
                table.column_2_step_sizes.push_back(scaled_base);
                table.column_3_step_sizes.push_back(0);
            }

            table.slice_sizes.push_back(scaled_base);
            table.get_table_values.emplace_back(&get_rho_renormalization_values);
            table.lookup_ids.push_back((BasicTableId)((size_t)KECCAK_RHO_1 + (bit_slice - 1)));
        });

        // generate table selector values for the 'left' slice
        bb::constexpr_for<0, num_left_tables, 1>([&]<size_t i> {
            constexpr size_t num_bits_processed = (i * MAXIMUM_MULTITABLE_BITS);

            constexpr size_t bit_slice = (num_bits_processed + MAXIMUM_MULTITABLE_BITS > left_bits)
                                             ? left_bits % MAXIMUM_MULTITABLE_BITS
                                             : MAXIMUM_MULTITABLE_BITS;
            constexpr uint64_t scaled_base = numeric::pow64(BASE, bit_slice);

            if (i != num_left_tables - 1) {
                table.column_1_step_sizes.push_back(scaled_base);
                table.column_2_step_sizes.push_back(scaled_base);
                table.column_3_step_sizes.push_back(0);
            }

            table.slice_sizes.push_back(scaled_base);
            table.get_table_values.emplace_back(&get_rho_renormalization_values);
            table.lookup_ids.push_back((BasicTableId)((size_t)KECCAK_RHO_1 + (bit_slice - 1)));
        });

        return table;
    }
};

} // namespace keccak_tables
} // namespace plookup
