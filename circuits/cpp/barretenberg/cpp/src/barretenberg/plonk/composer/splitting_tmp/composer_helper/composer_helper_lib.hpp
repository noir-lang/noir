#pragma once
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

namespace proof_system::plonk {
struct SelectorProperties {
    std::string name;
    // TODO: does the prover need the raw lagrange-base selector values?
    bool requires_lagrange_base_polynomial = false;
};

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
    std::shared_ptr<plonk::proving_key> const& proving_key, std::shared_ptr<VerifierReferenceString> const& vrs);

} // namespace proof_system::plonk
