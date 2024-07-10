#include "barretenberg/vm/generated/avm_verifier.hpp"

#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

AvmVerifier::AvmVerifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

AvmVerifier::AvmVerifier(AvmVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

AvmVerifier& AvmVerifier::operator=(AvmVerifier&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

using FF = AvmFlavor::FF;

// Evaluate the given public input column over the multivariate challenge points
[[maybe_unused]] inline FF evaluate_public_input_column(const std::vector<FF>& points,
                                                        const size_t circuit_size,
                                                        std::vector<FF> challenges)
{

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6361): we pad the points to the circuit size in order
    // to get the correct evaluation. This is not efficient, and will not be valid in production.
    std::vector<FF> new_points(circuit_size, 0);
    std::copy(points.begin(), points.end(), new_points.data());

    Polynomial<FF> polynomial(new_points);
    return polynomial.evaluate_mle(challenges);
}

/**
 * @brief This function verifies an Avm Honk proof for given program settings.
 *
 */
bool AvmVerifier::verify_proof(const HonkProof& proof, const std::vector<std::vector<FF>>& public_inputs)
{
    using Flavor = AvmFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using PCS = Flavor::PCS;
    // using Curve = Flavor::Curve;
    // using ZeroMorph = ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = Flavor::VerifierCommitments;
    using CommitmentLabels = Flavor::CommitmentLabels;

    RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof);

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    for (auto [comm, label] : zip_view(commitments.get_wires(), commitment_labels.get_wires())) {
        comm = transcript->template receive_from_prover<Commitment>(label);
    }

    auto [beta, gamm] = transcript->template get_challenges<FF>("beta", "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamm;

    // Get commitments to inverses
    commitments.perm_slice_mem = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_slice_mem);
    commitments.perm_main_alu = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_alu);
    commitments.perm_main_bin = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_bin);
    commitments.perm_main_conv = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_conv);
    commitments.perm_main_pos2_perm =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_pos2_perm);
    commitments.perm_main_pedersen =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_pedersen);
    commitments.perm_main_slice =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_slice);
    commitments.perm_main_mem_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_a);
    commitments.perm_main_mem_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_b);
    commitments.perm_main_mem_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_c);
    commitments.perm_main_mem_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_d);
    commitments.perm_main_mem_ind_addr_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_a);
    commitments.perm_main_mem_ind_addr_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_b);
    commitments.perm_main_mem_ind_addr_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_c);
    commitments.perm_main_mem_ind_addr_d =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_addr_d);
    commitments.lookup_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths);
    commitments.lookup_byte_operations =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations);
    commitments.lookup_cd_value =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_cd_value);
    commitments.lookup_ret_value =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_ret_value);
    commitments.lookup_opcode_gas =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_opcode_gas);
    commitments.range_check_l2_gas_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_l2_gas_hi);
    commitments.range_check_l2_gas_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_l2_gas_lo);
    commitments.range_check_da_gas_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_da_gas_hi);
    commitments.range_check_da_gas_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.range_check_da_gas_lo);
    commitments.kernel_output_lookup =
        transcript->template receive_from_prover<Commitment>(commitment_labels.kernel_output_lookup);
    commitments.lookup_into_kernel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_into_kernel);
    commitments.incl_main_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_main_tag_err);
    commitments.incl_mem_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_mem_tag_err);
    commitments.lookup_mem_rng_chk_lo =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_lo);
    commitments.lookup_mem_rng_chk_mid =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_mid);
    commitments.lookup_mem_rng_chk_hi =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_mem_rng_chk_hi);
    commitments.lookup_pow_2_0 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_pow_2_0);
    commitments.lookup_pow_2_1 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_pow_2_1);
    commitments.lookup_u8_0 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u8_0);
    commitments.lookup_u8_1 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u8_1);
    commitments.lookup_u16_0 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_0);
    commitments.lookup_u16_1 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_1);
    commitments.lookup_u16_2 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_2);
    commitments.lookup_u16_3 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_3);
    commitments.lookup_u16_4 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_4);
    commitments.lookup_u16_5 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_5);
    commitments.lookup_u16_6 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_6);
    commitments.lookup_u16_7 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_7);
    commitments.lookup_u16_8 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_8);
    commitments.lookup_u16_9 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_9);
    commitments.lookup_u16_10 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_10);
    commitments.lookup_u16_11 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_11);
    commitments.lookup_u16_12 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_12);
    commitments.lookup_u16_13 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_13);
    commitments.lookup_u16_14 = transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_u16_14);
    commitments.lookup_div_u16_0 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_0);
    commitments.lookup_div_u16_1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_1);
    commitments.lookup_div_u16_2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_2);
    commitments.lookup_div_u16_3 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_3);
    commitments.lookup_div_u16_4 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_4);
    commitments.lookup_div_u16_5 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_5);
    commitments.lookup_div_u16_6 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_6);
    commitments.lookup_div_u16_7 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_div_u16_7);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);

    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");

    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }

    auto [multivariate_challenge, claimed_evaluations, sumcheck_verified] =
        sumcheck.verify(relation_parameters, alpha, gate_challenges);

    // If Sumcheck did not verify, return false
    if (sumcheck_verified.has_value() && !sumcheck_verified.value()) {
        return false;
    }

    // Public columns evaluation checks
    std::vector<FF> mle_challenge(multivariate_challenge.begin(),
                                  multivariate_challenge.begin() + static_cast<int>(log_circuit_size));

    FF kernel_kernel_inputs_evaluation = evaluate_public_input_column(public_inputs[0], circuit_size, mle_challenge);
    if (kernel_kernel_inputs_evaluation != claimed_evaluations.kernel_kernel_inputs) {
        return false;
    }
    FF kernel_kernel_value_out_evaluation = evaluate_public_input_column(public_inputs[1], circuit_size, mle_challenge);
    if (kernel_kernel_value_out_evaluation != claimed_evaluations.kernel_kernel_value_out) {
        return false;
    }
    FF kernel_kernel_side_effect_out_evaluation =
        evaluate_public_input_column(public_inputs[2], circuit_size, mle_challenge);
    if (kernel_kernel_side_effect_out_evaluation != claimed_evaluations.kernel_kernel_side_effect_out) {
        return false;
    }
    FF kernel_kernel_metadata_out_evaluation =
        evaluate_public_input_column(public_inputs[3], circuit_size, mle_challenge);
    if (kernel_kernel_metadata_out_evaluation != claimed_evaluations.kernel_kernel_metadata_out) {
        return false;
    }
    FF main_calldata_evaluation = evaluate_public_input_column(public_inputs[4], circuit_size, mle_challenge);
    if (main_calldata_evaluation != claimed_evaluations.main_calldata) {
        return false;
    }
    FF main_returndata_evaluation = evaluate_public_input_column(public_inputs[5], circuit_size, mle_challenge);
    if (main_returndata_evaluation != claimed_evaluations.main_returndata) {
        return false;
    }

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    // NOTE: temporarily disabled - facing integration issues
    // auto opening_claim = ZeroMorph::verify(commitments.get_unshifted(),
    //                                         commitments.get_to_be_shifted(),
    //                                         claimed_evaluations.get_unshifted(),
    //                                         claimed_evaluations.get_shifted(),
    //                                         multivariate_challenge,
    //                                         pcs_verification_key->get_g1_identity(),
    //                                         transcript);

    // auto pairing_points = PCS::reduce_verify(opening_claim, transcript);
    // auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);
    // return sumcheck_verified.value() && verified;
    return sumcheck_verified.value();
}

} // namespace bb