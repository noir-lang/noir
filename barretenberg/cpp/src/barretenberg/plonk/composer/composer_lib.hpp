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

} // namespace bb::plonk
