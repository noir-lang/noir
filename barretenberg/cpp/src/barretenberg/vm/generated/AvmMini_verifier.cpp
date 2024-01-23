

#include "./AvmMini_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"

using namespace bb;
using namespace bb::honk::sumcheck;

namespace bb::honk {
AvmMiniVerifier::AvmMiniVerifier(std::shared_ptr<Flavor::VerificationKey> verifier_key)
    : key(verifier_key)
{}

AvmMiniVerifier::AvmMiniVerifier(AvmMiniVerifier&& other) noexcept
    : key(std::move(other.key))
    , pcs_verification_key(std::move(other.pcs_verification_key))
{}

AvmMiniVerifier& AvmMiniVerifier::operator=(AvmMiniVerifier&& other) noexcept
{
    key = other.key;
    pcs_verification_key = (std::move(other.pcs_verification_key));
    commitments.clear();
    return *this;
}

/**
 * @brief This function verifies an AvmMini Honk proof for given program settings.
 *
 */
bool AvmMiniVerifier::verify_proof(const plonk::proof& proof)
{
    using Flavor = honk::flavor::AvmMiniFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    // using Curve = Flavor::Curve;
    // using ZeroMorph = pcs::zeromorph::ZeroMorphVerifier_<Curve>;
    using VerifierCommitments = Flavor::VerifierCommitments;
    using CommitmentLabels = Flavor::CommitmentLabels;

    RelationParameters<FF> relation_parameters;

    transcript = std::make_shared<Transcript>(proof.proof_data);

    VerifierCommitments commitments{ key };
    CommitmentLabels commitment_labels;

    const auto circuit_size = transcript->template receive_from_prover<uint32_t>("circuit_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }

    // Get commitments to VM wires
    commitments.memTrace_m_clk = transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_clk);
    commitments.memTrace_m_sub_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_sub_clk);
    commitments.memTrace_m_addr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_addr);
    commitments.memTrace_m_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_tag);
    commitments.memTrace_m_val = transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_val);
    commitments.memTrace_m_lastAccess =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_lastAccess);
    commitments.memTrace_m_last =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_last);
    commitments.memTrace_m_rw = transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_rw);
    commitments.memTrace_m_in_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_in_tag);
    commitments.memTrace_m_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_tag_err);
    commitments.memTrace_m_one_min_inv =
        transcript->template receive_from_prover<Commitment>(commitment_labels.memTrace_m_one_min_inv);
    commitments.aluChip_alu_clk =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_clk);
    commitments.aluChip_alu_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_ia);
    commitments.aluChip_alu_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_ib);
    commitments.aluChip_alu_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_ic);
    commitments.aluChip_alu_op_add =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_op_add);
    commitments.aluChip_alu_op_sub =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_op_sub);
    commitments.aluChip_alu_op_mul =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_op_mul);
    commitments.aluChip_alu_op_div =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_op_div);
    commitments.aluChip_alu_ff_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_ff_tag);
    commitments.aluChip_alu_u8_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u8_tag);
    commitments.aluChip_alu_u16_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_tag);
    commitments.aluChip_alu_u32_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u32_tag);
    commitments.aluChip_alu_u64_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u64_tag);
    commitments.aluChip_alu_u128_tag =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u128_tag);
    commitments.aluChip_alu_u8_r0 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u8_r0);
    commitments.aluChip_alu_u8_r1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u8_r1);
    commitments.aluChip_alu_u16_r0 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r0);
    commitments.aluChip_alu_u16_r1 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r1);
    commitments.aluChip_alu_u16_r2 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r2);
    commitments.aluChip_alu_u16_r3 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r3);
    commitments.aluChip_alu_u16_r4 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r4);
    commitments.aluChip_alu_u16_r5 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r5);
    commitments.aluChip_alu_u16_r6 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r6);
    commitments.aluChip_alu_u16_r7 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u16_r7);
    commitments.aluChip_alu_u64_r0 =
        transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_u64_r0);
    commitments.aluChip_alu_cf = transcript->template receive_from_prover<Commitment>(commitment_labels.aluChip_alu_cf);
    commitments.avmMini_pc = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_pc);
    commitments.avmMini_internal_return_ptr =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_internal_return_ptr);
    commitments.avmMini_sel_internal_call =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_internal_call);
    commitments.avmMini_sel_internal_return =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_internal_return);
    commitments.avmMini_sel_jump =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_jump);
    commitments.avmMini_sel_halt =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_halt);
    commitments.avmMini_sel_op_add =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_op_add);
    commitments.avmMini_sel_op_sub =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_op_sub);
    commitments.avmMini_sel_op_mul =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_op_mul);
    commitments.avmMini_sel_op_div =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_sel_op_div);
    commitments.avmMini_in_tag = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_in_tag);
    commitments.avmMini_op_err = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_op_err);
    commitments.avmMini_tag_err =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_tag_err);
    commitments.avmMini_inv = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_inv);
    commitments.avmMini_ia = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_ia);
    commitments.avmMini_ib = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_ib);
    commitments.avmMini_ic = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_ic);
    commitments.avmMini_mem_op_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_mem_op_a);
    commitments.avmMini_mem_op_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_mem_op_b);
    commitments.avmMini_mem_op_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_mem_op_c);
    commitments.avmMini_rwa = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_rwa);
    commitments.avmMini_rwb = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_rwb);
    commitments.avmMini_rwc = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_rwc);
    commitments.avmMini_mem_idx_a =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_mem_idx_a);
    commitments.avmMini_mem_idx_b =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_mem_idx_b);
    commitments.avmMini_mem_idx_c =
        transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_mem_idx_c);
    commitments.avmMini_last = transcript->template receive_from_prover<Commitment>(commitment_labels.avmMini_last);

    // Execute Sumcheck Verifier
    const size_t log_circuit_size = numeric::get_msb(circuit_size);
    auto sumcheck = SumcheckVerifier<Flavor>(log_circuit_size, transcript);

    FF alpha = transcript->get_challenge("Sumcheck:alpha");

    auto gate_challenges = std::vector<FF>(log_circuit_size);
    for (size_t idx = 0; idx < log_circuit_size; idx++) {
        gate_challenges[idx] = transcript->get_challenge("Sumcheck:gate_challenge_" + std::to_string(idx));
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

} // namespace bb::honk
