#include "composer_helper.hpp"
#include "polynomials/polynomial.hpp"
#include <cstddef>
#include <proof_system/flavor/flavor.hpp>
#include <honk/pcs/commitment_key.hpp>
#include <numeric/bitop/get_msb.hpp>

namespace honk {

/**
 * Compute proving key base.
 *
 * 1. Load crs.
 * 2. Initialize this.circuit_proving_key.
 * 3. Create constraint selector polynomials from each of this composer's `selectors` vectors and add them to the
 * proving key.
 *
 * N.B. Need to add the fix for coefficients
 *
 * @param minimum_circuit_size Used as the total number of gates when larger than n + count of public inputs.
 * @param num_reserved_gates The number of reserved gates.
 * @return Pointer to the initialized proving key updated with selector polynomials.
 * */
template <typename CircuitConstructor>
std::shared_ptr<waffle::proving_key> ComposerHelper<CircuitConstructor>::compute_proving_key_base(
    CircuitConstructor& constructor, const size_t minimum_circuit_size, const size_t num_reserved_gates)
{
    /*
     * Map internal composer members for easier usage
     */

    auto& n = constructor.n;
    auto& public_inputs = constructor.public_inputs;
    auto& selector_names = constructor.selector_names_;
    auto& selectors = constructor.selectors;

    const size_t num_filled_gates = n + public_inputs.size();
    const size_t total_num_gates = std::max(minimum_circuit_size, num_filled_gates);
    const size_t subgroup_size =
        constructor.get_circuit_subgroup_size(total_num_gates + num_reserved_gates); // next power of 2

    auto crs = crs_factory_->get_prover_crs(subgroup_size + 1);

    // Initialize circuit_proving_key
    // TODO: replace composer types.
    circuit_proving_key = std::make_shared<waffle::proving_key>(
        subgroup_size, public_inputs.size(), crs, waffle::ComposerType::STANDARD_HONK);

    for (size_t i = 0; i < constructor.num_selectors; ++i) {
        std::vector<barretenberg::fr>& selector_values = selectors[i];
        ASSERT(n == selector_values.size());

        // Fill unfilled gates' selector values with zeroes (stopping 1 short; the last value will be nonzero).
        // TODO: (Do we want to copy the vectors and do it in a different place or do this iside the circuit itself?)
        for (size_t j = num_filled_gates; j < subgroup_size - 1; ++j) {
            selector_values.emplace_back(fr::zero());
        }

        // TODO: Now that we can't accomodate this, what do we do?
        // selector_values.emplace_back(i + 1);

        // Compute selector vector
        polynomial selector_poly_lagrange(subgroup_size);
        for (size_t k = 0; k < public_inputs.size(); ++k) {
            selector_poly_lagrange[k] = fr::zero();
        }
        for (size_t k = public_inputs.size(); k < subgroup_size; ++k) {
            selector_poly_lagrange[k] = selector_values[k - public_inputs.size()];
        }

        circuit_proving_key->polynomial_cache.put(selector_names[i] + "_lagrange", std::move(selector_poly_lagrange));
    }

    return circuit_proving_key;
}

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */

template <typename CircuitConstructor>
std::shared_ptr<waffle::verification_key> ComposerHelper<CircuitConstructor>::compute_verification_key_base(
    std::shared_ptr<waffle::proving_key> const& proving_key,
    std::shared_ptr<waffle::VerifierReferenceString> const& vrs)
{
    auto circuit_verification_key = std::make_shared<waffle::verification_key>(
        proving_key->n, proving_key->num_public_inputs, vrs, proving_key->composer_type);

    for (size_t i = 0; i < proving_key->polynomial_manifest.size(); ++i) {
        const auto& selector_poly_info = proving_key->polynomial_manifest[i];

        const std::string selector_poly_label(selector_poly_info.polynomial_label);
        const std::string selector_commitment_label(selector_poly_info.commitment_label);

        if (selector_poly_info.source == waffle::PolynomialSource::SELECTOR) {
            // Fetch the constraint selector polynomial in its vector form.
            // Disable for now so that GCC doesn't complain
            // TODO: restore when we actually implement the commitments
            // fr* selector_poly_coefficients;
            // selector_poly_coefficients = proving_key->polynomial_cache.get(selector_poly_label).get_coefficients();

            // Commit to the constraint selector polynomial and insert the commitment in the verification key.
            // TODO: Replace with actual commitment
            auto selector_poly_commitment = g1::affine_one;

            circuit_verification_key->constraint_selectors.insert(
                { selector_commitment_label, selector_poly_commitment });

        } else if (selector_poly_info.source == waffle::PolynomialSource::PERMUTATION) {
            // Fetch the permutation selector polynomial in its coefficient form.
            // Disable for now so that GCC doesn't complain
            // TODO: restore when we actually implement the commitments
            // fr* selector_poly_coefficients;
            // selector_poly_coefficients = proving_key->polynomial_cache.get(selector_poly_label).get_coefficients();

            // Commit to the permutation selector polynomial insert the commitment in the verification key.

            auto selector_poly_commitment = g1::affine_one;

            // TODO: Replace with actual commitment
            circuit_verification_key->permutation_selectors.insert(
                { selector_commitment_label, selector_poly_commitment });
        }
    }

    // Set the polynomial manifest in verification key.
    circuit_verification_key->polynomial_manifest = waffle::PolynomialManifest(proving_key->composer_type);

    return circuit_verification_key;
}

/**
 * Compute witness polynomials (w_1, w_2, w_3, w_4).
 *
 * @details Fills 3 or 4 witness polynomials w_1, w_2, w_3, w_4 with the values of in-circuit variables. The beginning
 * of w_1, w_2 polynomials is filled with public_input values.
 * @return Witness with computed witness polynomials.
 *
 * @tparam Program settings needed to establish if w_4 is being used.
 * */
template <typename CircuitConstructor>
template <size_t program_width>
void ComposerHelper<CircuitConstructor>::compute_witness_base(CircuitConstructor& circuit_constructor,
                                                              const size_t minimum_circuit_size)
{
    if (computed_witness) {
        return;
    }
    auto& n = circuit_constructor.n;
    auto& public_inputs = circuit_constructor.public_inputs;
    auto& w_l = circuit_constructor.w_l;
    auto& w_r = circuit_constructor.w_r;
    auto& w_o = circuit_constructor.w_o;
    auto& w_4 = circuit_constructor.w_4;
    auto zero_idx = circuit_constructor.zero_idx;

    const size_t total_num_gates = std::max(minimum_circuit_size, n + public_inputs.size());
    const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size(total_num_gates + NUM_RESERVED_GATES);

    // Note: randomness is added to 3 of the last 4 positions in plonk/proof_system/prover/prover.cpp
    // StandardProverBase::execute_preamble_round().
    for (size_t i = total_num_gates; i < subgroup_size; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
    }
    if (program_width > 3) {
        for (size_t i = total_num_gates; i < subgroup_size; ++i) {
            w_4.emplace_back(zero_idx);
        }
    }
    polynomial w_1_lagrange = polynomial(subgroup_size);
    polynomial w_2_lagrange = polynomial(subgroup_size);
    polynomial w_3_lagrange = polynomial(subgroup_size);
    polynomial w_4_lagrange;

    if (program_width > 3)
        w_4_lagrange = polynomial(subgroup_size);

    // Push the public inputs' values to the beginning of the wire witness polynomials.
    // Note: each public input variable is assigned to both w_1 and w_2. See
    // plonk/proof_system/public_inputs/public_inputs_impl.hpp for a giant comment explaining why.
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        fr::__copy(circuit_constructor.get_variable(public_inputs[i]), w_1_lagrange[i]);
        fr::__copy(circuit_constructor.get_variable(public_inputs[i]), w_2_lagrange[i]);
        fr::__copy(fr::zero(), w_3_lagrange[i]);
        if (program_width > 3)
            fr::__copy(fr::zero(), w_4_lagrange[i]);
    }

