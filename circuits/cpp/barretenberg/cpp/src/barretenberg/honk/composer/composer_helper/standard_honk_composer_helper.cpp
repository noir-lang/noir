#include "standard_honk_composer_helper.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"

#include <cstddef>
#include <cstdint>
#include <string>

namespace proof_system::honk {

/**
 * Compute proving key base.
 *
 * 1. Load crs.
 * 2. Initialize this.circuit_proving_key.
 * 3. Create constraint selector polynomials from each of this composer's `selectors` vectors and add them to the
 * proving key.
 *
 * @param minimum_circuit_size Used as the total number of gates when larger than n + count of public inputs.
 * @param num_reserved_gates The number of reserved gates.
 * @return Pointer to the initialized proving key updated with selector polynomials.
 * */
template <typename CircuitConstructor>
std::shared_ptr<plonk::proving_key> StandardHonkComposerHelper<CircuitConstructor>::compute_proving_key_base(
    const CircuitConstructor& constructor, const size_t minimum_circuit_size, const size_t num_randomized_gates)
{
    // Initialize circuit_proving_key
    // TODO(#229)(Kesha): replace composer types.
    circuit_proving_key = initialize_proving_key(
        constructor, crs_factory_.get(), minimum_circuit_size, num_randomized_gates, ComposerType::STANDARD_HONK);
    // Compute lagrange selectors
    construct_lagrange_selector_forms(constructor, circuit_proving_key.get());

    return circuit_proving_key;
}

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */

template <typename CircuitConstructor>
std::shared_ptr<plonk::verification_key> StandardHonkComposerHelper<CircuitConstructor>::compute_verification_key_base(
    std::shared_ptr<plonk::proving_key> const& proving_key, std::shared_ptr<VerifierReferenceString> const& vrs)
{
    auto key = std::make_shared<plonk::verification_key>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs, proving_key->composer_type);
    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    auto commitment_key = pcs::kzg::CommitmentKey(proving_key->circuit_size, "../srs_db/ignition");

    // Compute and store commitments to all precomputed polynomials
    key->commitments["Q_M"] = commitment_key.commit(proving_key->polynomial_store.get("q_m_lagrange"));
    key->commitments["Q_1"] = commitment_key.commit(proving_key->polynomial_store.get("q_1_lagrange"));
    key->commitments["Q_2"] = commitment_key.commit(proving_key->polynomial_store.get("q_2_lagrange"));
    key->commitments["Q_3"] = commitment_key.commit(proving_key->polynomial_store.get("q_3_lagrange"));
    key->commitments["Q_C"] = commitment_key.commit(proving_key->polynomial_store.get("q_c_lagrange"));
    key->commitments["SIGMA_1"] = commitment_key.commit(proving_key->polynomial_store.get("sigma_1_lagrange"));
    key->commitments["SIGMA_2"] = commitment_key.commit(proving_key->polynomial_store.get("sigma_2_lagrange"));
    key->commitments["SIGMA_3"] = commitment_key.commit(proving_key->polynomial_store.get("sigma_3_lagrange"));
    key->commitments["ID_1"] = commitment_key.commit(proving_key->polynomial_store.get("id_1_lagrange"));
    key->commitments["ID_2"] = commitment_key.commit(proving_key->polynomial_store.get("id_2_lagrange"));
    key->commitments["ID_3"] = commitment_key.commit(proving_key->polynomial_store.get("id_3_lagrange"));
    key->commitments["LAGRANGE_FIRST"] = commitment_key.commit(proving_key->polynomial_store.get("L_first_lagrange"));
    key->commitments["LAGRANGE_LAST"] = commitment_key.commit(proving_key->polynomial_store.get("L_last_lagrange"));

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
void StandardHonkComposerHelper<CircuitConstructor>::compute_witness(const CircuitConstructor& circuit_constructor,
                                                                     const size_t minimum_circuit_size)
{
    if (computed_witness) {
        return;
    }
    wire_polynomials = compute_witness_base(circuit_constructor, minimum_circuit_size, NUM_RANDOMIZED_GATES);

    computed_witness = true;
}

/**
 * Compute proving key.
 * Compute the polynomials q_l, q_r, etc. and sigma polynomial.
 *
 * @return Proving key with saved computed polynomials.
 * */

template <typename CircuitConstructor>
std::shared_ptr<plonk::proving_key> StandardHonkComposerHelper<CircuitConstructor>::compute_proving_key(
    const CircuitConstructor& circuit_constructor)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    // Compute q_l, q_r, q_o, etc polynomials
    StandardHonkComposerHelper::compute_proving_key_base(circuit_constructor, ComposerType::STANDARD_HONK);

    // Compute sigma polynomials (we should update that late)
    compute_standard_honk_sigma_permutations<CircuitConstructor::num_wires>(circuit_constructor,
                                                                            circuit_proving_key.get());
    compute_standard_honk_id_polynomials<CircuitConstructor::num_wires>(circuit_proving_key.get());

    compute_first_and_last_lagrange_polynomials(circuit_proving_key.get());

    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
template <typename CircuitConstructor>
std::shared_ptr<plonk::verification_key> StandardHonkComposerHelper<CircuitConstructor>::compute_verification_key(
    const CircuitConstructor& circuit_constructor)
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key(circuit_constructor);
    }

    circuit_verification_key = StandardHonkComposerHelper::compute_verification_key_base(
        circuit_proving_key, crs_factory_->get_verifier_crs());
    circuit_verification_key->composer_type = circuit_proving_key->composer_type;

    return circuit_verification_key;
}

template <typename CircuitConstructor>
StandardVerifier StandardHonkComposerHelper<CircuitConstructor>::create_verifier(
    const CircuitConstructor& circuit_constructor)
{
    compute_verification_key(circuit_constructor);
    StandardVerifier output_state(circuit_verification_key);

    // TODO(Cody): This should be more generic
    auto kate_verification_key = std::make_unique<pcs::kzg::VerificationKey>("../srs_db/ignition");

    output_state.kate_verification_key = std::move(kate_verification_key);

    return output_state;
}

template <typename CircuitConstructor>
template <typename Flavor>
// TODO(Cody): this file should be generic with regard to flavor/arithmetization/whatever.
StandardProver StandardHonkComposerHelper<CircuitConstructor>::create_prover(
    const CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    size_t num_sumcheck_rounds(circuit_proving_key->log_circuit_size);
    // TODO(luke): what is this manifest? Remove?
    auto manifest = Flavor::create_manifest(circuit_constructor.public_inputs.size(), num_sumcheck_rounds);
    StandardProver output_state(std::move(wire_polynomials), circuit_proving_key);

    return output_state;
}
template class StandardHonkComposerHelper<StandardCircuitConstructor>;
template StandardProver StandardHonkComposerHelper<StandardCircuitConstructor>::create_prover<StandardHonk>(
    const StandardCircuitConstructor& circuit_constructor);
} // namespace proof_system::honk
