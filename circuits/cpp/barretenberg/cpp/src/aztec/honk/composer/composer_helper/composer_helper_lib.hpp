#pragma once
#include <proof_system/proving_key/proving_key.hpp>
#include <proof_system/verification_key/verification_key.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <polynomials/polynomial.hpp>
namespace bonk {
struct SelectorProperties {
    std::string name;
    bool requires_lagrange_base_polynomial = false; // does the prover need the raw lagrange-base selector values?
};

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
                                                          plonk::ComposerType composer_type);

/**
 * @brief Construct lagrange selector polynomials from ciruit selector information and put into polynomial cache
 *
 * @tparam CircuitConstructor The class holding the circuit
 * @param circuit_constructor The object holding the circuit
 * @param key Pointer to the proving key
 */
template <typename CircuitConstructor>
void construct_lagrange_selector_forms(const CircuitConstructor& circuit_constructor, bonk::proving_key* key);

/**
 * @brief Retrieve lagrange forms of selector polynomials and compute monomial and coset-monomial forms and put into
 * cache
 *
 * @param key Pointer to the proving key
 * @param selector_properties Names of selectors
 */
void compute_monomial_and_coset_selector_forms(bonk::proving_key* key,
                                               std::vector<SelectorProperties> selector_properties);

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
                                                           const size_t number_of_randomized_gates);

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */
std::shared_ptr<bonk::verification_key> compute_verification_key_base_common(
    std::shared_ptr<bonk::proving_key> const& proving_key, std::shared_ptr<bonk::VerifierReferenceString> const& vrs);

} // namespace bonk