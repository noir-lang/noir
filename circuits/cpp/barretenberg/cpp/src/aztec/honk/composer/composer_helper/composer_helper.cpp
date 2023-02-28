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
std::shared_ptr<bonk::proving_key> ComposerHelper<CircuitConstructor>::compute_proving_key_base(
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
    circuit_proving_key =
        std::make_shared<bonk::proving_key>(subgroup_size, num_public_inputs, crs, plonk::ComposerType::STANDARD_HONK);

    for (size_t j = 0; j < constructor.num_selectors; ++j) {
        std::span<const barretenberg::fr> selector_values = constructor.selectors[j];
        ASSERT(num_gates == selector_values.size());

        // Compute selector vector, initialized to 0.
        // Copy the selector values for all gates, keeping the rows at which we store public inputs as 0.
        // Initializing the polynomials in this way automatically applies 0-padding to the selectors.
        polynomial selector_poly_lagrange(subgroup_size);
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
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */

template <typename CircuitConstructor>
std::shared_ptr<bonk::verification_key> ComposerHelper<CircuitConstructor>::compute_verification_key_base(
    std::shared_ptr<bonk::proving_key> const& proving_key, std::shared_ptr<bonk::VerifierReferenceString> const& vrs)
{
    auto key = std::make_shared<bonk::verification_key>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs, proving_key->composer_type);
    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    auto commitment_key = pcs::kzg::CommitmentKey(proving_key->circuit_size, "../srs_db/ignition");

    // Compute and store commitments to all precomputed polynomials
    key->commitments["Q_M"] = commitment_key.commit(proving_key->polynomial_cache.get("q_m_lagrange"));
    key->commitments["Q_1"] = commitment_key.commit(proving_key->polynomial_cache.get("q_1_lagrange"));
    key->commitments["Q_2"] = commitment_key.commit(proving_key->polynomial_cache.get("q_2_lagrange"));
    key->commitments["Q_3"] = commitment_key.commit(proving_key->polynomial_cache.get("q_3_lagrange"));
    key->commitments["Q_C"] = commitment_key.commit(proving_key->polynomial_cache.get("q_c_lagrange"));
    key->commitments["SIGMA_1"] = commitment_key.commit(proving_key->polynomial_cache.get("sigma_1_lagrange"));
    key->commitments["SIGMA_2"] = commitment_key.commit(proving_key->polynomial_cache.get("sigma_2_lagrange"));
    key->commitments["SIGMA_3"] = commitment_key.commit(proving_key->polynomial_cache.get("sigma_3_lagrange"));
    key->commitments["ID_1"] = commitment_key.commit(proving_key->polynomial_cache.get("id_1_lagrange"));
    key->commitments["ID_2"] = commitment_key.commit(proving_key->polynomial_cache.get("id_2_lagrange"));
    key->commitments["ID_3"] = commitment_key.commit(proving_key->polynomial_cache.get("id_3_lagrange"));
    key->commitments["LAGRANGE_FIRST"] = commitment_key.commit(proving_key->polynomial_cache.get("L_first_lagrange"));
    key->commitments["LAGRANGE_LAST"] = commitment_key.commit(proving_key->polynomial_cache.get("L_last_lagrange"));

    return key;
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
        // Construct wire polynomials in place
        auto& wire_lagrange = wire_polynomials.emplace_back(polynomial(subgroup_size));

        // Place all public inputs at the start of w_l and w_r.
        // All selectors at these indices are set to 0 so these values are not constrained at all.
        if ((j == 0) || (j == 1)) {
            for (size_t i = 0; i < num_public_inputs; ++i) {
                wire_lagrange[i] = circuit_constructor.get_variable(public_inputs[i]);
            }
        }

        // Assign the variable values (which are pointed-to by the `w_` wires) to the wire witness polynomials
        // `poly_w_`, shifted to make room for the public inputs at the beginning.
        for (size_t i = 0; i < num_gates; ++i) {
            wire_lagrange[num_public_inputs + i] = circuit_constructor.get_variable(w[j][i]);
        }
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
std::shared_ptr<bonk::proving_key> ComposerHelper<CircuitConstructor>::compute_proving_key(
    const CircuitConstructor& circuit_constructor)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    // Compute q_l, q_r, q_o, etc polynomials
    ComposerHelper::compute_proving_key_base(circuit_constructor, plonk::ComposerType::STANDARD_HONK);

    // Compute sigma polynomials (we should update that late)
    compute_standard_honk_sigma_permutations<CircuitConstructor::program_width>(circuit_constructor,
                                                                                circuit_proving_key.get());
    compute_standard_honk_id_polynomials<CircuitConstructor::program_width>(circuit_proving_key.get());

    compute_first_and_last_lagrange_polynomials(circuit_proving_key.get());

    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
template <typename CircuitConstructor>
std::shared_ptr<bonk::verification_key> ComposerHelper<CircuitConstructor>::compute_verification_key(
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

template <typename CircuitConstructor>
StandardVerifier ComposerHelper<CircuitConstructor>::create_verifier(const CircuitConstructor& circuit_constructor)
{
    compute_verification_key(circuit_constructor);
    StandardVerifier output_state(
        circuit_verification_key,
        honk::StandardHonk::create_manifest(circuit_constructor.public_inputs.size(),
                                            numeric::get_msb(circuit_verification_key->circuit_size)));

    // TODO(Cody): This should be more generic
    auto kate_verification_key = std::make_unique<pcs::kzg::VerificationKey>("../srs_db/ignition");

    output_state.kate_verification_key = std::move(kate_verification_key);

    return output_state;
}

template <typename CircuitConstructor>
template <typename Flavor>
// TODO(Cody): this file should be generic with regard to flavor/arithmetization/whatever.
StandardProver ComposerHelper<CircuitConstructor>::create_prover(const CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    size_t num_sumcheck_rounds(circuit_proving_key->log_circuit_size);
    auto manifest = Flavor::create_manifest(circuit_constructor.public_inputs.size(), num_sumcheck_rounds);
    StandardProver output_state(std::move(wire_polynomials), circuit_proving_key, manifest);

    // TODO(Cody): This should be more generic
    std::unique_ptr<pcs::kzg::CommitmentKey> kate_commitment_key =
        std::make_unique<pcs::kzg::CommitmentKey>(circuit_proving_key->circuit_size, "../srs_db/ignition");

    output_state.commitment_key = std::move(kate_commitment_key);

    return output_state;
}

template class ComposerHelper<StandardCircuitConstructor>;
template StandardProver ComposerHelper<StandardCircuitConstructor>::create_prover<StandardHonk>(
    const StandardCircuitConstructor& circuit_constructor);
} // namespace honk
