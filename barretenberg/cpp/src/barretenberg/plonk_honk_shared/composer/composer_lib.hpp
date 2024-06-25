#pragma once
#include "barretenberg/common/ref_array.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/polynomial_store.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/types.hpp"

#include <memory>

namespace bb {

template <typename Flavor>
void construct_lookup_table_polynomials(RefArray<typename Flavor::Polynomial, 4> table_polynomials,
                                        const typename Flavor::CircuitBuilder& circuit,
                                        size_t dyadic_circuit_size,
                                        size_t additional_offset = 0)
{
    // Create lookup selector polynomials which interpolate each table column.
    // Our selector polys always need to interpolate the full subgroup size, so here we offset so as to
    // put the table column's values at the end. (The first gates are for non-lookup constraints).
    // [0, ..., 0, ...table, 0, 0, 0, x]
    //  ^^^^^^^^^  ^^^^^^^^  ^^^^^^^  ^nonzero to ensure uniqueness and to avoid infinity commitments
    //  |          table     randomness
    //  ignored, as used for regular constraints and padding to the next power of 2.
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1033): construct tables and counts at top of trace
    ASSERT(dyadic_circuit_size > circuit.get_tables_size() + additional_offset);
    size_t offset = dyadic_circuit_size - circuit.get_tables_size() - additional_offset;

    for (const auto& table : circuit.lookup_tables) {
        const fr table_index(table.table_index);

        for (size_t i = 0; i < table.size(); ++i) {
            table_polynomials[0][offset] = table.column_1[i];
            table_polynomials[1][offset] = table.column_2[i];
            table_polynomials[2][offset] = table.column_3[i];
            table_polynomials[3][offset] = table_index;
            ++offset;
        }
    }
}

/**
 * @brief Construct polynomial whose value at index i is the number of times the table entry at that index has been
 * read.
 * @details Read counts are needed for the log derivative lookup argument. The table polynomials are constructed as a
 * concatenation of basic 3-column tables. Similarly, the read counts polynomial is constructed as the concatenation of
 * read counts for the individual tables.
 */
template <typename Flavor>
void construct_lookup_read_counts(typename Flavor::Polynomial& read_counts,
                                  typename Flavor::Polynomial& read_tags,
                                  typename Flavor::CircuitBuilder& circuit,
                                  size_t dyadic_circuit_size)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1033): construct tables and counts at top of trace
    size_t offset = dyadic_circuit_size - circuit.get_tables_size();

    size_t table_offset = offset; // offset of the present table in the table polynomials
    // loop over all tables used in the circuit; each table contains data about the lookups made on it
    for (auto& table : circuit.lookup_tables) {
        table.initialize_index_map();

        for (auto& gate_data : table.lookup_gates) {
            // convert lookup gate data to an array of three field elements, one for each of the 3 columns
            auto table_entry = gate_data.to_table_components(table.use_twin_keys);

            // find the index of the entry in the table
            auto index_in_table = table.index_map[table_entry];

            // increment the read count at the corresponding index in the full polynomial
            size_t index_in_poly = table_offset + index_in_table;
            read_counts[index_in_poly]++;
            read_tags[index_in_poly] = 1; // tag is 1 if entry has been read 1 or more times
        }
        table_offset += table.size(); // set the offset of the next table within the polynomials
    }
}

} // namespace bb
