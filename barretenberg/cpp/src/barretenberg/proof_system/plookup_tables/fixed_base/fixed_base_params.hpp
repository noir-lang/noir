#pragma once

#include "barretenberg/plonk/proof_system/constants.hpp"
#include <array>
#include <barretenberg/common/assert.hpp>
#include <cstddef>
#include <cstdint>

namespace bb::plookup {
/**
 * @brief Parameters definitions for our fixed-base-scalar-multiplication lookup tables
 *
 */
struct FixedBaseParams {
    static constexpr size_t BITS_PER_TABLE = 9;
    static constexpr size_t BITS_ON_CURVE = 254;

    // We split 1 254-bit scalar mul into two scalar muls of size BITS_PER_LO_SCALAR, BITS_PER_HI_SCALAR.
    // This enables us to efficiently decompose our input scalar multiplier into two chunks of a known size.
    // (i.e. we get free BITS_PER_LO_SCALAR, BITS_PER_HI_SCALAR range checks as part of the lookup table subroutine)
    // This in turn allows us to perform a primality test more efficiently.
    // i.e. check that input scalar < prime modulus when evaluated over the integers
    // (the primality check requires us to split the input into high / low bit chunks so getting this for free as part
    // of the lookup algorithm is nice!)
    static constexpr size_t BITS_PER_LO_SCALAR = 128;
    static constexpr size_t BITS_PER_HI_SCALAR = BITS_ON_CURVE - BITS_PER_LO_SCALAR;
    // max table size because the last lookup table might be smaller (BITS_PER_TABLE does not neatly divide
    // BITS_PER_LO_SCALAR)
    static constexpr size_t MAX_TABLE_SIZE = (1UL) << BITS_PER_TABLE;
    // how many BITS_PER_TABLE lookup tables do we need to traverse BITS_PER_LO_SCALAR-amount of bits?
    // (we implicitly assume BITS_PER_LO_SCALAR > BITS_PER_HI_SCALAR)
    static constexpr size_t MAX_NUM_TABLES_IN_MULTITABLE =
        (BITS_PER_LO_SCALAR / BITS_PER_TABLE) + (BITS_PER_LO_SCALAR % BITS_PER_TABLE == 0 ? 0 : 1);
    static constexpr size_t NUM_POINTS = 2;
    // how many multitables are we creating? It's 4 because we want enough lookup tables to cover two field elements,
    // two field elements = 2 scalar muls = 4 scalar mul hi/lo slices = 4 multitables
    static constexpr size_t NUM_FIXED_BASE_MULTI_TABLES = NUM_POINTS * 2;
    static constexpr size_t NUM_TABLES_PER_LO_MULTITABLE =
        (BITS_PER_LO_SCALAR / BITS_PER_TABLE) + ((BITS_PER_LO_SCALAR % BITS_PER_TABLE == 0) ? 0 : 1);
    static constexpr size_t NUM_TABLES_PER_HI_MULTITABLE =
        (BITS_PER_HI_SCALAR / BITS_PER_TABLE) + ((BITS_PER_HI_SCALAR % BITS_PER_TABLE == 0) ? 0 : 1);
    // how many lookups are required to perform a scalar mul of a field element with a base point?
    static constexpr size_t NUM_BASIC_TABLES_PER_BASE_POINT =
        (NUM_TABLES_PER_LO_MULTITABLE + NUM_TABLES_PER_HI_MULTITABLE);
    // how many basic lookup tables are we creating in total to support fixed-base-scalar-muls over two precomputed base
    // points.
    static constexpr size_t NUM_FIXED_BASE_BASIC_TABLES = NUM_BASIC_TABLES_PER_BASE_POINT * NUM_POINTS;

    /**
     * @brief For a scalar multiplication table that covers input scalars up to `(1 << num_bits) - 1`,
     *        how many individual lookup tables of max size BITS_PER_TABLE do we need?
     *        (e.g. if BITS_PER_TABLE = 9, for `num_bits = 126` it's 14. For `num_bits = 128` it's 15)
     * @tparam num_bits
     * @return constexpr size_t
     */
    template <size_t num_bits> inline static constexpr size_t get_num_tables_per_multi_table() noexcept
    {
        return (num_bits / BITS_PER_TABLE) + ((num_bits % BITS_PER_TABLE == 0) ? 0 : 1);
    }

    /**
     * @brief For a given multitable index, how many scalar mul bits are we traversing with our multitable?
     *
     * @param multitable_index Ranges from 0 to NUM_FIXED_BASE_MULTI_TABLES - 1
     * @return constexpr size_t
     */
    static constexpr size_t get_num_bits_of_multi_table(const size_t multitable_index)
    {
        ASSERT(multitable_index < NUM_FIXED_BASE_MULTI_TABLES);
        constexpr std::array<size_t, 4> MULTI_TABLE_BIT_LENGTHS{
            BITS_PER_LO_SCALAR, BITS_PER_HI_SCALAR, BITS_PER_LO_SCALAR, BITS_PER_HI_SCALAR
        };
        return MULTI_TABLE_BIT_LENGTHS[multitable_index];
    }
};
} // namespace bb::plookup