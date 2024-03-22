#include "goblin_translator_prover.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/proof_system/library/grand_product_library.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {

/**
 * Create GoblinTranslatorProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
GoblinTranslatorProver::GoblinTranslatorProver(const std::shared_ptr<typename Flavor::ProvingKey>& input_key,
                                               const std::shared_ptr<CommitmentKey>& commitment_key,
                                               const std::shared_ptr<Transcript>& transcript)
    : transcript(transcript)
    , key(input_key)
    , commitment_key(commitment_key)
{
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), key->get_all())) {
        ASSERT(flavor_get_label(prover_polynomials, prover_poly) == flavor_get_label(*key, key_poly));
        prover_poly = key_poly.share();
    }
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_shifted(), key->get_to_be_shifted())) {
        ASSERT(flavor_get_label(prover_polynomials, prover_poly) == flavor_get_label(*key, key_poly) + "_shift");
        prover_poly = key_poly.shifted();
    }
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/810): resolve weirdness around concatenated range
    // constraints
    prover_polynomials.concatenated_range_constraints_0 = key->concatenated_range_constraints_0;
    prover_polynomials.concatenated_range_constraints_1 = key->concatenated_range_constraints_1;
    prover_polynomials.concatenated_range_constraints_2 = key->concatenated_range_constraints_2;
    prover_polynomials.concatenated_range_constraints_3 = key->concatenated_range_constraints_3;
}

GoblinTranslatorProver::GoblinTranslatorProver(CircuitBuilder& circuit_builder,
                                               const std::shared_ptr<Transcript>& transcript)
    : dyadic_circuit_size(Flavor::compute_dyadic_circuit_size(circuit_builder))
    , mini_circuit_dyadic_size(Flavor::compute_mini_circuit_dyadic_size(circuit_builder))

{
    BB_OP_COUNT_TIME();

    // Compute total number of gates, dyadic circuit size, etc.
    key = std::make_shared<ProvingKey>(circuit_builder);
    dyadic_circuit_size = key->circuit_size;
    compute_witness(circuit_builder);
    compute_commitment_key(key->circuit_size);

    *this = GoblinTranslatorProver(key, commitment_key, transcript);
}

/**
 * @brief Construct the witness polynomials from the witness vectors in the circuit constructor.
 *
 * @details In goblin translator wires come as is, since they have to reflect the structure of polynomials in the first
 * 4 wires, which we've commited to
 *
 * @tparam Flavor provides the circuit constructor type and the number of wires.
 * @param circuit_builder
 * @param dyadic_circuit_size Power of 2 circuit size
 * @todo TODO(https://github.com/AztecProtocol/barretenberg/issues/783) Optimize memory operations.
 * @return std::vector<Polynomial>
 * */
std::vector<GoblinTranslatorProver::Polynomial> construct_wire_polynomials(
    const GoblinTranslatorProver::CircuitBuilder& circuit_builder, const size_t dyadic_circuit_size)
{
    const size_t num_gates = circuit_builder.num_gates;

    std::vector<GoblinTranslatorProver::Polynomial> wire_polynomials;
    // Populate the wire polynomials with values from conventional wires
    for (size_t wire_idx = 0; wire_idx < GoblinTranslatorFlavor::NUM_WIRES; ++wire_idx) {
        // Expect all values to be set to 0 initially
        GoblinTranslatorProver::Polynomial w_lagrange(dyadic_circuit_size);

        // Insert conventional gate wire values into the wire polynomial
        for (size_t i = 0; i < num_gates; ++i) {
            auto& wire = circuit_builder.wires[wire_idx];
            w_lagrange[i] = circuit_builder.get_variable(wire[i]);
        }

        wire_polynomials.push_back(std::move(w_lagrange));
    }
    return wire_polynomials;
}

/**
 * @brief Compute witness polynomials
 *
 */
