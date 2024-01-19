#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/proof_system/polynomial_store/polynomial_store.hpp"

#include <memory>

namespace bb {

/**
 * @brief Construct selector polynomials from circuit selector information and put into polynomial cache
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
    size_t gate_offset = zero_row_offset + circuit_constructor.public_inputs.size();

    // If Goblin, (1) update the conventional gate offset to account for ecc op gates at the top of the execution trace,
    // and (2) construct ecc op gate selector polynomial. This selector is handled separately from the others since it
    // is computable based simply on num_ecc_op_gates and thus is not constructed explicitly in the builder.
    // Note 1: All other selectors will be automatically and correctly initialized to 0 on this domain.
    // Note 2: If applicable, the ecc op gates are shifted down by 1 to account for a zero row.
    if constexpr (IsGoblinFlavor<Flavor>) {
        const size_t num_ecc_op_gates = circuit_constructor.num_ecc_op_gates;
        gate_offset += num_ecc_op_gates;
        const size_t op_gate_offset = zero_row_offset;
        // The op gate selector is simply the indicator on the domain [offset, num_ecc_op_gates + offset - 1]
        bb::polynomial ecc_op_selector(proving_key->circuit_size);
        for (size_t i = 0; i < num_ecc_op_gates; ++i) {
            ecc_op_selector[i + op_gate_offset] = 1;
        }
        proving_key->lagrange_ecc_op = ecc_op_selector.share();
    }

    // TODO(#398): Loose coupling here! Would rather build up pk from arithmetization
    if constexpr (IsHonkFlavor<Flavor>) {
        for (auto [poly, selector_values] : zip_view(ZipAllowDifferentSizes::FLAG,
                                                     proving_key->get_precomputed_polynomials(),
                                                     circuit_constructor.selectors.get())) {
            ASSERT(proving_key->circuit_size >= selector_values.size());

            // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
            // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
            typename Flavor::Polynomial selector_poly_lagrange(proving_key->circuit_size);
            for (size_t i = 0; i < selector_values.size(); ++i) {
                selector_poly_lagrange[i + gate_offset] = selector_values[i];
            }
            poly = selector_poly_lagrange.share();
        }
    } else if constexpr (IsPlonkFlavor<Flavor>) {
        size_t selector_idx = 0;
        for (auto& selector_values : circuit_constructor.selectors.get()) {
            ASSERT(proving_key->circuit_size >= selector_values.size());

            // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
            // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
            typename Flavor::Polynomial selector_poly_lagrange(proving_key->circuit_size);
            for (size_t i = 0; i < selector_values.size(); ++i) {
                selector_poly_lagrange[i + gate_offset] = selector_values[i];
            }
            // TODO(Cody): Loose coupling here of selector_names and selector_properties.
            proving_key->polynomial_store.put(circuit_constructor.selector_names[selector_idx] + "_lagrange",
                                              std::move(selector_poly_lagrange));
            ++selector_idx;
        }
    }
}

/**
 * @brief Construct the witness polynomials from the witness vectors in the circuit constructor.
 *
 * @details We can think of the entries in the wire polynomials as reflecting the structure of the circuit execution
 * trace. The execution trace is broken up into several distinct blocks, depending on Flavor. For example, for Goblin
 * Ultra Honk, the order is: leading zero row, ECC op gates, public inputs, conventional wires. The actual values used
 * to populate the wire polynomials are determined by corresponding arrays of indices into the variables vector of the
 * circuit builder, and their location in the polynomials is determined by applying the appropriate "offset" for the
 * corresponding block.
 *
 * @tparam Flavor provides the circuit constructor type and the number of wires.
 * @param circuit_constructor
 * @param dyadic_circuit_size Power of 2 circuit size
 *
 * @return std::vector<typename Flavor::Polynomial>
 * */
template <typename Flavor>
std::vector<typename Flavor::Polynomial> construct_wire_polynomials_base(
    const typename Flavor::CircuitBuilder& circuit_constructor, const size_t dyadic_circuit_size)
{
    // Determine size of each block of data in the wire polynomials
    const size_t num_zero_rows = Flavor::has_zero_row ? 1 : 0;
    const size_t num_gates = circuit_constructor.num_gates;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;
    const size_t num_public_inputs = public_inputs.size();
    size_t num_ecc_op_gates = 0;
    if constexpr (IsGoblinFlavor<Flavor>) {
        num_ecc_op_gates = circuit_constructor.num_ecc_op_gates;
    }

    // Define offsets at which to start writing different blocks of data
    size_t op_gate_offset = num_zero_rows;
    size_t pub_input_offset = num_zero_rows + num_ecc_op_gates;
    size_t gate_offset = num_zero_rows + num_ecc_op_gates + num_public_inputs;

    std::vector<typename Flavor::Polynomial> wire_polynomials;

    // Populate the wire polynomials with values from ecc op gates, public inputs and conventional wires
    for (size_t wire_idx = 0; wire_idx < Flavor::NUM_WIRES; ++wire_idx) {

        // Expect all values to be set to 0 initially
        typename Flavor::Polynomial w_lagrange(dyadic_circuit_size);

        // Insert leading zero row into wire poly (for clarity; not stricly necessary due to zero-initialization)
        for (size_t i = 0; i < num_zero_rows; ++i) {
            w_lagrange[i] = circuit_constructor.get_variable(circuit_constructor.zero_idx);
        }

        // Insert ECC op wire values into wire polynomial
        if constexpr (IsGoblinFlavor<Flavor>) {
            for (size_t i = 0; i < num_ecc_op_gates; ++i) {
                auto& op_wire = circuit_constructor.ecc_op_wires[wire_idx];
                w_lagrange[i + op_gate_offset] = circuit_constructor.get_variable(op_wire[i]);
            }
        }

        // Insert public inputs (first two wire polynomials only)
        if (wire_idx < 2) {
            for (size_t i = 0; i < num_public_inputs; ++i) {
                w_lagrange[i + pub_input_offset] = circuit_constructor.get_variable(public_inputs[i]);
            }
        }

        // Insert conventional gate wire values into the wire polynomial
        for (size_t i = 0; i < num_gates; ++i) {
            auto& wire = circuit_constructor.wires[wire_idx];
            w_lagrange[i + gate_offset] = circuit_constructor.get_variable(wire[i]);
        }

        wire_polynomials.push_back(std::move(w_lagrange));
    }
    return wire_polynomials;
}
} // namespace bb
