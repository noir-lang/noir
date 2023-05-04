#include "standard_honk_composer_helper.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/srs/reference_string/reference_string.hpp"

#include <cstddef>
#include <cstdint>
#include <string>

namespace proof_system::honk {

/**
 * Compute proving key base.
 *
 * 1. Load crs.
 * 2. Initialize this->proving_key.
 * 3. Create constraint selector polynomials from each of this composer's `selectors` vectors and add them to the
 * proving key.
 *
 * @param minimum_circuit_size Used as the total number of gates when larger than n + count of public inputs.
 * @param num_reserved_gates The number of reserved gates.
 * @return Pointer to the initialized proving key updated with selector polynomials.
 * */
std::shared_ptr<StandardHonkComposerHelper::ProvingKey> StandardHonkComposerHelper::compute_proving_key_base(
    const CircuitConstructor& constructor, const size_t minimum_circuit_size, const size_t num_randomized_gates)
{
    // Initialize proving_key
    // TODO(#392)(Kesha): replace composer types.
    proving_key = initialize_proving_key<Flavor>(
        constructor, crs_factory_.get(), minimum_circuit_size, num_randomized_gates, ComposerType::STANDARD_HONK);
    // Compute lagrange selectors
    construct_selector_polynomials<Flavor>(constructor, proving_key.get());

    return proving_key;
}

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */

std::shared_ptr<StandardHonkComposerHelper::VerificationKey> StandardHonkComposerHelper::compute_verification_key_base(
    std::shared_ptr<StandardHonkComposerHelper::ProvingKey> const& proving_key,
    std::shared_ptr<VerifierReferenceString> const& vrs)
{
    auto key = std::make_shared<VerificationKey>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs, proving_key->composer_type);
    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    auto commitment_key = pcs::kzg::CommitmentKey(proving_key->circuit_size, "../srs_db/ignition");

    // Compute and store commitments to all precomputed polynomials
    key->q_m = commitment_key.commit(proving_key->q_m);
    key->q_l = commitment_key.commit(proving_key->q_l);
    key->q_r = commitment_key.commit(proving_key->q_r);
    key->q_o = commitment_key.commit(proving_key->q_o);
    key->q_c = commitment_key.commit(proving_key->q_c);
    key->sigma_1 = commitment_key.commit(proving_key->sigma_1);
    key->sigma_2 = commitment_key.commit(proving_key->sigma_2);
    key->sigma_3 = commitment_key.commit(proving_key->sigma_3);
    key->id_1 = commitment_key.commit(proving_key->id_1);
    key->id_2 = commitment_key.commit(proving_key->id_2);
    key->id_3 = commitment_key.commit(proving_key->id_3);
    key->lagrange_first = commitment_key.commit(proving_key->lagrange_first);
    key->lagrange_last = commitment_key.commit(proving_key->lagrange_last);

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
void StandardHonkComposerHelper::compute_witness(const CircuitConstructor& circuit_constructor,
                                                 const size_t minimum_circuit_size)
{
    if (computed_witness) {
        return;
    }
    auto wire_polynomials =
        construct_wire_polynomials_base<Flavor>(circuit_constructor, minimum_circuit_size, NUM_RANDOMIZED_GATES);

    proving_key->w_l = wire_polynomials[0];
    proving_key->w_r = wire_polynomials[1];
    proving_key->w_o = wire_polynomials[2];

    computed_witness = true;
}

/**
 * Compute proving key.
 * Compute the polynomials q_l, q_r, etc. and sigma polynomial.
 *
 * @return Proving key with saved computed polynomials.
 * */

std::shared_ptr<StandardHonkComposerHelper::ProvingKey> StandardHonkComposerHelper::compute_proving_key(
    const CircuitConstructor& circuit_constructor)
{
    if (proving_key) {
        return proving_key;
    }
    // Compute q_l, q_r, q_o, etc polynomials
    StandardHonkComposerHelper::compute_proving_key_base(
        circuit_constructor, /*minimum_circuit_size=*/0, NUM_RANDOMIZED_GATES);

    // Compute sigma polynomials (we should update that late)
    compute_standard_honk_sigma_permutations<Flavor>(circuit_constructor, proving_key.get());
    compute_standard_honk_id_polynomials<Flavor>(proving_key.get());

    compute_first_and_last_lagrange_polynomials<Flavor>(proving_key.get());

    return proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
std::shared_ptr<StandardHonkComposerHelper::VerificationKey> StandardHonkComposerHelper::compute_verification_key(
    const CircuitConstructor& circuit_constructor)
{
    if (verification_key) {
        return verification_key;
    }
    if (!proving_key) {
        compute_proving_key(circuit_constructor);
    }

    verification_key =
        StandardHonkComposerHelper::compute_verification_key_base(proving_key, crs_factory_->get_verifier_crs());
    verification_key->composer_type = proving_key->composer_type;

    return verification_key;
}

StandardVerifier StandardHonkComposerHelper::create_verifier(const CircuitConstructor& circuit_constructor)
{
    compute_verification_key(circuit_constructor);
    StandardVerifier output_state(verification_key);

    // TODO(Cody): This should be more generic
    auto kate_verification_key = std::make_unique<pcs::kzg::VerificationKey>("../srs_db/ignition");

    output_state.kate_verification_key = std::move(kate_verification_key);

    return output_state;
}

StandardProver StandardHonkComposerHelper::create_prover(const CircuitConstructor& circuit_constructor)
{
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    StandardProver output_state(proving_key);

    return output_state;
}
} // namespace proof_system::honk