void GoblinTranslatorProver::compute_witness(CircuitBuilder& circuit_builder)
{
    if (computed_witness) {
        return;
    }

    // Construct the conventional wire polynomials
    auto wire_polynomials = construct_wire_polynomials(circuit_builder, dyadic_circuit_size);

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/907)
    // In order:
    //   wire_polynomials
    //    = WireEntities::get_wires - concatenated
    //    = WireNonShiftedEntities + WireToBeShiftedEntities - concatenated
    key->op = wire_polynomials[0];
    key->x_lo_y_hi = wire_polynomials[1];
    key->x_hi_z_1 = wire_polynomials[2];
    key->y_lo_z_2 = wire_polynomials[3];
    key->p_x_low_limbs = wire_polynomials[4];
    key->p_x_low_limbs_range_constraint_0 = wire_polynomials[5];
    key->p_x_low_limbs_range_constraint_1 = wire_polynomials[6];
    key->p_x_low_limbs_range_constraint_2 = wire_polynomials[7];
    key->p_x_low_limbs_range_constraint_3 = wire_polynomials[8];
    key->p_x_low_limbs_range_constraint_4 = wire_polynomials[9];
    key->p_x_low_limbs_range_constraint_tail = wire_polynomials[10];
    key->p_x_high_limbs = wire_polynomials[11];
    key->p_x_high_limbs_range_constraint_0 = wire_polynomials[12];
    key->p_x_high_limbs_range_constraint_1 = wire_polynomials[13];
    key->p_x_high_limbs_range_constraint_2 = wire_polynomials[14];
    key->p_x_high_limbs_range_constraint_3 = wire_polynomials[15];
    key->p_x_high_limbs_range_constraint_4 = wire_polynomials[16];
    key->p_x_high_limbs_range_constraint_tail = wire_polynomials[17];
    key->p_y_low_limbs = wire_polynomials[18];
    key->p_y_low_limbs_range_constraint_0 = wire_polynomials[19];
    key->p_y_low_limbs_range_constraint_1 = wire_polynomials[20];
    key->p_y_low_limbs_range_constraint_2 = wire_polynomials[21];
    key->p_y_low_limbs_range_constraint_3 = wire_polynomials[22];
    key->p_y_low_limbs_range_constraint_4 = wire_polynomials[23];
    key->p_y_low_limbs_range_constraint_tail = wire_polynomials[24];
    key->p_y_high_limbs = wire_polynomials[25];
    key->p_y_high_limbs_range_constraint_0 = wire_polynomials[26];
    key->p_y_high_limbs_range_constraint_1 = wire_polynomials[27];
    key->p_y_high_limbs_range_constraint_2 = wire_polynomials[28];
    key->p_y_high_limbs_range_constraint_3 = wire_polynomials[29];
    key->p_y_high_limbs_range_constraint_4 = wire_polynomials[30];
    key->p_y_high_limbs_range_constraint_tail = wire_polynomials[31];
    key->z_low_limbs = wire_polynomials[32];
    key->z_low_limbs_range_constraint_0 = wire_polynomials[33];
    key->z_low_limbs_range_constraint_1 = wire_polynomials[34];
    key->z_low_limbs_range_constraint_2 = wire_polynomials[35];
    key->z_low_limbs_range_constraint_3 = wire_polynomials[36];
    key->z_low_limbs_range_constraint_4 = wire_polynomials[37];
    key->z_low_limbs_range_constraint_tail = wire_polynomials[38];
    key->z_high_limbs = wire_polynomials[39];
    key->z_high_limbs_range_constraint_0 = wire_polynomials[40];
    key->z_high_limbs_range_constraint_1 = wire_polynomials[41];
    key->z_high_limbs_range_constraint_2 = wire_polynomials[42];
    key->z_high_limbs_range_constraint_3 = wire_polynomials[43];
    key->z_high_limbs_range_constraint_4 = wire_polynomials[44];
    key->z_high_limbs_range_constraint_tail = wire_polynomials[45];
    key->accumulators_binary_limbs_0 = wire_polynomials[46];
    key->accumulators_binary_limbs_1 = wire_polynomials[47];
    key->accumulators_binary_limbs_2 = wire_polynomials[48];
    key->accumulators_binary_limbs_3 = wire_polynomials[49];
    key->accumulator_low_limbs_range_constraint_0 = wire_polynomials[50];
    key->accumulator_low_limbs_range_constraint_1 = wire_polynomials[51];
    key->accumulator_low_limbs_range_constraint_2 = wire_polynomials[52];
    key->accumulator_low_limbs_range_constraint_3 = wire_polynomials[53];
    key->accumulator_low_limbs_range_constraint_4 = wire_polynomials[54];
    key->accumulator_low_limbs_range_constraint_tail = wire_polynomials[55];
    key->accumulator_high_limbs_range_constraint_0 = wire_polynomials[56];
    key->accumulator_high_limbs_range_constraint_1 = wire_polynomials[57];
    key->accumulator_high_limbs_range_constraint_2 = wire_polynomials[58];
    key->accumulator_high_limbs_range_constraint_3 = wire_polynomials[59];
    key->accumulator_high_limbs_range_constraint_4 = wire_polynomials[60];
    key->accumulator_high_limbs_range_constraint_tail = wire_polynomials[61];
    key->quotient_low_binary_limbs = wire_polynomials[62];
    key->quotient_high_binary_limbs = wire_polynomials[63];
    key->quotient_low_limbs_range_constraint_0 = wire_polynomials[64];
    key->quotient_low_limbs_range_constraint_1 = wire_polynomials[65];
    key->quotient_low_limbs_range_constraint_2 = wire_polynomials[66];
    key->quotient_low_limbs_range_constraint_3 = wire_polynomials[67];
    key->quotient_low_limbs_range_constraint_4 = wire_polynomials[68];
    key->quotient_low_limbs_range_constraint_tail = wire_polynomials[69];
    key->quotient_high_limbs_range_constraint_0 = wire_polynomials[70];
    key->quotient_high_limbs_range_constraint_1 = wire_polynomials[71];
    key->quotient_high_limbs_range_constraint_2 = wire_polynomials[72];
    key->quotient_high_limbs_range_constraint_3 = wire_polynomials[73];
    key->quotient_high_limbs_range_constraint_4 = wire_polynomials[74];
    key->quotient_high_limbs_range_constraint_tail = wire_polynomials[75];
    key->relation_wide_limbs = wire_polynomials[76];
    key->relation_wide_limbs_range_constraint_0 = wire_polynomials[77];
    key->relation_wide_limbs_range_constraint_1 = wire_polynomials[78];
    key->relation_wide_limbs_range_constraint_2 = wire_polynomials[79];
    key->relation_wide_limbs_range_constraint_3 = wire_polynomials[80];

    // We construct concatenated versions of range constraint polynomials, where several polynomials are concatenated
    // into one. These polynomials are not commited to.
    bb::compute_concatenated_polynomials<Flavor>(key.get());

    // We also contruct ordered polynomials, which have the same values as concatenated ones + enough values to bridge
    // the range from 0 to maximum range defined by the range constraint.
    bb::compute_goblin_translator_range_constraint_ordered_polynomials<Flavor>(key.get(), mini_circuit_dyadic_size);

    computed_witness = true;
}

