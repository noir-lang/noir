// #include "./ultra_verifier.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/ultra_recursive_verifier.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"

namespace proof_system::plonk::stdlib::recursion::honk {

template <typename Flavor>
UltraRecursiveVerifier_<Flavor>::UltraRecursiveVerifier_(Builder* builder,
                                                         std::shared_ptr<VerificationKey> verifier_key)
    : key(verifier_key)
    , builder(builder)
{}

/**
 * @brief This function constructs a recursive verifier circuit for an Ultra Honk proof of a given flavor.
 *
 */
template <typename Flavor>
std::array<typename Flavor::GroupElement, 2> UltraRecursiveVerifier_<Flavor>::verify_proof(const plonk::proof& proof)
{
    using Sumcheck = ::proof_system::honk::sumcheck::SumcheckVerifier<Flavor>;
    using Curve = typename Flavor::Curve;
    using Gemini = ::proof_system::honk::pcs::gemini::GeminiVerifier_<Curve>;
    using Shplonk = ::proof_system::honk::pcs::shplonk::ShplonkVerifier_<Curve>;
    using KZG = ::proof_system::honk::pcs::kzg::KZG<Curve>; // note: This can only be KZG
    using VerifierCommitments = typename Flavor::VerifierCommitments;
    using CommitmentLabels = typename Flavor::CommitmentLabels;
    using RelationParams = ::proof_system::RelationParameters<FF>;
    using UnivariateClaim = ::proof_system::honk::pcs::OpeningClaim<Curve>;

    RelationParams relation_parameters;

    info("Initial: num gates = ", builder->get_num_gates());
    size_t prev_num_gates = builder->get_num_gates();

    transcript = Transcript<Builder>{ builder, proof.proof_data };

    auto commitments = VerifierCommitments(key);
    auto commitment_labels = CommitmentLabels();

    const auto circuit_size = transcript.template receive_from_prover<uint32_t>("circuit_size");
    const auto public_input_size = transcript.template receive_from_prover<uint32_t>("public_input_size");
    const auto pub_inputs_offset = transcript.template receive_from_prover<uint32_t>("pub_inputs_offset");

    // For debugging purposes only
    ASSERT(static_cast<uint32_t>(circuit_size.get_value()) == key->circuit_size);
    ASSERT(static_cast<uint32_t>(public_input_size.get_value()) == key->num_public_inputs);

    std::vector<FF> public_inputs;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        auto public_input_i = transcript.template receive_from_prover<FF>("public_input_" + std::to_string(i));
        public_inputs.emplace_back(public_input_i);
    }

    // Get commitments to first three wire polynomials
    commitments.w_l = transcript.template receive_from_prover<Commitment>(commitment_labels.w_l);
    commitments.w_r = transcript.template receive_from_prover<Commitment>(commitment_labels.w_r);
    commitments.w_o = transcript.template receive_from_prover<Commitment>(commitment_labels.w_o);

