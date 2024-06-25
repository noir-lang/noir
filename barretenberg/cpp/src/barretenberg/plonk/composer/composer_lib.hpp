#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

namespace bb::plonk {
struct SelectorProperties {
    std::string name;
    // TODO: does the prover need the raw lagrange-base selector values?
    bool requires_lagrange_base_polynomial = false;
};

/**
 * @brief Fill the last index of each selector polynomial in lagrange form with a non-zero value
 *
 * @tparam Flavor
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
void enforce_nonzero_selector_polynomials(const auto& circuit_constructor, auto* proving_key)
{
    for (size_t idx = 0; idx < circuit_constructor.num_selectors; ++idx) {
        auto current_selector =
            proving_key->polynomial_store.get(circuit_constructor.selector_names[idx] + "_lagrange");
        current_selector[current_selector.size() - 1] = idx + 1;
        proving_key->polynomial_store.put(circuit_constructor.selector_names[idx] + "_lagrange",
                                          std::move(current_selector));
    }
}

/**
 * @brief Retrieve lagrange forms of selector polynomials and compute monomial and coset-monomial forms and put into
 * cache
 *
 * @param key Pointer to the proving key TODO(#293)
 * @param selector_properties Names of selectors
 */
void compute_monomial_and_coset_selector_forms(plonk::proving_key* key,
                                               std::vector<SelectorProperties> selector_properties);

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */
std::shared_ptr<plonk::verification_key> compute_verification_key_common(
    std::shared_ptr<plonk::proving_key> const& proving_key,
    // silencing for now but need to figure out where to extract type of VerifierCrs from :-/
    std::shared_ptr<bb::srs::factories::VerifierCrs<curve::BN254>> const& vrs);

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
        for (size_t i = 0; i < table.size(); ++i) {
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
            const auto components = entry.to_table_components(table.use_twin_keys);
            sorted_polynomials[0][s_index] = components[0];
            sorted_polynomials[1][s_index] = components[1];
            sorted_polynomials[2][s_index] = components[2];
            sorted_polynomials[3][s_index] = table_index;
            ++s_index;
        }
    }
    return sorted_polynomials;
}

} // namespace bb::plonk