std::shared_ptr<GoblinTranslatorProver::CommitmentKey> GoblinTranslatorProver::compute_commitment_key(
    size_t circuit_size)
{
    if (commitment_key) {
        return commitment_key;
    }

    commitment_key = std::make_shared<CommitmentKey>(circuit_size);
    return commitment_key;
};

/**
 * @brief Add circuit size and values used in the relations to the transcript
 *
 */
void GoblinTranslatorProver::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);
    const auto SHIFT = uint256_t(1) << Flavor::NUM_LIMB_BITS;
    const auto SHIFTx2 = uint256_t(1) << (Flavor::NUM_LIMB_BITS * 2);
    const auto SHIFTx3 = uint256_t(1) << (Flavor::NUM_LIMB_BITS * 3);
    const auto accumulated_result =
        BF(uint256_t(key->accumulators_binary_limbs_0[1]) + uint256_t(key->accumulators_binary_limbs_1[1]) * SHIFT +
           uint256_t(key->accumulators_binary_limbs_2[1]) * SHIFTx2 +
           uint256_t(key->accumulators_binary_limbs_3[1]) * SHIFTx3);
    transcript->send_to_verifier("circuit_size", circuit_size);
    transcript->send_to_verifier("evaluation_input_x", key->evaluation_input_x);
    transcript->send_to_verifier("accumulated_result", accumulated_result);
}

/**
 * @brief Compute commitments to the first three wires
 *
 */
void GoblinTranslatorProver::execute_wire_and_sorted_constraints_commitments_round()
{
    // Commit to all wire polynomials
    auto wire_polys = key->get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < wire_polys.size(); ++idx) {
        transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }
}

/**
 * @brief Compute permutation product polynomial and commitments
 *
 */
