#include "ultra_composer.hpp"
#include "barretenberg/plonk/composer/composer_lib.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/plonk_honk_shared/composer/permutation_lib.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

#include <cstddef>
#include <cstdint>
#include <string>

namespace bb::plonk {

void UltraComposer::construct_sorted_polynomials(CircuitBuilder& circuit, size_t subgroup_size)
{
    // Save space in the sorted list polynomials for randomness (zk) plus one additional spot used to ensure the polys
    // aren't identically 0.
    size_t additional_offset = s_randomness + 1;
    auto sorted_polynomials = construct_sorted_list_polynomials<Flavor>(circuit, subgroup_size, additional_offset);

    circuit_proving_key->polynomial_store.put("s_1_lagrange", std::move(sorted_polynomials[0]));
    circuit_proving_key->polynomial_store.put("s_2_lagrange", std::move(sorted_polynomials[1]));
    circuit_proving_key->polynomial_store.put("s_3_lagrange", std::move(sorted_polynomials[2]));
    circuit_proving_key->polynomial_store.put("s_4_lagrange", std::move(sorted_polynomials[3]));
}

/**
 * @brief Compute proving key and construct an Ultra Prover
 */
UltraProver UltraComposer::create_prover(CircuitBuilder& circuit)
{
    compute_proving_key(circuit);

    return construct_prover<ultra_settings>(circuit);
}

/**
 * @brief Compute proving key and construct an UltraToStandardProver Prover
 */
UltraToStandardProver UltraComposer::create_ultra_to_standard_prover(CircuitBuilder& circuit_constructor)
{
    compute_proving_key(circuit_constructor);

    return construct_prover<ultra_to_standard_settings>(circuit_constructor);
}

/**
 * @brief Compute proving key and construct an UltraWithKeccakProver Prover
 */
UltraWithKeccakProver UltraComposer::create_ultra_with_keccak_prover(CircuitBuilder& circuit_constructor)
{
    compute_proving_key(circuit_constructor);

    return construct_prover<ultra_with_keccak_settings>(circuit_constructor);
}

/**
 * @brief Construct a Prover of given settings and populate it with the appropriate widgets
 *
 * @tparam settings
 * @param circuit_constructor
 * @return ProverBase<settings>
 */
template <typename settings> ProverBase<settings> UltraComposer::construct_prover(CircuitBuilder& circuit_constructor)
{
    ProverBase<settings> prover{ circuit_proving_key, create_manifest(circuit_constructor.public_inputs.size()) };

    auto permutation_widget = std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());
    auto plookup_widget = std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());
    auto arithmetic_widget = std::make_unique<ProverPlookupArithmeticWidget<settings>>(circuit_proving_key.get());
    auto sort_widget = std::make_unique<ProverGenPermSortWidget<settings>>(circuit_proving_key.get());
    auto elliptic_widget = std::make_unique<ProverEllipticWidget<settings>>(circuit_proving_key.get());
    auto auxiliary_widget = std::make_unique<ProverPlookupAuxiliaryWidget<settings>>(circuit_proving_key.get());

    prover.random_widgets.emplace_back(std::move(permutation_widget));
    prover.random_widgets.emplace_back(std::move(plookup_widget));

    prover.transition_widgets.emplace_back(std::move(arithmetic_widget));
    prover.transition_widgets.emplace_back(std::move(sort_widget));
    prover.transition_widgets.emplace_back(std::move(elliptic_widget));
    prover.transition_widgets.emplace_back(std::move(auxiliary_widget));

    prover.commitment_scheme = std::make_unique<KateCommitmentScheme<ultra_settings>>();

    return prover;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */

plonk::UltraVerifier UltraComposer::create_verifier(CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    plonk::UltraVerifier output_state(circuit_verification_key,
                                      create_manifest(circuit_constructor.public_inputs.size()));

    output_state.commitment_scheme = std::make_unique<plonk::KateCommitmentScheme<plonk::ultra_settings>>();

    return output_state;
}

/**
 * @brief Create a verifier using pedersen hash for the transcript
 *
 * @param circuit_constructor
 * @return UltraToStandardVerifier
 */
UltraToStandardVerifier UltraComposer::create_ultra_to_standard_verifier(CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    UltraToStandardVerifier output_state(circuit_verification_key,
                                         create_manifest(circuit_constructor.public_inputs.size()));

    output_state.commitment_scheme = std::make_unique<KateCommitmentScheme<ultra_to_standard_settings>>();

    return output_state;
}

/**
 * @brief Create a verifier using keccak for the transcript
 *
 * @param circuit_constructor
 * @return UltraWithKeccakVerifier
 */
UltraWithKeccakVerifier UltraComposer::create_ultra_with_keccak_verifier(CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    UltraWithKeccakVerifier output_state(circuit_verification_key,
                                         create_manifest(circuit_constructor.public_inputs.size()));

    output_state.commitment_scheme = std::make_unique<KateCommitmentScheme<ultra_with_keccak_settings>>();

    return output_state;
}

size_t UltraComposer::compute_dyadic_circuit_size(CircuitBuilder& circuit)
{
    const size_t filled_gates = circuit.num_gates + circuit.public_inputs.size();
    const size_t size_required_for_lookups = circuit.get_tables_size() + circuit.get_lookups_size();
    const size_t total_num_gates = std::max(filled_gates, size_required_for_lookups);
    return circuit.get_circuit_subgroup_size(total_num_gates + NUM_RESERVED_GATES);
}

