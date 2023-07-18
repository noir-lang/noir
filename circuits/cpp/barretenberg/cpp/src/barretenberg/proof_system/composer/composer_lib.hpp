#pragma once
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <memory>

namespace proof_system {

/**
 * @brief Construct selector polynomials from ciruit selector information and put into polynomial cache
 *
 * @tparam Flavor
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
template <typename Flavor>
void construct_selector_polynomials(const typename Flavor::CircuitBuilder& circuit_constructor,
                                    typename Flavor::ProvingKey* proving_key)
{
    // Offset for starting to write selectors is zero row offset + num public inputs
    const size_t zero_row_offset = Flavor::has_zero_row ? 1 : 0;
    const size_t gate_offset = zero_row_offset + circuit_constructor.public_inputs.size();
    // TODO(#398): Loose coupling here! Would rather build up pk from arithmetization
    size_t selector_idx = 0; // TODO(#391) zip
    for (auto& selector_values : circuit_constructor.selectors) {
        ASSERT(proving_key->circuit_size >= selector_values.size());

        // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
        // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
        barretenberg::polynomial selector_poly_lagrange(proving_key->circuit_size);
        for (size_t i = 0; i < selector_values.size(); ++i) {
            selector_poly_lagrange[i + gate_offset] = selector_values[i];
        }
        if constexpr (IsHonkFlavor<Flavor>) {
            // TODO(#398): Loose coupling here of arithmetization and flavor.
            proving_key->_precomputed_polynomials[selector_idx] = selector_poly_lagrange;
        } else if constexpr (IsPlonkFlavor<Flavor>) {
            // TODO(Cody): Loose coupling here of selector_names and selector_properties.
            proving_key->polynomial_store.put(circuit_constructor.selector_names_[selector_idx] + "_lagrange",
                                              std::move(selector_poly_lagrange));
        }
        ++selector_idx;
    }
}

/**
 * @brief Construct the witness polynomials from the witness vectors in the circuit constructor.
 *
 * @details The first two witness polynomials begin with the public input values.
 *
 *
 * @tparam Flavor provides the circuit constructor type and the number of wires.
 * @param circuit_constructor
 * @param dyadic_circuit_size Power of 2 circuit size
 *
 * @return std::vector<barretenberg::polynomial>
 * */
template <typename Flavor>
std::vector<barretenberg::polynomial> construct_wire_polynomials_base(
    const typename Flavor::CircuitBuilder& circuit_constructor, const size_t dyadic_circuit_size)
{
    const size_t zero_row_offset = Flavor::has_zero_row ? 1 : 0;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;
    const size_t num_public_inputs = public_inputs.size();

    std::vector<barretenberg::polynomial> wire_polynomials;
    // Note: randomness is added to 3 of the last 4 positions in plonk/proof_system/prover/prover.cpp
    // StandardProverBase::execute_preamble_round().
    size_t wire_idx = 0; // TODO(#391) zip
    for (auto& wire : circuit_constructor.wires) {
        // Initialize the polynomial with all the actual copies variable values
        // Expect all values to be set to 0 initially
        barretenberg::polynomial w_lagrange(dyadic_circuit_size);

        // Place all public inputs at the start of the first two wires, possibly offset by a zero row.
        // All selectors at these indices are set to 0, so these values are not constrained at all.
        const size_t pub_input_offset = zero_row_offset; // offset at which to start writing pub inputs
        if (wire_idx < 2) {
            for (size_t i = 0; i < num_public_inputs; ++i) {
                w_lagrange[i + pub_input_offset] = circuit_constructor.get_variable(public_inputs[i]);
            }
            ++wire_idx;
        }

        // Assign the variable values (which are pointed-to by the `w_` wire_polynomials) to the wire witness
        // polynomials `poly_w_`, shifted to make room for public inputs and the specified offset (possibly 0).
        const size_t gate_offset = num_public_inputs + pub_input_offset; // offset at which to start writing gates
        for (size_t i = 0; i < circuit_constructor.num_gates; ++i) {
            w_lagrange[i + gate_offset] = circuit_constructor.get_variable(wire[i]);
        }
        wire_polynomials.push_back(std::move(w_lagrange));
    }
    return wire_polynomials;
}
} // namespace proof_system