void GoblinTranslatorProver::execute_grand_product_computation_round()
{
    // Compute and store parameters required by relations in Sumcheck
    FF gamma = transcript->template get_challenge<FF>("gamma");
    const size_t NUM_LIMB_BITS = Flavor::NUM_LIMB_BITS;
    relation_parameters.beta = 0;
    relation_parameters.gamma = gamma;
    relation_parameters.public_input_delta = 0;
    relation_parameters.lookup_grand_product_delta = 0;
    auto uint_evaluation_input = uint256_t(key->evaluation_input_x);
    relation_parameters.evaluation_input_x = { uint_evaluation_input.slice(0, NUM_LIMB_BITS),
                                               uint_evaluation_input.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
                                               uint_evaluation_input.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
                                               uint_evaluation_input.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
                                               uint_evaluation_input };

    relation_parameters.accumulated_result = { key->accumulators_binary_limbs_0[1],
                                               key->accumulators_binary_limbs_1[1],
                                               key->accumulators_binary_limbs_2[1],
                                               key->accumulators_binary_limbs_3[1] };

    std::vector<uint256_t> uint_batching_challenge_powers;
    auto batching_challenge_v = key->batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(batching_challenge_v);
    auto running_power = batching_challenge_v * batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(running_power);
    running_power *= batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(running_power);
    running_power *= batching_challenge_v;
    uint_batching_challenge_powers.emplace_back(running_power);

    for (size_t i = 0; i < 4; i++) {
        relation_parameters.batching_challenge_v[i] = {
            uint_batching_challenge_powers[i].slice(0, NUM_LIMB_BITS),
            uint_batching_challenge_powers[i].slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2),
            uint_batching_challenge_powers[i].slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3),
            uint_batching_challenge_powers[i].slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4),
            uint_batching_challenge_powers[i]
        };
    }
    // Compute constraint permutation grand product
    compute_grand_products<Flavor>(*key, prover_polynomials, relation_parameters);

    transcript->send_to_verifier(commitment_labels.z_perm, commitment_key->commit(key->z_perm));
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
void GoblinTranslatorProver::execute_relation_check_rounds()
{
    using Sumcheck = SumcheckProver<Flavor>;

    auto sumcheck = Sumcheck(key->circuit_size, transcript);
    FF alpha = transcript->template get_challenge<FF>("Sumcheck:alpha");
    std::vector<FF> gate_challenges(numeric::get_msb(key->circuit_size));
    for (size_t idx = 0; idx < gate_challenges.size(); idx++) {
        gate_challenges[idx] = transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx));
    }
    sumcheck_output = sumcheck.prove(prover_polynomials, relation_parameters, alpha, gate_challenges);
}

/**
 * @brief Execute the ZeroMorph protocol to prove the multilinear evaluations produced by Sumcheck
 * @details See https://hackmd.io/dlf9xEwhTQyE3hiGbq4FsA?view for a complete description of the unrolled protocol.
 *
 * */
void GoblinTranslatorProver::execute_zeromorph_rounds()
{
    using ZeroMorph = ZeroMorphProver_<PCS>;
    ZeroMorph::prove(prover_polynomials.get_unshifted(),
                     prover_polynomials.get_to_be_shifted(),
                     sumcheck_output.claimed_evaluations.get_unshifted(),
                     sumcheck_output.claimed_evaluations.get_shifted(),
                     sumcheck_output.challenge,
                     commitment_key,
                     transcript,
                     prover_polynomials.get_concatenated_constraints(),
                     sumcheck_output.claimed_evaluations.get_concatenated_constraints(),
                     prover_polynomials.get_concatenation_groups());
}

HonkProof& GoblinTranslatorProver::export_proof()
{
    proof = transcript->export_proof();
    return proof;
}

HonkProof& GoblinTranslatorProver::construct_proof()
{
    BB_OP_COUNT_TIME_NAME("GoblinTranslatorProver::construct_proof");

    // Add circuit size public input size and public inputs to transcript.
    execute_preamble_round();

    // Compute first three wire commitments
    execute_wire_and_sorted_constraints_commitments_round();

    // Fiat-Shamir: gamma
    // Compute grand product(s) and commitments.
    execute_grand_product_computation_round();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_zeromorph_rounds();

    return export_proof();
}

} // namespace bb