    // Assign the variable values (which are pointed-to by the `w_` wires) to the wire witness polynomials `poly_w_`,
    // shifted to make room for the public inputs at the beginning.
    for (size_t i = public_inputs.size(); i < total_num_gates; ++i) {
        fr::__copy(circuit_constructor.get_variable(w_l[i - public_inputs.size()]), w_1_lagrange.at(i));
        fr::__copy(circuit_constructor.get_variable(w_r[i - public_inputs.size()]), w_2_lagrange.at(i));
        fr::__copy(circuit_constructor.get_variable(w_o[i - public_inputs.size()]), w_3_lagrange.at(i));
        if (program_width > 3)
            fr::__copy(circuit_constructor.get_variable(w_4[i - public_inputs.size()]), w_4_lagrange.at(i));
    }

    circuit_proving_key->polynomial_cache.put("w_1_lagrange", std::move(w_1_lagrange));
    circuit_proving_key->polynomial_cache.put("w_2_lagrange", std::move(w_2_lagrange));
    circuit_proving_key->polynomial_cache.put("w_3_lagrange", std::move(w_3_lagrange));
    if (program_width > 3) {
        circuit_proving_key->polynomial_cache.put("w_4_lagrange", std::move(w_4_lagrange));
    }

    computed_witness = true;
}

/**
 * Compute proving key.
 * Compute the polynomials q_l, q_r, etc. and sigma polynomial.
 *
 * @return Proving key with saved computed polynomials.
 * */

template <typename CircuitConstructor>
std::shared_ptr<waffle::proving_key> ComposerHelper<CircuitConstructor>::compute_proving_key(
    CircuitConstructor& circuit_constructor)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    // Compute q_l, q_r, q_o, etc polynomials
    ComposerHelper::compute_proving_key_base(circuit_constructor, waffle::ComposerType::STANDARD_HONK);

    // Compute sigma polynomials (we should update that late)
    compute_standard_honk_sigma_permutations<CircuitConstructor::program_width>(circuit_constructor,
                                                                                circuit_proving_key.get());
    compute_standard_honk_id_polynomials<CircuitConstructor::program_width>(circuit_proving_key.get());

    compute_first_and_last_lagrange_polynomials(circuit_proving_key.get());

    // TODO(Cody): this is a workaround
    circuit_proving_key->polynomial_cache.put("z_perm", Polynomial<barretenberg::fr>(1));

    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
