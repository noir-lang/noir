/**
 * @file composer_helper_lib.cpp
 * @brief Contains implementations of some of the functions used both by Honk and Plonk-style composers (excluding
 * permutation functions)
 *
 */
#include "composer_helper_lib.hpp"
#include <honk/pcs/commitment_key.hpp>
#include <honk/circuit_constructors/standard_circuit_constructor.hpp>
namespace bonk {

/**
 * @brief Initialize proving key and load the crs
 *
 * @tparam CircuitConstructor Class containing the circuit
 * @param circuit_constructor  Object containing the circuit
 * @param minimum_circuit_size The minimum size of polynomials without randomized elements
 * @param num_randomized_gates Number of gates with randomized witnesses
 * @param composer_type The type of composer we are using
 * @return std::shared_ptr<bonk::proving_key>
 */
template <typename CircuitConstructor>
std::shared_ptr<bonk::proving_key> initialize_proving_key(const CircuitConstructor& circuit_constructor,
                                                          bonk::ReferenceStringFactory* crs_factory,
                                                          const size_t minimum_circuit_size,
                                                          const size_t num_randomized_gates,
                                                          plonk::ComposerType composer_type)
{
    const size_t num_gates = circuit_constructor.num_gates;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;

    const size_t num_public_inputs = public_inputs.size();
    const size_t num_constraints = num_gates + num_public_inputs;
    const size_t total_num_constraints = std::max(minimum_circuit_size, num_constraints);
    const size_t subgroup_size =
        circuit_constructor.get_circuit_subgroup_size(total_num_constraints + num_randomized_gates); // next power of 2

    auto crs = crs_factory->get_prover_crs(subgroup_size + 1);

    return std::make_shared<bonk::proving_key>(subgroup_size, num_public_inputs, crs, composer_type);
}

/**
 * @brief Construct lagrange selector polynomials from ciruit selector information and put into polynomial cache
 *
 * @tparam CircuitConstructor The class holding the circuit
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
template <typename CircuitConstructor>
void put_selectors_in_polynomial_cache(const CircuitConstructor& circuit_constructor,
                                       bonk::proving_key* circuit_proving_key)
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
        // TODO(Adrian): We may want to add a unique value (e.g. j+1) in the last position of each selector polynomial
        // to guard against some edge cases that may occur during the MSM.
        // If we do so, we should ensure that this does not clash with any other values we want to place at the end of
        // of the witness vectors.
        // In later iterations of the Sumcheck, we will be able to efficiently cancel out any checks in the last 2^k
        // rows, so any randomness or unique values should be placed there.

        circuit_proving_key->polynomial_cache.put(circuit_constructor.selector_names_[j] + "_lagrange",
                                                  std::move(selector_poly_lagrange));
    }
}

/**
 * @brief Retrieve lagrange forms of selector polynomials and compute monomial and coset-monomial forms and put into
 * cache
 *
 * @param key Pointer to the proving key
 * @param selector_properties Names of selectors
 */
void compute_monomial_and_coset_selector_forms(bonk::proving_key* circuit_proving_key,
                                               std::vector<SelectorProperties> selector_properties)
{
    for (size_t i = 0; i < selector_properties.size(); i++) {
        // Compute monomial form of selector polynomial

        auto& selector_poly_lagrange =
            circuit_proving_key->polynomial_cache.get(selector_properties[i].name + "_lagrange");
        barretenberg::polynomial selector_poly(circuit_proving_key->circuit_size);
        barretenberg::polynomial_arithmetic::ifft(
            &selector_poly_lagrange[0], &selector_poly[0], circuit_proving_key->small_domain);

        // Compute coset FFT of selector polynomial
        barretenberg::polynomial selector_poly_fft(selector_poly, circuit_proving_key->circuit_size * 4 + 4);
        selector_poly_fft.coset_fft(circuit_proving_key->large_domain);

        // TODO(kesha): Delete lagrange polynomial from cache if it's not needed
        // if (selector_properties[i].requires_lagrange_base_polynomial) {
        //     key->polynomial_cache.put(selector_properties[i].name + "_lagrange", std::move(selector_poly_lagrange));
        // }
        circuit_proving_key->polynomial_cache.put(selector_properties[i].name, std::move(selector_poly));
        circuit_proving_key->polynomial_cache.put(selector_properties[i].name + "_fft", std::move(selector_poly_fft));
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
    // TODO(Adrian): Not a fan of specifying NUM_RANDOMIZED_GATES everywhere,
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

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */
std::shared_ptr<bonk::verification_key> compute_verification_key_base_common(
    std::shared_ptr<bonk::proving_key> const& proving_key, std::shared_ptr<bonk::VerifierReferenceString> const& vrs)
{
    auto circuit_verification_key = std::make_shared<bonk::verification_key>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs, proving_key->composer_type);
    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    auto commitment_key = honk::pcs::kzg::CommitmentKey(proving_key->circuit_size, "../srs_db/ignition");

    for (size_t i = 0; i < proving_key->polynomial_manifest.size(); ++i) {
        const auto& poly_info = proving_key->polynomial_manifest[i];

        const std::string poly_label(poly_info.polynomial_label);
        const std::string selector_commitment_label(poly_info.commitment_label);

        if (poly_info.source == bonk::PolynomialSource::SELECTOR ||
            poly_info.source == bonk::PolynomialSource::PERMUTATION ||
            poly_info.source == bonk::PolynomialSource::OTHER) {
            // Fetch the polynomial in its vector form.

            // Commit to the constraint selector polynomial and insert the commitment in the verification key.

            auto poly_commitment = commitment_key.commit(proving_key->polynomial_cache.get(poly_label));
            circuit_verification_key->commitments.insert({ selector_commitment_label, poly_commitment });
        }
    }

    // Set the polynomial manifest in verification key.
    circuit_verification_key->polynomial_manifest = bonk::PolynomialManifest(proving_key->composer_type);

    return circuit_verification_key;
}

template std::shared_ptr<bonk::proving_key> initialize_proving_key<StandardCircuitConstructor>(
    const StandardCircuitConstructor&, bonk::ReferenceStringFactory*, const size_t, const size_t, plonk::ComposerType);
template void put_selectors_in_polynomial_cache<StandardCircuitConstructor>(const StandardCircuitConstructor&,
                                                                            bonk::proving_key*);
template std::vector<barretenberg::polynomial> compute_witness_base<StandardCircuitConstructor>(
    const StandardCircuitConstructor&, const size_t, const size_t);

} // namespace bonk