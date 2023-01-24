#include "composer_helper.hpp"
#include "permutation_helper.hpp"
#include <polynomials/polynomial.hpp>
#include <proof_system/flavor/flavor.hpp>
#include <honk/pcs/commitment_key.hpp>
#include <numeric/bitop/get_msb.hpp>

#include <cstddef>
#include <cstdint>
#include <string>

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
    const CircuitConstructor& constructor, const size_t minimum_circuit_size, const size_t num_randomized_gates)
{
    const size_t num_gates = constructor.num_gates;
    std::span<const uint32_t> public_inputs = constructor.public_inputs;

    const size_t num_public_inputs = public_inputs.size();
    const size_t num_constraints = num_gates + num_public_inputs;
    const size_t total_num_constraints = std::max(minimum_circuit_size, num_constraints);
    const size_t subgroup_size =
        constructor.get_circuit_subgroup_size(total_num_constraints + num_randomized_gates); // next power of 2

    auto crs = crs_factory_->get_prover_crs(subgroup_size + 1);

    // Initialize circuit_proving_key
    // TODO: replace composer types.
    circuit_proving_key = std::make_shared<waffle::proving_key>(
        subgroup_size, num_public_inputs, crs, waffle::ComposerType::STANDARD_HONK);

    for (size_t j = 0; j < constructor.num_selectors; ++j) {
        std::span<const barretenberg::fr> selector_values = constructor.selectors[j];
        ASSERT(num_gates == selector_values.size());

        // Compute selector vector, initialized to 0.
        // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
        // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
        polynomial selector_poly_lagrange(subgroup_size, subgroup_size);
        for (size_t i = 0; i < num_gates; ++i) {
            selector_poly_lagrange[num_public_inputs + i] = selector_values[i];
        }
        // TODO(Adrian): We may want to add a unique value (e.g. j+1) in the last position of each selector polynomial
        // to guard against some edge cases that may occur during the MSM.
        // If we do so, we should ensure that this does not clash with any other values we want to place at the end of
        // of the witness vectors.
        // In later iterations of the Sumcheck, we will be able to efficiently cancel out any checks in the last 2^k
        // rows, so any randomness or unique values should be placed there.

        circuit_proving_key->polynomial_cache.put(constructor.selector_names_[j] + "_lagrange",
                                                  std::move(selector_poly_lagrange));
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
    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    auto commitment_key = pcs::kzg::CommitmentKey(proving_key->n, "../srs_db/ignition");

    for (size_t i = 0; i < proving_key->polynomial_manifest.size(); ++i) {
        const auto& selector_poly_info = proving_key->polynomial_manifest[i];

        const std::string selector_poly_label(selector_poly_info.polynomial_label);
        const std::string selector_commitment_label(selector_poly_info.commitment_label);

        if (selector_poly_info.source == waffle::PolynomialSource::SELECTOR) {
            // Fetch the constraint selector polynomial in its vector form.

            fr* selector_poly_coefficients;
            selector_poly_coefficients = proving_key->polynomial_cache.get(selector_poly_label).get_coefficients();

            // Commit to the constraint selector polynomial and insert the commitment in the verification key.

            auto selector_poly_commitment = commitment_key.commit({ selector_poly_coefficients, proving_key->n });
            circuit_verification_key->constraint_selectors.insert(
                { selector_commitment_label, selector_poly_commitment });

        } else if (selector_poly_info.source == waffle::PolynomialSource::PERMUTATION) {
            // Fetch the permutation selector polynomial in its coefficient form.
            fr* selector_poly_coefficients;
            selector_poly_coefficients = proving_key->polynomial_cache.get(selector_poly_label).get_coefficients();

            // Commit to the permutation selector polynomial insert the commitment in the verification key.

            auto selector_poly_commitment = commitment_key.commit({ selector_poly_coefficients, proving_key->n });

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
void ComposerHelper<CircuitConstructor>::compute_witness_base(const CircuitConstructor& circuit_constructor,
                                                              const size_t minimum_circuit_size)
{
    if (computed_witness) {
        return;
    }
    const size_t num_gates = circuit_constructor.num_gates;
    std::span<const uint32_t> public_inputs = circuit_constructor.public_inputs;
    const size_t num_public_inputs = public_inputs.size();

    const size_t num_constraints = std::max(minimum_circuit_size, num_gates + num_public_inputs);
    // TODO(Adrian): Not a fan of specifying NUM_RANDOMIZED_GATES everywhere,
    // Each flavor of Honk should have a "fixed" number of random places to add randomness to.
    // It should be taken care of in as few places possible.
    const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size(num_constraints + NUM_RANDOMIZED_GATES);

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

    // Note: randomness is added to 3 of the last 4 positions in plonk/proof_system/prover/prover.cpp
    // StandardProverBase::execute_preamble_round().
    for (size_t j = 0; j < program_width; ++j) {
        // Initialize the polynomial with all the actual copies variable values
        // Expect all values to be set to 0 initially
        polynomial w_lagrange(subgroup_size, subgroup_size);

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
        std::string index = std::to_string(j + 1);
        circuit_proving_key->polynomial_cache.put("w_" + index + "_lagrange", std::move(w_lagrange));
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
    const CircuitConstructor& circuit_constructor)
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
    circuit_proving_key->polynomial_cache.put("z_perm_lagrange", Polynomial<barretenberg::fr>(1));

    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
template <typename CircuitConstructor>
std::shared_ptr<waffle::verification_key> ComposerHelper<CircuitConstructor>::compute_verification_key(
    const CircuitConstructor& circuit_constructor)
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
StandardVerifier ComposerHelper<CircuitConstructor>::create_verifier(const CircuitConstructor& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);
    // TODO figure out types, actually
    // circuit_verification_key->composer_type = type;

    // TODO: initialize verifier according to manifest and key
    // Verifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));
    StandardVerifier output_state;
    // TODO: Do we need a commitment scheme defined here?
    // std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
    //    std::make_unique<KateCommitmentScheme<standard_settings>>();

    // output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

template <typename CircuitConstructor>
StandardUnrolledVerifier ComposerHelper<CircuitConstructor>::create_unrolled_verifier(
    const CircuitConstructor& circuit_constructor)
{
    compute_verification_key(circuit_constructor);
    StandardUnrolledVerifier output_state(
        circuit_verification_key,
        honk::StandardHonk::create_unrolled_manifest(circuit_constructor.public_inputs.size(),
                                                     numeric::get_msb(circuit_verification_key->n)));
    // StandardUnrolledVerifier output_state;

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
    const CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    size_t num_sumcheck_rounds(circuit_proving_key->log_n);
    auto manifest = Flavor::create_unrolled_manifest(circuit_constructor.public_inputs.size(), num_sumcheck_rounds);
    StandardUnrolledProver output_state(circuit_proving_key, manifest);

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
StandardProver ComposerHelper<CircuitConstructor>::create_prover(const CircuitConstructor& circuit_constructor)
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
    const StandardCircuitConstructor& circuit_constructor);
} // namespace honk