template <typename CircuitConstructor>
std::shared_ptr<waffle::verification_key> ComposerHelper<CircuitConstructor>::compute_verification_key(
    CircuitConstructor& circuit_constructor)
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key(circuit_constructor);
    }

    circuit_verification_key =
        ComposerHelper::compute_verification_key_base(circuit_proving_key, crs_factory_->get_verifier_crs());
    circuit_verification_key->composer_type = circuit_proving_key->composer_type;

    return circuit_verification_key;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */
template <typename CircuitConstructor>
waffle::Verifier ComposerHelper<CircuitConstructor>::create_verifier(CircuitConstructor& circuit_constructor)
{
    compute_verification_key(circuit_constructor);
    // TODO figure out types, actuallt
    // circuit_verification_key->composer_type = type;

    // TODO: initialize verifier according to manifest and key
    // Verifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));
    waffle::Verifier output_state;
    // TODO: Do we need a commitment scheme defined here?
    // std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
    //    std::make_unique<KateCommitmentScheme<standard_settings>>();

    // output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

template <typename CircuitConstructor>
waffle::UnrolledVerifier ComposerHelper<CircuitConstructor>::create_unrolled_verifier(
    CircuitConstructor& circuit_constructor)
{
    compute_verification_key(circuit_constructor);
    // UnrolledVerifier output_state(circuit_verification_key,
    //                               create_unrolled_manifest(circuit_constructor.n,
    //                               circuit_constructor.public_inputs.size()));
    waffle::UnrolledVerifier output_state;

    // TODO: Deal with commitments
    // std::unique_ptr<KateCommitmentScheme<unrolled_standard_settings>> kate_commitment_scheme =
    //     std::make_unique<KateCommitmentScheme<unrolled_standard_settings>>();

    // output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

template <typename CircuitConstructor>
template <typename Flavor>
// TODO(Cody): this file should be generic with regard to flavor/arithmetization/whatever.
StandardUnrolledProver ComposerHelper<CircuitConstructor>::create_unrolled_prover(
    CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    size_t num_sumcheck_rounds(circuit_proving_key->log_n);
    auto manifest = Flavor::create_unrolled_manifest(circuit_constructor.public_inputs.size(), num_sumcheck_rounds);
    StandardUnrolledProver output_state(circuit_proving_key, manifest);

    // TODO: Initialize constraints
    // std::unique_ptr<ProverPermutationWidget<3, false>> permutation_widget =
    //     std::make_unique<ProverPermutationWidget<3, false>>(circuit_proving_key.get());
    // std::unique_ptr<ProverArithmeticWidget<unrolled_standard_settings>> arithmetic_widget =
    //     std::make_unique<ProverArithmeticWidget<unrolled_standard_settings>>(circuit_proving_key.get());

    // output_state.random_widgets.emplace_back(std::move(permutation_widget));
    // output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));

    // TODO(Cody): This should be more generic
    std::unique_ptr<pcs::kzg::CommitmentKey> kate_commitment_key =
        std::make_unique<pcs::kzg::CommitmentKey>(circuit_proving_key->n, "../srs_db/ignition");

    output_state.commitment_key = std::move(kate_commitment_key);

    return output_state;
}
/**
 * Create prover.
 *  1. Compute the starting polynomials (q_l, etc, sigma, witness polynomials).
 *  2. Initialize StandardProver with them.
 *  3. Add Permutation and arithmetic widgets to the prover.
 *  4. Add KateCommitmentScheme to the prover.
 *
 * @return Initialized prover.
 * */
template <typename CircuitConstructor>
StandardProver ComposerHelper<CircuitConstructor>::create_prover(CircuitConstructor& circuit_constructor)
{
    // Compute q_l, etc. and sigma polynomials.
    compute_proving_key(circuit_constructor);

    // Compute witness polynomials.
    compute_witness(circuit_constructor);
    // TODO: Initialize prover properly
    // Prover output_state(circuit_proving_key, create_manifest(public_inputs.size()));
    StandardProver output_state;
    // Initialize constraints

    // std::unique_ptr<ProverPermutationWidget<3, false>> permutation_widget =
    //     std::make_unique<ProverPermutationWidget<3, false>>(circuit_proving_key.get());

    // std::unique_ptr<ProverArithmeticWidget<standard_settings>> arithmetic_widget =
    //     std::make_unique<ProverArithmeticWidget<standard_settings>>(circuit_proving_key.get());

    // output_state.random_widgets.emplace_back(std::move(permutation_widget));
    // output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));

    // Is commitment scheme going to stay a part of the prover? Why is it here?
    // std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
    //    std::make_unique<KateCommitmentScheme<standard_settings>>();

    // output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

template class ComposerHelper<StandardCircuitConstructor>;
template StandardUnrolledProver ComposerHelper<StandardCircuitConstructor>::create_unrolled_prover<StandardHonk>(
    StandardCircuitConstructor& circuit_constructor);
} // namespace honk
