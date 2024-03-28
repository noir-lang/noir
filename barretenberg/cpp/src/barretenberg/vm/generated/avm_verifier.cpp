

#include "./avm_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
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

/**
 * @brief This function verifies an Avm Honk proof for given program settings.
 *
 */
bool AvmVerifier::verify_proof(const HonkProof& proof)
{
    using Flavor = AvmFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using PCS = Flavor::PCS;
    // using ZeroMorph = ZeroMorphVerifier_<PCS>;
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
    commitments.avm_alu_alu_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_alu_sel);
    commitments.avm_alu_cf = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_cf);
    commitments.avm_alu_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_clk);
    commitments.avm_alu_ff_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ff_tag);
    commitments.avm_alu_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ia);
    commitments.avm_alu_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ib);
    commitments.avm_alu_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_ic);
    commitments.avm_alu_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_in_tag);
    commitments.avm_alu_op_add = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_add);
    commitments.avm_alu_op_div = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_div);
    commitments.avm_alu_op_eq = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_eq);
    commitments.avm_alu_op_eq_diff_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_eq_diff_inv);
    commitments.avm_alu_op_mul = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_mul);
    commitments.avm_alu_op_not = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_not);
    commitments.avm_alu_op_sub = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_op_sub);
    commitments.avm_alu_u128_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u128_tag);
    commitments.avm_alu_u16_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r0);
    commitments.avm_alu_u16_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r1);
    commitments.avm_alu_u16_r10 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r10);
    commitments.avm_alu_u16_r11 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r11);
    commitments.avm_alu_u16_r12 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r12);
    commitments.avm_alu_u16_r13 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r13);
    commitments.avm_alu_u16_r14 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r14);
    commitments.avm_alu_u16_r2 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r2);
    commitments.avm_alu_u16_r3 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r3);
    commitments.avm_alu_u16_r4 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r4);
    commitments.avm_alu_u16_r5 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r5);
    commitments.avm_alu_u16_r6 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r6);
    commitments.avm_alu_u16_r7 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r7);
    commitments.avm_alu_u16_r8 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r8);
    commitments.avm_alu_u16_r9 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_r9);
    commitments.avm_alu_u16_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u16_tag);
    commitments.avm_alu_u32_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u32_tag);
    commitments.avm_alu_u64_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u64_r0);
    commitments.avm_alu_u64_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u64_tag);
    commitments.avm_alu_u8_r0 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u8_r0);
    commitments.avm_alu_u8_r1 = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u8_r1);
    commitments.avm_alu_u8_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_alu_u8_tag);
    commitments.avm_binary_acc_ia =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_acc_ia);
    commitments.avm_binary_acc_ib =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_acc_ib);
    commitments.avm_binary_acc_ic =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_acc_ic);
    commitments.avm_binary_bin_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_bin_sel);
    commitments.avm_binary_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_clk);
    commitments.avm_binary_ia_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_ia_bytes);
    commitments.avm_binary_ib_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_ib_bytes);
    commitments.avm_binary_ic_bytes =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_ic_bytes);
    commitments.avm_binary_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_in_tag);
    commitments.avm_binary_mem_tag_ctr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_mem_tag_ctr);
    commitments.avm_binary_mem_tag_ctr_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_mem_tag_ctr_inv);
    commitments.avm_binary_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_op_id);
    commitments.avm_binary_start =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_binary_start);
    commitments.avm_byte_lookup_bin_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_bin_sel);
    commitments.avm_byte_lookup_table_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_byte_lengths);
    commitments.avm_byte_lookup_table_in_tags =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_in_tags);
    commitments.avm_byte_lookup_table_input_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_input_a);
    commitments.avm_byte_lookup_table_input_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_input_b);
    commitments.avm_byte_lookup_table_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_op_id);
    commitments.avm_byte_lookup_table_output =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_byte_lookup_table_output);
    commitments.avm_main_alu_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_alu_sel);
    commitments.avm_main_bin_op_id =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_bin_op_id);
    commitments.avm_main_bin_sel =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_bin_sel);
    commitments.avm_main_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ia);
    commitments.avm_main_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ib);
    commitments.avm_main_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ic);
    commitments.avm_main_ind_a = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_a);
    commitments.avm_main_ind_b = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_b);
    commitments.avm_main_ind_c = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_c);
    commitments.avm_main_ind_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_a);
    commitments.avm_main_ind_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_b);
    commitments.avm_main_ind_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_ind_op_c);
    commitments.avm_main_internal_return_ptr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_internal_return_ptr);
    commitments.avm_main_inv = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_inv);
    commitments.avm_main_last = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_last);
    commitments.avm_main_mem_idx_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_a);
    commitments.avm_main_mem_idx_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_b);
    commitments.avm_main_mem_idx_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_idx_c);
    commitments.avm_main_mem_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_a);
    commitments.avm_main_mem_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_b);
    commitments.avm_main_mem_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_mem_op_c);
    commitments.avm_main_op_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_op_err);
    commitments.avm_main_pc = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_pc);
    commitments.avm_main_r_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_r_in_tag);
    commitments.avm_main_rwa = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwa);
    commitments.avm_main_rwb = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwb);
    commitments.avm_main_rwc = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_rwc);
    commitments.avm_main_sel_halt =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_halt);
    commitments.avm_main_sel_internal_call =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_internal_call);
    commitments.avm_main_sel_internal_return =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_internal_return);
    commitments.avm_main_sel_jump =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_jump);
    commitments.avm_main_sel_mov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_mov);
    commitments.avm_main_sel_op_add =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_add);
    commitments.avm_main_sel_op_and =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_and);
    commitments.avm_main_sel_op_div =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_div);
    commitments.avm_main_sel_op_eq =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_eq);
    commitments.avm_main_sel_op_mul =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_mul);
    commitments.avm_main_sel_op_not =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_not);
    commitments.avm_main_sel_op_or =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_or);
    commitments.avm_main_sel_op_sub =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_sub);
    commitments.avm_main_sel_op_xor =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_op_xor);
    commitments.avm_main_sel_rng_16 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_rng_16);
    commitments.avm_main_sel_rng_8 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_sel_rng_8);
    commitments.avm_main_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_tag_err);
    commitments.avm_main_w_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_main_w_in_tag);
    commitments.avm_mem_addr = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_addr);
    commitments.avm_mem_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_clk);
    commitments.avm_mem_ind_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_a);
    commitments.avm_mem_ind_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_b);
    commitments.avm_mem_ind_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_ind_op_c);
    commitments.avm_mem_last = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_last);
    commitments.avm_mem_lastAccess =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_lastAccess);
    commitments.avm_mem_one_min_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_one_min_inv);
    commitments.avm_mem_op_a = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_a);
    commitments.avm_mem_op_b = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_b);
    commitments.avm_mem_op_c = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_op_c);
    commitments.avm_mem_r_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_r_in_tag);
    commitments.avm_mem_rw = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_rw);
    commitments.avm_mem_sel_mov =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_sel_mov);
    commitments.avm_mem_sub_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_sub_clk);
    commitments.avm_mem_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_tag);
    commitments.avm_mem_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_tag_err);
    commitments.avm_mem_val = transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_val);
    commitments.avm_mem_w_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avm_mem_w_in_tag);
    commitments.perm_main_alu = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_alu);
    commitments.perm_main_bin = transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_bin);
    commitments.perm_main_mem_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_a);
    commitments.perm_main_mem_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_b);
    commitments.perm_main_mem_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_c);
    commitments.perm_main_mem_ind_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_a);
    commitments.perm_main_mem_ind_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_b);
    commitments.perm_main_mem_ind_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.perm_main_mem_ind_c);
    commitments.lookup_byte_lengths =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths);
    commitments.lookup_byte_operations =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations);
    commitments.incl_main_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_main_tag_err);
    commitments.incl_mem_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_mem_tag_err);
    commitments.lookup_byte_lengths_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_lengths_counts);
    commitments.lookup_byte_operations_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.lookup_byte_operations_counts);
    commitments.incl_main_tag_err_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_main_tag_err_counts);
    commitments.incl_mem_tag_err_counts =
        transcript->template receive_from_prover<Commitment>(commitment_labels.incl_mem_tag_err_counts);

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

    // Execute ZeroMorph rounds. See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the
    // unrolled protocol.
    // NOTE: temporarily disabled - facing integration issues
    // auto pairing_points = ZeroMorph::verify(commitments.get_unshifted(),
    //                                         commitments.get_to_be_shifted(),
    //                                         claimed_evaluations.get_unshifted(),
    //                                         claimed_evaluations.get_shifted(),
    //                                         multivariate_challenge,
    //                                         transcript);

    // auto verified = pcs_verification_key->pairing_check(pairing_points[0], pairing_points[1]);
    // return sumcheck_verified.value() && verified;
    return sumcheck_verified.value();
}

} // namespace bb
