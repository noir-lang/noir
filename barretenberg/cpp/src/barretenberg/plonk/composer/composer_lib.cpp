/**
 * @file composer_lib.cpp
 * @brief Contains some functions that are shared between the various Plonk composers.
 */
#include "composer_lib.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"

namespace bb::plonk {

/**
 * @brief Retrieve lagrange forms of selector polynomials and compute monomial and coset-monomial forms and put into
 * cache.
 *
 * @param key Pointer to the proving key
 * @param selector_properties Names of selectors
 */
void compute_monomial_and_coset_selector_forms(plonk::proving_key* circuit_proving_key,
                                               std::vector<SelectorProperties> selector_properties)
{
    for (size_t i = 0; i < selector_properties.size(); i++) {
        // Compute monomial form of selector polynomial
        auto selector_poly_lagrange =
            circuit_proving_key->polynomial_store.get(selector_properties[i].name + "_lagrange");
        bb::polynomial selector_poly(circuit_proving_key->circuit_size);
        bb::polynomial_arithmetic::ifft(
            &selector_poly_lagrange[0], &selector_poly[0], circuit_proving_key->small_domain);

        // Compute coset FFT of selector polynomial
        bb::polynomial selector_poly_fft(selector_poly, circuit_proving_key->circuit_size * 4 + 4);
        selector_poly_fft.coset_fft(circuit_proving_key->large_domain);

        // Note: For Standard, the lagrange polynomials could be removed from the store at this point but this
        // is not the case for Ultra.
        circuit_proving_key->polynomial_store.put(selector_properties[i].name, std::move(selector_poly));
        circuit_proving_key->polynomial_store.put(selector_properties[i].name + "_fft", std::move(selector_poly_fft));
    }
}

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */
std::shared_ptr<plonk::verification_key> compute_verification_key_common(
    std::shared_ptr<plonk::proving_key> const& proving_key,
    // Here too
    std::shared_ptr<bb::srs::factories::VerifierCrs<curve::BN254>> const& vrs)
{
    auto circuit_verification_key = std::make_shared<plonk::verification_key>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs, proving_key->circuit_type);
    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    using KZGCommitmentKey = honk::pcs::CommitmentKey<curve::BN254>;
    auto commitment_key = KZGCommitmentKey(proving_key->circuit_size, proving_key->reference_string);

    for (size_t i = 0; i < proving_key->polynomial_manifest.size(); ++i) {
        const auto& poly_info = proving_key->polynomial_manifest[i];

        const std::string poly_label(poly_info.polynomial_label);
        const std::string selector_commitment_label(poly_info.commitment_label);

        if (poly_info.source == PolynomialSource::SELECTOR || poly_info.source == PolynomialSource::PERMUTATION ||
            poly_info.source == PolynomialSource::OTHER) {
            // Fetch the polynomial in its vector form.

            // Commit to the constraint selector polynomial and insert the commitment in the verification key.

            auto poly_commitment = commitment_key.commit(proving_key->polynomial_store.get(poly_label));
            circuit_verification_key->commitments.insert({ selector_commitment_label, poly_commitment });
        }
    }

    // Set the polynomial manifest in verification key.
    circuit_verification_key->polynomial_manifest = bb::plonk::PolynomialManifest(proving_key->circuit_type);

    return circuit_verification_key;
}

} // namespace bb::plonk
