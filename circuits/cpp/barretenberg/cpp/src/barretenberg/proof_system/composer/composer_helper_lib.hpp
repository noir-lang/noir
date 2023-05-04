#pragma once
#include <memory>
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"

namespace proof_system {

/**
 * @brief Initilalize proving key and load the crs
 *
 * @tparam Flavor
 * @param circuit_constructor  Object containing the circuit
 * @param crs_factory Produces the prover's reference string
 * @param minimum_circuit_size The minimum size of polynomials without randomized elements
 * @param num_randomized_gates Number of gates with randomized witnesses
 * @param composer_type The type of composer we are using
 * @return std::shared_ptr<typename Flavor::ProvingKey>
 */
template <typename Flavor>
std::shared_ptr<typename Flavor::ProvingKey> initialize_proving_key(
    const typename Flavor::CircuitConstructor& circuit_constructor,
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

    return std::make_shared<typename Flavor::ProvingKey>(subgroup_size, num_public_inputs, crs, composer_type);
}

/**
 * @brief Construct selector polynomials from ciruit selector information and put into polynomial cache
 *
 * @tparam Flavor
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
template <typename Flavor>
void construct_selector_polynomials(const typename Flavor::CircuitConstructor& circuit_constructor,
                                    typename Flavor::ProvingKey* proving_key)
{
    const size_t num_public_inputs = circuit_constructor.public_inputs.size();
    // TODO(#398): Loose coupling here! Would rather build up pk from arithmetization
    size_t selector_idx = 0; // TODO(#391) zip
    for (auto& selector_values : circuit_constructor.selectors) {
        ASSERT(proving_key->circuit_size >= selector_values.size());

        // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
        // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
        barretenberg::polynomial selector_poly_lagrange(proving_key->circuit_size);
        for (size_t i = 0; i < selector_values.size(); ++i) {
            selector_poly_lagrange[num_public_inputs + i] = selector_values[i];
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
 * @brief Fill the last index of each selector polynomial in lagrange form with a non-zero value
 *
 * @tparam Flavor
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 *
 * @warning We should ensure that this does not clash with any other values we want to place at the end of of the
 * witness vectors. In later iterations of the Sumcheck, we will be able to efficiently cancel out any checks in the
 * last 2^k rows, so any randomness or unique values should be placed there. -@adr1anh
 */
template <typename Flavor>
void enforce_nonzero_selector_polynomials(const typename Flavor::CircuitConstructor& circuit_constructor,
                                          typename Flavor::ProvingKey* proving_key)
{
    if constexpr (IsHonkFlavor<Flavor>) {
        size_t idx = 1;
        for (auto selector : proving_key->get_selectors()) {
            selector[selector.size() - 1] = idx;
            ++idx;
        }
    } else if constexpr (IsPlonkFlavor<Flavor>) {
        for (size_t idx = 0; idx < circuit_constructor.num_selectors; ++idx) {
            auto current_selector =
                proving_key->polynomial_store.get(circuit_constructor.selector_names_[idx] + "_lagrange");
            current_selector[current_selector.size() - 1] = idx + 1;
            proving_key->polynomial_store.put(circuit_constructor.selector_names_[idx] + "_lagrange",
                                              std::move(current_selector));
        }
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
 * @param minimum_circuit_size
 * @param number_of_randomized_gates
 *
 * @return std::vector<barretenberg::polynomial>
 * */
template <typename Flavor>
std::vector<barretenberg::polynomial> construct_wire_polynomials_base(
    const typename Flavor::CircuitConstructor& circuit_constructor,
    const size_t minimum_circuit_size,
    const size_t number_of_randomized_gates)
{
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

    std::vector<barretenberg::polynomial> wire_polynomials;
    // Note: randomness is added to 3 of the last 4 positions in plonk/proof_system/prover/prover.cpp
    // StandardProverBase::execute_preamble_round().
    size_t wire_idx = 0; // TODO(#391) zip
    for (auto& wire : circuit_constructor.wires) {
        // Initialize the polynomial with all the actual copies variable values
        // Expect all values to be set to 0 initially
        barretenberg::polynomial w_lagrange(subgroup_size);

        // Place all public inputs at the start of the first two wires.
        // All selectors at these indices are set to 0, so these values are not constrained at all.
        if (wire_idx < 2) {
            for (size_t i = 0; i < num_public_inputs; ++i) {
                w_lagrange[i] = circuit_constructor.get_variable(public_inputs[i]);
            }
            ++wire_idx;
        }

        // Assign the variable values (which are pointed-to by the `w_` wire_polynomials) to the wire witness
        // polynomials `poly_w_`, shifted to make room for the public inputs at the beginning.
        for (size_t i = 0; i < num_gates; ++i) {
            w_lagrange[num_public_inputs + i] = circuit_constructor.get_variable(wire[i]);
        }
        wire_polynomials.push_back(std::move(w_lagrange));
    }
    return wire_polynomials;
}
} // namespace proof_system