    // If Goblin, get commitments to ECC op wire polynomials
    if constexpr (IsGoblinFlavor<Flavor>) {
        commitments.ecc_op_wire_1 =
            transcript.template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_1);
        commitments.ecc_op_wire_2 =
            transcript.template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_2);
        commitments.ecc_op_wire_3 =
            transcript.template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_3);
        commitments.ecc_op_wire_4 =
            transcript.template receive_from_prover<Commitment>(commitment_labels.ecc_op_wire_4);
    }

    // Get challenge for sorted list batching and wire four memory records
    auto eta = transcript.get_challenge("eta");
    relation_parameters.eta = eta;

    // Get commitments to sorted list accumulator and fourth wire
    commitments.sorted_accum = transcript.template receive_from_prover<Commitment>(commitment_labels.sorted_accum);
    commitments.w_4 = transcript.template receive_from_prover<Commitment>(commitment_labels.w_4);

    // Get permutation challenges
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    const FF public_input_delta = proof_system::honk::compute_public_input_delta<Flavor>(
        public_inputs, beta, gamma, circuit_size, static_cast<uint32_t>(pub_inputs_offset.get_value()));
    const FF lookup_grand_product_delta =
        proof_system::honk::compute_lookup_grand_product_delta<FF>(beta, gamma, circuit_size);

    relation_parameters.beta = beta;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = public_input_delta;
    relation_parameters.lookup_grand_product_delta = lookup_grand_product_delta;

    // Get commitment to permutation and lookup grand products
    commitments.z_perm = transcript.template receive_from_prover<Commitment>(commitment_labels.z_perm);
    commitments.z_lookup = transcript.template receive_from_prover<Commitment>(commitment_labels.z_lookup);

    // Execute Sumcheck Verifier
    auto sumcheck = Sumcheck(key->circuit_size);

    std::optional sumcheck_output = sumcheck.verify(relation_parameters, transcript);

    info("Sumcheck: num gates = ",
         builder->get_num_gates() - prev_num_gates,
         ", (total = ",
         builder->get_num_gates(),
         ")");
    prev_num_gates = builder->get_num_gates();

    // If Sumcheck does not return an output, sumcheck verification has failed
    ASSERT(sumcheck_output.has_value()); // TODO(luke): Appropriate way to handle this in circuit?

    // Extract multivariate opening point u = (u_0, ..., u_{d-1}) and purported multivariate evaluations at u
    auto [multivariate_challenge, purported_evaluations] = *sumcheck_output;

    // Compute powers of batching challenge rho
    FF rho = transcript.get_challenge("rho");
    std::vector<FF> rhos = ::proof_system::honk::pcs::gemini::powers_of_rho(rho, Flavor::NUM_ALL_ENTITIES);

    // Compute batched multivariate evaluation
    FF batched_evaluation = FF(0);
    size_t evaluation_idx = 0;
    for (auto& value : purported_evaluations.get_unshifted_then_shifted()) {
        batched_evaluation += value * rhos[evaluation_idx];
        ++evaluation_idx;
    }

    info("Batched eval: num gates = ",
         builder->get_num_gates() - prev_num_gates,
         ", (total = ",
         builder->get_num_gates(),
         ")");
    prev_num_gates = builder->get_num_gates();

    // Compute batched commitments needed for input to Gemini.
    // Note: For efficiency in emulating the construction of the batched commitments, we want to perform a batch mul
    // rather than naively accumulate the points one by one. To do this, we collect the points and scalars required for
    // each MSM then perform the two batch muls.
    const size_t NUM_UNSHIFTED = commitments.get_unshifted().size();
    const size_t NUM_TO_BE_SHIFTED = commitments.get_to_be_shifted().size();
    std::vector<FF> scalars_unshifted;
    std::vector<FF> scalars_to_be_shifted;
    size_t idx = 0;
    for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
        scalars_unshifted.emplace_back(rhos[idx++]);
    }
    for (size_t i = 0; i < NUM_TO_BE_SHIFTED; ++i) {
        scalars_to_be_shifted.emplace_back(rhos[idx++]);
    }
    // TODO(luke): The powers_of_rho fctn does not set the context of rhos[0] = FF(1) so we do it explicitly here. Can
    // we do something silly like set it to rho.pow(0) in the fctn to make it work both native and stdlib?
    scalars_unshifted[0] = FF::from_witness(builder, 1);

    // Batch the commitments to the unshifted and to-be-shifted polynomials using powers of rho
    auto batched_commitment_unshifted = GroupElement::batch_mul(commitments.get_unshifted(), scalars_unshifted);

    info("Batch mul (unshifted): num gates = ",
         builder->get_num_gates() - prev_num_gates,
         ", (total = ",
         builder->get_num_gates(),
         ")");
    prev_num_gates = builder->get_num_gates();

    auto batched_commitment_to_be_shifted =
        GroupElement::batch_mul(commitments.get_to_be_shifted(), scalars_to_be_shifted);

    info("Batch mul (to-be-shited): num gates = ",
         builder->get_num_gates() - prev_num_gates,
         ", (total = ",
         builder->get_num_gates(),
         ")");
    prev_num_gates = builder->get_num_gates();

    // Produce a Gemini claim consisting of:
    // - d+1 commitments [Fold_{r}^(0)], [Fold_{-r}^(0)], and [Fold^(l)], l = 1:d-1
    // - d+1 evaluations a_0_pos, and a_l, l = 0:d-1
    auto univariate_opening_claims = Gemini::reduce_verification(multivariate_challenge,
                                                                 batched_evaluation,
                                                                 batched_commitment_unshifted,
                                                                 batched_commitment_to_be_shifted,
                                                                 transcript);

    info("Gemini: num gates = ",
         builder->get_num_gates() - prev_num_gates,
         ", (total = ",
         builder->get_num_gates(),
         ")");
    prev_num_gates = builder->get_num_gates();

    // Perform ECC op queue transcript aggregation protocol
    if constexpr (IsGoblinFlavor<Flavor>) {
        // Receive commitments [t_i^{shift}], [T_{i-1}], and [T_i]
        std::array<Commitment, Flavor::NUM_WIRES> prev_agg_op_queue_commitments;
        std::array<Commitment, Flavor::NUM_WIRES> shifted_op_wire_commitments;
        std::array<Commitment, Flavor::NUM_WIRES> agg_op_queue_commitments;
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            std::string suffix = std::to_string(idx + 1);
            prev_agg_op_queue_commitments[idx] =
                transcript.template receive_from_prover<Commitment>("PREV_AGG_OP_QUEUE_" + suffix);
            shifted_op_wire_commitments[idx] =
                transcript.template receive_from_prover<Commitment>("SHIFTED_OP_WIRE_" + suffix);
            agg_op_queue_commitments[idx] =
                transcript.template receive_from_prover<Commitment>("AGG_OP_QUEUE_" + suffix);
        }

        // Receive claimed evaluations of t_i^{shift}, T_{i-1}, and T_i
        FF kappa = transcript.get_challenge("kappa");
        std::array<FF, Flavor::NUM_WIRES> prev_agg_op_queue_evals;
        std::array<FF, Flavor::NUM_WIRES> shifted_op_wire_evals;
        std::array<FF, Flavor::NUM_WIRES> agg_op_queue_evals;
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            std::string suffix = std::to_string(idx + 1);
            prev_agg_op_queue_evals[idx] =
                transcript.template receive_from_prover<FF>("prev_agg_op_queue_eval_" + suffix);
            shifted_op_wire_evals[idx] = transcript.template receive_from_prover<FF>("op_wire_eval_" + suffix);
            agg_op_queue_evals[idx] = transcript.template receive_from_prover<FF>("agg_op_queue_eval_" + suffix);

            ASSERT(agg_op_queue_evals[idx].get_value() ==
                   prev_agg_op_queue_evals[idx].get_value() + shifted_op_wire_evals[idx].get_value());

            // Check the identity T_i(\kappa) = T_{i-1}(\kappa) + t_i^{shift}(\kappa).
            agg_op_queue_evals[idx].assert_equal(prev_agg_op_queue_evals[idx] + shifted_op_wire_evals[idx]);
        }

        // Add corresponding univariate opening claims {(\kappa, p(\kappa), [p(X)]}
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            univariate_opening_claims.emplace_back(
                UnivariateClaim{ { kappa, prev_agg_op_queue_evals[idx] }, prev_agg_op_queue_commitments[idx] });
        }
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            univariate_opening_claims.emplace_back(
                UnivariateClaim{ { kappa, shifted_op_wire_evals[idx] }, shifted_op_wire_commitments[idx] });
        }
        for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
            univariate_opening_claims.emplace_back(
                UnivariateClaim{ { kappa, agg_op_queue_evals[idx] }, agg_op_queue_commitments[idx] });
        }
    }

    // Produce a Shplonk claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    auto shplonk_claim = Shplonk::reduce_verification(pcs_verification_key, univariate_opening_claims, transcript);

    info("Shplonk: num gates = ",
         builder->get_num_gates() - prev_num_gates,
         ", (total = ",
         builder->get_num_gates(),
         ")");
    prev_num_gates = builder->get_num_gates();

    // Constuct the inputs to the final KZG pairing check
    auto pairing_points = KZG::compute_pairing_points(shplonk_claim, transcript);

    info("KZG: num gates = ", builder->get_num_gates() - prev_num_gates, ", (total = ", builder->get_num_gates(), ")");

    return pairing_points;
}

template class UltraRecursiveVerifier_<proof_system::honk::flavor::UltraRecursive_<UltraCircuitBuilder>>;
template class UltraRecursiveVerifier_<proof_system::honk::flavor::UltraRecursive_<GoblinUltraCircuitBuilder>>;
template class UltraRecursiveVerifier_<proof_system::honk::flavor::GoblinUltraRecursive_<UltraCircuitBuilder>>;
template class UltraRecursiveVerifier_<proof_system::honk::flavor::GoblinUltraRecursive_<GoblinUltraCircuitBuilder>>;

} // namespace proof_system::plonk::stdlib::recursion::honk
