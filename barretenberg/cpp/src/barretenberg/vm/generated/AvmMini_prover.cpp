

#include "AvmMini_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/honk/proof_system/power_polynomial.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"
#include "barretenberg/relations/lookup_relation.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace proof_system::honk {

using Flavor = honk::flavor::AvmMiniFlavor;

/**
 * Create AvmMiniProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
AvmMiniProver::AvmMiniProver(std::shared_ptr<Flavor::ProvingKey> input_key,
                             std::shared_ptr<PCSCommitmentKey> commitment_key)
    : key(input_key)
    , commitment_key(commitment_key)
{
    // TODO: take every polynomial and assign it to the key!!

    prover_polynomials.avmMini_clk = key->avmMini_clk;
    prover_polynomials.avmMini_first = key->avmMini_first;
    prover_polynomials.memTrace_m_clk = key->memTrace_m_clk;
    prover_polynomials.memTrace_m_sub_clk = key->memTrace_m_sub_clk;
    prover_polynomials.memTrace_m_addr = key->memTrace_m_addr;
    prover_polynomials.memTrace_m_val = key->memTrace_m_val;
    prover_polynomials.memTrace_m_lastAccess = key->memTrace_m_lastAccess;
    prover_polynomials.memTrace_m_rw = key->memTrace_m_rw;
    prover_polynomials.avmMini_subop = key->avmMini_subop;
    prover_polynomials.avmMini_ia = key->avmMini_ia;
    prover_polynomials.avmMini_ib = key->avmMini_ib;
    prover_polynomials.avmMini_ic = key->avmMini_ic;
    prover_polynomials.avmMini_mem_op_a = key->avmMini_mem_op_a;
    prover_polynomials.avmMini_mem_op_b = key->avmMini_mem_op_b;
    prover_polynomials.avmMini_mem_op_c = key->avmMini_mem_op_c;
    prover_polynomials.avmMini_rwa = key->avmMini_rwa;
    prover_polynomials.avmMini_rwb = key->avmMini_rwb;
    prover_polynomials.avmMini_rwc = key->avmMini_rwc;
    prover_polynomials.avmMini_mem_idx_a = key->avmMini_mem_idx_a;
    prover_polynomials.avmMini_mem_idx_b = key->avmMini_mem_idx_b;
    prover_polynomials.avmMini_mem_idx_c = key->avmMini_mem_idx_c;
    prover_polynomials.avmMini_last = key->avmMini_last;

    prover_polynomials.memTrace_m_addr = key->memTrace_m_addr;
    prover_polynomials.memTrace_m_addr_shift = key->memTrace_m_addr.shifted();

    prover_polynomials.memTrace_m_rw = key->memTrace_m_rw;
    prover_polynomials.memTrace_m_rw_shift = key->memTrace_m_rw.shifted();

    prover_polynomials.memTrace_m_val = key->memTrace_m_val;
    prover_polynomials.memTrace_m_val_shift = key->memTrace_m_val.shifted();

    // prover_polynomials.lookup_inverses = key->lookup_inverses;
    // key->z_perm = Polynomial(key->circuit_size);
    // prover_polynomials.z_perm = key->z_perm;
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
void AvmMiniProver::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);

    transcript->send_to_verifier("circuit_size", circuit_size);
}

/**
 * @brief Compute commitments to the first three wires
 *
 */
void AvmMiniProver::execute_wire_commitments_round()
{
    auto wire_polys = key->get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < wire_polys.size(); ++idx) {
        transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
void AvmMiniProver::execute_relation_check_rounds()
{
    using Sumcheck = sumcheck::SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);
    auto alpha = transcript->get_challenge("alpha");

    sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters, alpha);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
void AvmMiniProver::execute_zeromorph_rounds()
{
    ZeroMorph::prove(prover_polynomials.get_unshifted(),
                     prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript);
}

plonk::proof& AvmMiniProver::export_proof()
{
    proof.proof_data = transcript->proof_data;
    return proof;
}

plonk::proof& AvmMiniProver::construct_proof()
{
    // Add circuit size public input size and public inputs to transcript->
    execute_preamble_round();

    // Compute wire commitments
    execute_wire_commitments_round();

    // TODO: not implemented for codegen just yet
    // Compute sorted list accumulator and commitment
    // execute_log_derivative_commitments_round();

    // Fiat-Shamir: bbeta & gamma
    // Compute grand product(s) and commitments.
    // execute_grand_product_computation_round();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_zeromorph_rounds();

    return export_proof();
}

} // namespace proof_system::honk
