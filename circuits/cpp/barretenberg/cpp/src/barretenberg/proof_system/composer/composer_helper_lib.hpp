#pragma once
#include <memory>
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"

namespace proof_system {

/**
 * @brief Initialize proving key and load the crs
 *
 * @tparam CircuitConstructor Class containing the circuit
 * @param circuit_constructor  Object containing the circuit
 * @param minimum_circuit_size The minimum size of polynomials without randomized elements
 * @param num_randomized_gates Number of gates with randomized witnesses
 * @param composer_type The type of composer we are using
 * @return std::shared_ptr<plonk::proving_key>
 */
template <typename CircuitConstructor>
std::shared_ptr<plonk::proving_key> initialize_proving_key(const CircuitConstructor& circuit_constructor,
                                                           ReferenceStringFactory* crs_factory,
                                                           const size_t minimum_circuit_size,
                                                           const size_t num_randomized_gates,
                                                           ComposerType composer_type)
{
    const size_t num_gates = circuit_constructor.num_gates;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;

    const size_t num_public_inputs = public_inputs.size();
    const size_t num_constraints = num_gates + num_public_inputs;
    const size_t total_num_constraints = std::max(minimum_circuit_size, num_constraints);
    const size_t subgroup_size =
        circuit_constructor.get_circuit_subgroup_size(total_num_constraints + num_randomized_gates); // next power of 2

    auto crs = crs_factory->get_prover_crs(subgroup_size + 1);

    return std::make_shared<plonk::proving_key>(subgroup_size, num_public_inputs, crs, composer_type);
}

/**
 * @brief Construct lagrange selector polynomials from ciruit selector information and put into polynomial cache
 *
 * @tparam CircuitConstructor The class holding the circuit
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
template <typename CircuitConstructor>
void construct_lagrange_selector_forms(const CircuitConstructor& circuit_constructor,
                                       plonk::proving_key* circuit_proving_key)
{
    const size_t num_public_inputs = circuit_constructor.public_inputs.size();
    for (size_t j = 0; j < circuit_constructor.num_selectors; ++j) {
        std::span<const barretenberg::fr> selector_values = circuit_constructor.selectors[j];
        ASSERT(circuit_proving_key->circuit_size >= selector_values.size());

        // Compute selector vector, initialized to 0.
        // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
        // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
        barretenberg::polynomial selector_poly_lagrange(circuit_proving_key->circuit_size);
        for (size_t i = 0; i < selector_values.size(); ++i) {
            selector_poly_lagrange[num_public_inputs + i] = selector_values[i];
        }
        // TODO(#217)(Adrian): We may want to add a unique value (e.g. j+1) in the last position of each selector
        // polynomial to guard against some edge cases that may occur during the MSM. If we do so, we should ensure that
        // this does not clash with any other values we want to place at the end of of the witness vectors. In later
        // iterations of the Sumcheck, we will be able to efficiently cancel out any checks in the last 2^k rows, so any
        // randomness or unique values should be placed there.
        circuit_proving_key->polynomial_store.put(circuit_constructor.selector_names_[j] + "_lagrange",
                                                  std::move(selector_poly_lagrange));
    }
}

/**
 * @brief Fill the last index of each selector polynomial in lagrange form with a non-zero value
 *
 * @tparam CircuitConstructor The class holding the circuit
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
template <typename CircuitConstructor>
void enforce_nonzero_polynomial_selectors(const CircuitConstructor& circuit_constructor,
                                          plonk::proving_key* circuit_proving_key)
{
    for (size_t j = 0; j < circuit_constructor.num_selectors; ++j) {
        auto current_selector =
            circuit_proving_key->polynomial_store.get(circuit_constructor.selector_names_[j] + "_lagrange");
        current_selector[current_selector.size() - 1] = j + 1;
        circuit_proving_key->polynomial_store.put(circuit_constructor.selector_names_[j] + "_lagrange",
                                                  std::move(current_selector));
    }
}

/**
 * Compute witness polynomials (w_1, w_2, w_3, w_4) and put them into polynomial cache
 *
 * @details Fills 3 or 4 witness polynomials w_1, w_2, w_3, w_4 with the values of in-circuit variables. The beginning
 * of w_1, w_2 polynomials is filled with public_input values.
 * @return Witness with computed witness polynomials.
 *
 * @tparam Program settings needed to establish if w_4 is being used.
 * */
template <typename CircuitConstructor>
std::vector<barretenberg::polynomial> compute_witness_base(const CircuitConstructor& circuit_constructor,
                                                           const size_t minimum_circuit_size,
                                                           const size_t number_of_randomized_gates)
{
    const size_t program_width = CircuitConstructor::program_width;
    const size_t num_gates = circuit_constructor.num_gates;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;
    const size_t num_public_inputs = public_inputs.size();

    const size_t num_constraints = std::max(minimum_circuit_size, num_gates + num_public_inputs);
    // TODO(#216)(Adrian): Not a fan of specifying NUM_RANDOMIZED_GATES everywhere,
    // Each flavor of Honk should have a "fixed" number of random places to add randomness to.
    // It should be taken care of in as few places possible.
    const size_t subgroup_size =
        circuit_constructor.get_circuit_subgroup_size(num_constraints + number_of_randomized_gates);

    // construct a view over all the wire's variable indices
    // w[j][i] is the index of the variable in the j-th wire, at gate i
    // Each array should be of size `num_gates`
    std::array<std::span<const uint32_t>, program_width> w;
    w[0] = circuit_constructor.w_l;
    w[1] = circuit_constructor.w_r;
    w[2] = circuit_constructor.w_o;
    if constexpr (program_width > 3) {
        w[3] = circuit_constructor.w_4;
    }
    std::vector<barretenberg::polynomial> wires;
    // Note: randomness is added to 3 of the last 4 positions in plonk/proof_system/prover/prover.cpp
    // StandardProverBase::execute_preamble_round().
    for (size_t j = 0; j < program_width; ++j) {
        // Initialize the polynomial with all the actual copies variable values
        // Expect all values to be set to 0 initially
        barretenberg::polynomial w_lagrange(subgroup_size);

        // Place all public inputs at the start of w_l and w_r.
        // All selectors at these indices are set to 0 so these values are not constrained at all.
        if ((j == 0) || (j == 1)) {
            for (size_t i = 0; i < num_public_inputs; ++i) {
                w_lagrange[i] = circuit_constructor.get_variable(public_inputs[i]);
            }
        }

        // Assign the variable values (which are pointed-to by the `w_` wires) to the wire witness polynomials
        // `poly_w_`, shifted to make room for the public inputs at the beginning.
        for (size_t i = 0; i < num_gates; ++i) {
            w_lagrange[num_public_inputs + i] = circuit_constructor.get_variable(w[j][i]);
        }
        wires.push_back(std::move(w_lagrange));
    }
    return wires;
}
} // namespace proof_system
