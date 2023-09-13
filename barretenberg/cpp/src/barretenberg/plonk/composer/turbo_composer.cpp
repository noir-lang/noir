#include "turbo_composer.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/plonk/composer/composer_lib.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/plonk/proof_system/widgets/random_widgets/permutation_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/fixed_base_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/transition_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/turbo_arithmetic_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/turbo_logic_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/turbo_range_widget.hpp"
#include "barretenberg/proof_system/circuit_builder/turbo_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"

using namespace barretenberg;

namespace proof_system::plonk {

/**
 * Compute proving key
 *
 * 1. Load crs.
 * 2. Initialize this.circuit_proving_key.
 * 3. Create constraint selector polynomials from each of this composer's `selectors` vectors and add them to the
 * proving key.
 * 4. Compute sigma polynomial
 *
 * @return Pointer to the initialized proving key updated with selector polynomials.
 * */
std::shared_ptr<plonk::proving_key> TurboComposer::compute_proving_key(const CircuitBuilder& circuit_constructor)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    const size_t minimum_circuit_size = 0;
    const size_t num_randomized_gates = NUM_RESERVED_GATES;
    // Initialize circuit_proving_key
    // TODO(#392)(Kesha): replace composer types.
    circuit_proving_key = initialize_proving_key(
        circuit_constructor, crs_factory_.get(), minimum_circuit_size, num_randomized_gates, CircuitType::TURBO);

    construct_selector_polynomials<Flavor>(circuit_constructor, circuit_proving_key.get());

    enforce_nonzero_selector_polynomials(circuit_constructor, circuit_proving_key.get());

    compute_monomial_and_coset_selector_forms(circuit_proving_key.get(), turbo_selector_properties());

    // Compute sigma polynomials (TODO(kesha): we should update that late)
    compute_standard_plonk_sigma_permutations<Flavor>(circuit_constructor, circuit_proving_key.get());
    circuit_proving_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(circuit_constructor.recursive_proof_public_input_indices.begin(),
                              circuit_constructor.recursive_proof_public_input_indices.end());
    circuit_proving_key->contains_recursive_proof = circuit_constructor.contains_recursive_proof;
    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
std::shared_ptr<plonk::verification_key> TurboComposer::compute_verification_key(
    const CircuitBuilder& circuit_constructor)
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key(circuit_constructor);
    }

    circuit_verification_key =
        plonk::compute_verification_key_common(circuit_proving_key, crs_factory_->get_verifier_crs());
    circuit_verification_key->circuit_type = circuit_proving_key->circuit_type;
    circuit_verification_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(circuit_constructor.recursive_proof_public_input_indices.begin(),
                              circuit_constructor.recursive_proof_public_input_indices.end());
    circuit_verification_key->contains_recursive_proof = circuit_constructor.contains_recursive_proof;

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
void TurboComposer::compute_witness(const CircuitBuilder& circuit_constructor, const size_t minimum_circuit_size)
{

    if (computed_witness) {
        return;
    }
    const size_t num_gates = circuit_constructor.num_gates;
    const size_t num_public_inputs = circuit_constructor.public_inputs.size();

    const size_t num_constraints = std::max(minimum_circuit_size, num_gates + num_public_inputs);
    const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size(num_constraints + NUM_RESERVED_GATES);

    auto wire_polynomial_evaluations = construct_wire_polynomials_base<Flavor>(circuit_constructor, subgroup_size);

    for (size_t j = 0; j < program_width; ++j) {
        std::string index = std::to_string(j + 1);
        circuit_proving_key->polynomial_store.put("w_" + index + "_lagrange",
                                                  std::move(wire_polynomial_evaluations[j]));
    }
    computed_witness = true;
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
plonk::TurboProver TurboComposer::create_prover(const CircuitBuilder& circuit_constructor)
{
    // Compute q_l, etc. and sigma polynomials.
    compute_proving_key(circuit_constructor);

    // Compute witness polynomials.
    compute_witness(circuit_constructor);

    plonk::TurboProver output_state(circuit_proving_key, create_manifest(circuit_constructor.public_inputs.size()));

    auto permutation_widget = std::make_unique<ProverPermutationWidget<4, false>>(circuit_proving_key.get());

    auto arithmetic_widget = std::make_unique<ProverTurboArithmeticWidget<turbo_settings>>(circuit_proving_key.get());
    auto fixed_base_widget = std::make_unique<ProverTurboFixedBaseWidget<turbo_settings>>(circuit_proving_key.get());
    auto range_widget = std::make_unique<ProverTurboRangeWidget<turbo_settings>>(circuit_proving_key.get());
    auto logic_widget = std::make_unique<ProverTurboLogicWidget<turbo_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(range_widget));
    output_state.transition_widgets.emplace_back(std::move(logic_widget));

    std::unique_ptr<KateCommitmentScheme<turbo_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<turbo_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */
plonk::TurboVerifier TurboComposer::create_verifier(const CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    plonk::TurboVerifier output_state(circuit_verification_key,
                                      create_manifest(circuit_constructor.public_inputs.size()));

    std::unique_ptr<plonk::KateCommitmentScheme<plonk::turbo_settings>> kate_commitment_scheme =
        std::make_unique<plonk::KateCommitmentScheme<plonk::turbo_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

} // namespace proof_system::plonk
