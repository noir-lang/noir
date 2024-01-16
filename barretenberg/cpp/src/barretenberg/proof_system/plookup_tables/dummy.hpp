#pragma once
/**
 * @file dummy.hpp
 * @author Rumata888
 * @brief This file contains functions for the dummy tables that we use in UltraHonk to make table, sorted and lookup
 * selector polynomials non-zero.
 *
 */

#include "types.hpp"

namespace plookup {
namespace dummy_tables {

/**
 * @brief Lookup the value corresponding to a specific key
 *
 * @details We need this function for when we are constructing the circuit and want to query the table. Since we need
 * two basic tables to make the table polynomial have non-zero values, we instantiate two tables with the same function,
 * but change it slightly through templating
 *
 * @tparam id The id of the basic table used to parametrize the values for 2 fake tables
 * @param key The key that we are looking up
 * @return std::array<bb::fr, 2>
 */
template <uint64_t id> inline std::array<bb::fr, 2> get_value_from_key(const std::array<uint64_t, 2> key)
{
    return { key[0] * 3 + key[1] * 4 + id * 0x1337ULL, 0ULL };
}

/**
 * @brief Generate the whole table
 *
 * @details This function is used to generate the whole table for the table polynomial. It's templated with id, since we
 * generate 2 slightly different fake tables.
 *
 * @tparam table_id The id of the table this function is instantiated for
 * @param id Table id that is the same for all circuits
 * @param table_index The index for this table that is used in this circuit. 0, 1, ...
 * @return A table of values
 */
template <uint64_t table_id>
inline BasicTable generate_honk_dummy_table(const BasicTableId id, const size_t table_index)
{

    // We do the assertion, since this function is templated, but the general API for these functions contains the id,
    // too. This helps us ensure that the correct instantion is used for a particular BasicTableId
    ASSERT(table_id == static_cast<uint64_t>(id));
    const size_t base = 1 << 1; // Probably has to be a power of 2
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = base * base;
    table.use_twin_keys = true;
    for (uint64_t i = 0; i < base; ++i) {
        for (uint64_t j = 0; j < base; ++j) {
            table.column_1.emplace_back(i);
            table.column_2.emplace_back(j);
            table.column_3.emplace_back(i * 3 + j * 4 + static_cast<uint64_t>(id) * 0x1337ULL);
        }
    }

    table.get_values_from_key = &get_value_from_key<table_id>;

    table.column_1_step_size = base;
    table.column_2_step_size = base;
    table.column_3_step_size = base;

    return table;
}
/**
 * @brief Create a multitable for filling UltraHonk polynomials with non-zero values
 *
 * @details Consists of 2 Basic tables that are almost identical. Each of those basic tables should only have 4 entries,
 * so the overall overhead is just 8
 *
 */
inline MultiTable get_honk_dummy_multitable()
{
    const MultiTableId id = HONK_DUMMY_MULTI;
    const size_t number_of_elements_in_argument = 1 << 1; // Probably has to be a power of 2
    const size_t number_of_lookups = 2;
    MultiTable table(number_of_elements_in_argument,
                     number_of_elements_in_argument,
                     number_of_elements_in_argument,
                     number_of_lookups);
    table.id = id;
    table.slice_sizes.emplace_back(number_of_elements_in_argument);
    table.lookup_ids.emplace_back(HONK_DUMMY_BASIC1);
    table.get_table_values.emplace_back(&get_value_from_key<HONK_DUMMY_BASIC1>);
    table.slice_sizes.emplace_back(number_of_elements_in_argument);
    table.lookup_ids.emplace_back(HONK_DUMMY_BASIC2);
    table.get_table_values.emplace_back(&get_value_from_key<HONK_DUMMY_BASIC2>);
    return table;
}
} // namespace dummy_tables
} // namespace plookup