std::shared_ptr<proving_key> UltraComposer::compute_proving_key(CircuitBuilder& circuit)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }

    circuit.finalize_circuit();

    const size_t subgroup_size = compute_dyadic_circuit_size(circuit);

    auto crs = srs::get_bn254_crs_factory()->get_prover_crs(subgroup_size + 1);
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/392): Composer type
    circuit_proving_key =
        std::make_shared<plonk::proving_key>(subgroup_size, circuit.public_inputs.size(), crs, CircuitType::ULTRA);

    // Construct and add to proving key the wire, selector and copy constraint polynomials
    Trace::populate(circuit, *circuit_proving_key);

    enforce_nonzero_selector_polynomials(circuit, circuit_proving_key.get());

    compute_monomial_and_coset_selector_forms(circuit_proving_key.get(), ultra_selector_properties());

    construct_table_polynomials(circuit, subgroup_size);

    // Instantiate z_lookup and s polynomials in the proving key (no values assigned yet).
    // Note: might be better to add these polys to cache only after they've been computed, as is convention
    polynomial z_lookup_fft(subgroup_size * 4);
    polynomial s_fft(subgroup_size * 4);
    circuit_proving_key->polynomial_store.put("z_lookup_fft", std::move(z_lookup_fft));
    circuit_proving_key->polynomial_store.put("s_fft", std::move(s_fft));

    circuit_proving_key->recursive_proof_public_input_indices = std::vector<uint32_t>(
        circuit.recursive_proof_public_input_indices.begin(), circuit.recursive_proof_public_input_indices.end());

    circuit_proving_key->contains_recursive_proof = circuit.contains_recursive_proof;

    construct_sorted_polynomials(circuit, subgroup_size);

    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */

std::shared_ptr<plonk::verification_key> UltraComposer::compute_verification_key(CircuitBuilder& circuit_constructor)
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }

    if (!circuit_proving_key) {
        compute_proving_key(circuit_constructor);
    }
    circuit_verification_key =
        compute_verification_key_common(circuit_proving_key, srs::get_bn254_crs_factory()->get_verifier_crs());

    circuit_verification_key->circuit_type = CircuitType::ULTRA;

    // See `add_recusrive_proof()` for how this recursive data is assigned.
    circuit_verification_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(circuit_constructor.recursive_proof_public_input_indices.begin(),
                              circuit_constructor.recursive_proof_public_input_indices.end());

    circuit_verification_key->contains_recursive_proof = circuit_constructor.contains_recursive_proof;

    circuit_verification_key->is_recursive_circuit = circuit_constructor.is_recursive_circuit;

    return circuit_verification_key;
}

void UltraComposer::add_table_column_selector_poly_to_proving_key(polynomial& selector_poly_lagrange_form,
                                                                  const std::string& tag)
{
    polynomial selector_poly_lagrange_form_copy(selector_poly_lagrange_form, circuit_proving_key->small_domain.size);

    selector_poly_lagrange_form.ifft(circuit_proving_key->small_domain);
    auto& selector_poly_coeff_form = selector_poly_lagrange_form;

    polynomial selector_poly_coset_form(selector_poly_coeff_form, circuit_proving_key->circuit_size * 4);
    selector_poly_coset_form.coset_fft(circuit_proving_key->large_domain);

    circuit_proving_key->polynomial_store.put(tag, std::move(selector_poly_coeff_form));
    circuit_proving_key->polynomial_store.put(tag + "_lagrange", std::move(selector_poly_lagrange_form_copy));
    circuit_proving_key->polynomial_store.put(tag + "_fft", std::move(selector_poly_coset_form));
}

void UltraComposer::construct_table_polynomials(CircuitBuilder& circuit, size_t subgroup_size)
{
    size_t additional_offset = s_randomness + 1;
    auto table_polynomials = construct_lookup_table_polynomials<Flavor>(circuit, subgroup_size, additional_offset);

    // // In the case of using UltraPlonkComposer for a circuit which does _not_ make use of any lookup tables, all four
    // // table columns would be all zeros. This would result in these polys' commitments all being the point at
    // infinity
    // // (which is bad because our point arithmetic assumes we'll never operate on the point at infinity). To avoid
    // this,
    // // we set the last evaluation of each poly to be nonzero. The last `num_roots_cut_out_of_vanishing_poly = 4`
    // // evaluations are ignored by constraint checks; we arbitrarily choose the very-last evaluation to be nonzero.
    // See
    // // ComposerBase::compute_proving_key_base for further explanation, as a similar trick is done there. We could
    // // have chosen `1` for each such evaluation here, but that would have resulted in identical commitments for
    // // all four columns. We don't want to have equal commitments, because biggroup operations assume no points are
    // // equal, so if we tried to verify an ultra proof in a circuit, the biggroup operations would fail. To combat
    // // this, we just choose distinct values:
    // ASSERT(offset == subgroup_size - 1);
    auto unique_last_value =
        get_num_selectors() + 1; // Note: in compute_proving_key_base, moments earlier, each selector
                                 // vector was given a unique last value from 1..num_selectors. So we
                                 // avoid those values and continue the count, to ensure uniqueness.
    table_polynomials[0][subgroup_size - 1] = unique_last_value;
    table_polynomials[1][subgroup_size - 1] = ++unique_last_value;
    table_polynomials[2][subgroup_size - 1] = ++unique_last_value;
    table_polynomials[3][subgroup_size - 1] = ++unique_last_value;

    add_table_column_selector_poly_to_proving_key(table_polynomials[0], "table_value_1");
    add_table_column_selector_poly_to_proving_key(table_polynomials[1], "table_value_2");
    add_table_column_selector_poly_to_proving_key(table_polynomials[2], "table_value_3");
    add_table_column_selector_poly_to_proving_key(table_polynomials[3], "table_value_4");
}
} // namespace bb::plonk
