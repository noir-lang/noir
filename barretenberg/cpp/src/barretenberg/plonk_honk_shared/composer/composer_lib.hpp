#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/polynomial_store.hpp"

#include <memory>

namespace bb {

template <typename Flavor>
std::array<typename Flavor::Polynomial, 4> construct_lookup_table_polynomials(
    const typename Flavor::CircuitBuilder& circuit, size_t dyadic_circuit_size, size_t additional_offset = 0)
{
    using Polynomial = typename Flavor::Polynomial;
    std::array<Polynomial, 4> table_polynomials;
    for (auto& poly : table_polynomials) {
        poly = Polynomial(dyadic_circuit_size);
    }

    // Create lookup selector polynomials which interpolate each table column.
    // Our selector polys always need to interpolate the full subgroup size, so here we offset so as to
    // put the table column's values at the end. (The first gates are for non-lookup constraints).
    // [0, ..., 0, ...table, 0, 0, 0, x]
    //  ^^^^^^^^^  ^^^^^^^^  ^^^^^^^  ^nonzero to ensure uniqueness and to avoid infinity commitments
    //  |          table     randomness
    //  ignored, as used for regular constraints and padding to the next power of 2.
    size_t offset = dyadic_circuit_size - circuit.get_tables_size() - additional_offset;

    for (const auto& table : circuit.lookup_tables) {
        const fr table_index(table.table_index);

        for (size_t i = 0; i < table.size; ++i) {
            table_polynomials[0][offset] = table.column_1[i];
            table_polynomials[1][offset] = table.column_2[i];
            table_polynomials[2][offset] = table.column_3[i];
            table_polynomials[3][offset] = table_index;
            ++offset;
        }
    }
    return table_polynomials;
}

/**
 * @brief Construct polynomials containing the sorted concatenation of the lookups and the lookup tables
 *
 * @tparam Flavor
 * @param circuit
 * @param dyadic_circuit_size
 * @param additional_offset Additional space needed in polynomials to add randomness for zk (Plonk only)
 * @return std::array<typename Flavor::Polynomial, 4>
 */
template <typename Flavor>
std::array<typename Flavor::Polynomial, 4> construct_sorted_list_polynomials(typename Flavor::CircuitBuilder& circuit,
                                                                             const size_t dyadic_circuit_size,
                                                                             size_t additional_offset = 0)
{
    using Polynomial = typename Flavor::Polynomial;
    std::array<Polynomial, 4> sorted_polynomials;
    // Initialise the sorted concatenated list polynomials for the lookup argument
    for (auto& s_i : sorted_polynomials) {
        s_i = Polynomial(dyadic_circuit_size);
    }

    // The sorted list polynomials have (tables_size + lookups_size) populated entries. We define the index below so
    // that these entries are written into the last indices of the polynomials. The values on the first
    // dyadic_circuit_size - (tables_size + lookups_size) indices are automatically initialized to zero via the
    // polynomial constructor.
    size_t s_index = dyadic_circuit_size - (circuit.get_tables_size() + circuit.get_lookups_size()) - additional_offset;
    ASSERT(s_index > 0); // We need at least 1 row of zeroes for the permutation argument

    for (auto& table : circuit.lookup_tables) {
        const fr table_index(table.table_index);
        auto& lookup_gates = table.lookup_gates;
        for (size_t i = 0; i < table.size; ++i) {
            if (table.use_twin_keys) {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        table.column_2[i].from_montgomery_form().data[0],
                    },
                    {
                        table.column_3[i],
                        0,
                    },
                });
            } else {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        0,
                    },
                    {
                        table.column_2[i],
                        table.column_3[i],
                    },
                });
            }
        }

#ifdef NO_TBB
        std::sort(lookup_gates.begin(), lookup_gates.end());
#else
        std::sort(std::execution::par_unseq, lookup_gates.begin(), lookup_gates.end());
#endif

        for (const auto& entry : lookup_gates) {
            const auto components = entry.to_sorted_list_components(table.use_twin_keys);
            sorted_polynomials[0][s_index] = components[0];
            sorted_polynomials[1][s_index] = components[1];
            sorted_polynomials[2][s_index] = components[2];
            sorted_polynomials[3][s_index] = table_index;
            ++s_index;
        }
    }
    return sorted_polynomials;
}

} // namespace bb
