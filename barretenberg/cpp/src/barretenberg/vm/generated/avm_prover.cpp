#include "barretenberg/vm/generated/avm_prover.hpp"

#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

#include "barretenberg/vm/avm_trace/stats.hpp"

namespace bb {

using Flavor = AvmFlavor;
using FF = Flavor::FF;

namespace {

// Loops through LookupRelations and calculates the logderivatives.
// Metaprogramming is used to loop through the relations, because they are types.
template <size_t relation_idx = 0, typename PP>
void compute_logderivative_rel(const RelationParameters<FF>& relation_parameters,
                               PP& prover_polynomials,
                               size_t circuit_size)
{
    using Relation = std::tuple_element_t<relation_idx, Flavor::LookupRelations>;
    AVM_TRACK_TIME(
        Relation::NAME + std::string("_ms"),
        (compute_logderivative_inverse<Flavor, Relation>(prover_polynomials, relation_parameters, circuit_size)));

    if constexpr (relation_idx + 1 < std::tuple_size_v<Flavor::LookupRelations>) {
        compute_logderivative_rel<relation_idx + 1, PP>(relation_parameters, prover_polynomials, circuit_size);
    }
}

} // namespace

/**
 * Create AvmProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 */
AvmProver::AvmProver(std::shared_ptr<Flavor::ProvingKey> input_key, std::shared_ptr<PCSCommitmentKey> commitment_key)
    : key(input_key)
    , commitment_key(commitment_key)
{
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), key->get_all())) {
        ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) == bb::flavor_get_label(*key, key_poly));
        prover_poly = key_poly.share();
    }
    for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_shifted(), key->get_to_be_shifted())) {
        ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
               bb::flavor_get_label(*key, key_poly) + "_shift");
        prover_poly = key_poly.shifted();
    }
}

/**
 * @brief Add circuit size, public input size, and public inputs to transcript
 *
 */
void AvmProver::execute_preamble_round()
{
    const auto circuit_size = static_cast<uint32_t>(key->circuit_size);

    transcript->send_to_verifier("circuit_size", circuit_size);
}

/**
 * @brief Compute commitments to all of the witness wires (apart from the logderivative inverse wires)
 *
 */
void AvmProver::execute_wire_commitments_round()
{
    // Commit to all polynomials (apart from logderivative inverse polynomials, which are committed to in the later
    // logderivative phase)
    auto wire_polys = prover_polynomials.get_wires();
    auto labels = commitment_labels.get_wires();
    for (size_t idx = 0; idx < wire_polys.size(); ++idx) {
        transcript->send_to_verifier(labels[idx], commitment_key->commit(wire_polys[idx]));
    }
}

void AvmProver::execute_log_derivative_inverse_round()
{
    auto [beta, gamm] = transcript->template get_challenges<FF>("beta", "gamma");
    relation_parameters.beta = beta;
    relation_parameters.gamma = gamm;

    auto prover_polynomials = ProverPolynomials(*key);
    compute_logderivative_rel(relation_parameters, prover_polynomials, key->circuit_size);

    // Commit to all logderivative inverse polynomials
    for (auto [commitment, key_poly] : zip_view(witness_commitments.get_derived(), key->get_derived())) {
        commitment = commitment_key->commit(key_poly);
    }

    // Send all commitments to the verifier
    for (auto [label, commitment] : zip_view(commitment_labels.get_derived(), witness_commitments.get_derived())) {
        transcript->send_to_verifier(label, commitment);
    }
}

/**
 * @brief Run Sumcheck resulting in u = (u_1,...,u_d) challenges and all evaluations at u being calculated.
 *
 */
void AvmProver::execute_relation_check_rounds()
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
 */
void AvmProver::execute_pcs_rounds()
{
    auto prover_opening_claim = ZeroMorph::prove(key->circuit_size,
                                                 prover_polynomials.get_unshifted(),
                                                 prover_polynomials.get_to_be_shifted(),
                                                 sumcheck_output.claimed_evaluations.get_unshifted(),
                                                 sumcheck_output.claimed_evaluations.get_shifted(),
                                                 sumcheck_output.challenge,
                                                 commitment_key,
                                                 transcript);
    PCS::compute_opening_proof(commitment_key, prover_opening_claim, transcript);
}

HonkProof AvmProver::export_proof()
{
    proof = transcript->proof_data;
    return proof;
}

HonkProof AvmProver::construct_proof()
{
    // Add circuit size public input size and public inputs to transcript.
    execute_preamble_round();

    // Compute wire commitments
    execute_wire_commitments_round();

    // Compute sorted list accumulator and commitment
    execute_log_derivative_inverse_round();

    // Fiat-Shamir: alpha
    // Run sumcheck subprotocol.
    execute_relation_check_rounds();

    // Fiat-Shamir: rho, y, x, z
    // Execute Zeromorph multilinear PCS
    execute_pcs_rounds();

    return export_proof();
}

} // namespace bb