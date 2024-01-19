#include "merge_prover.hpp"

namespace bb::honk {

/**
 * Create MergeProver_
 *
 */
template <typename Flavor>
MergeProver_<Flavor>::MergeProver_(const std::shared_ptr<CommitmentKey>& commitment_key,
                                   const std::shared_ptr<ECCOpQueue>& op_queue,
                                   const std::shared_ptr<Transcript>& transcript)
    : transcript(transcript)
    , op_queue(op_queue)
    , pcs_commitment_key(commitment_key)
{}

/**
 * @brief Prove proper construction of the aggregate Goblin ECC op queue polynomials T_i^(j), j = 1,2,3,4.
 * @details Let T_i^(j) be the jth column of the aggregate op queue after incorporating the contribution from the
 * present circuit. T_{i-1}^(j) corresponds to the aggregate op queue at the previous stage and $t_i^(j)$ represents
 * the contribution from the present circuit only. For each j, we have the relationship T_i = T_{i-1} + right_shift(t_i,
 * M_{i-1}), where the shift magnitude M_{i-1} is the length of T_{i-1}. This protocol demonstrates that the aggregate
 * op queue has been constructed correctly via a simple Schwartz-Zippel check. Evaluations are proven via batched KZG.
 *
 * TODO(#746): Prove connection between t_i^{shift}, committed to herein, and t_i, used in the main protocol. See issue
 * for details (https://github.com/AztecProtocol/barretenberg/issues/746).
 *
 * @tparam Flavor
 * @return plonk::proof&
 */
template <typename Flavor> plonk::proof& MergeProver_<Flavor>::construct_proof()
{
    size_t N = op_queue->get_current_size();

    // Extract T_i, T_{i-1}
    auto T_current = op_queue->get_aggregate_transcript();
    auto T_prev = op_queue->get_previous_aggregate_transcript();
    // TODO(#723): Cannot currently support an empty T_{i-1}. Need to be able to properly handle zero commitment.
    ASSERT(T_prev[0].size() > 0);

    // Construct t_i^{shift} as T_i - T_{i-1}
    std::array<Polynomial, Flavor::NUM_WIRES> t_shift;
    for (size_t i = 0; i < Flavor::NUM_WIRES; ++i) {
        t_shift[i] = Polynomial(T_current[i]);
        t_shift[i] -= T_prev[i];
    }

    // Compute/get commitments [t_i^{shift}], [T_{i-1}], and [T_i] and add to transcript
    std::array<Commitment, Flavor::NUM_WIRES> C_T_current;
    for (size_t idx = 0; idx < t_shift.size(); ++idx) {
        // Get previous transcript commitment [T_{i-1}] from op queue
        auto C_T_prev = op_queue->ultra_ops_commitments[idx];
        // Compute commitment [t_i^{shift}] directly
        auto C_t_shift = pcs_commitment_key->commit(t_shift[idx]);
        // Compute updated aggregate transcript commitment as [T_i] = [T_{i-1}] + [t_i^{shift}]
        C_T_current[idx] = C_T_prev + C_t_shift;

        std::string suffix = std::to_string(idx + 1);
        transcript->send_to_verifier("T_PREV_" + suffix, C_T_prev);
        transcript->send_to_verifier("t_SHIFT_" + suffix, C_t_shift);
        transcript->send_to_verifier("T_CURRENT_" + suffix, C_T_current[idx]);
    }

    // Store the commitments [T_{i}] (to be used later in subsequent iterations as [T_{i-1}]).
    op_queue->set_commitment_data(C_T_current);

    // Compute evaluations T_i(\kappa), T_{i-1}(\kappa), t_i^{shift}(\kappa), add to transcript. For each polynomial
    // we add a univariate opening claim {p(X), (\kappa, p(\kappa))} to the set of claims to be checked via batched KZG.
    FF kappa = transcript->get_challenge("kappa");

    // Add univariate opening claims for each polynomial.
    std::vector<OpeningClaim> opening_claims;
    // Compute evaluation T_{i-1}(\kappa)
    for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
        auto polynomial = Polynomial(T_prev[idx]);
        auto evaluation = polynomial.evaluate(kappa);
        transcript->send_to_verifier("T_prev_eval_" + std::to_string(idx + 1), evaluation);
        opening_claims.emplace_back(OpeningClaim{ polynomial, { kappa, evaluation } });
    }
    // Compute evaluation t_i^{shift}(\kappa)
    for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
        auto evaluation = t_shift[idx].evaluate(kappa);
        transcript->send_to_verifier("t_shift_eval_" + std::to_string(idx + 1), evaluation);
        opening_claims.emplace_back(OpeningClaim{ t_shift[idx], { kappa, evaluation } });
    }
    // Compute evaluation T_i(\kappa)
    for (size_t idx = 0; idx < Flavor::NUM_WIRES; ++idx) {
        auto polynomial = Polynomial(T_current[idx]);
        auto evaluation = polynomial.evaluate(kappa);
        transcript->send_to_verifier("T_current_eval_" + std::to_string(idx + 1), evaluation);
        opening_claims.emplace_back(OpeningClaim{ polynomial, { kappa, evaluation } });
    }

    FF alpha = transcript->get_challenge("alpha");

    // Construct batched polynomial to opened via KZG
    auto batched_polynomial = Polynomial(N);
    auto batched_eval = FF(0);
    auto alpha_pow = FF(1);
    for (auto& claim : opening_claims) {
        batched_polynomial.add_scaled(claim.polynomial, alpha_pow);
        batched_eval += alpha_pow * claim.opening_pair.evaluation;
        alpha_pow *= alpha;
    }

    // Construct and commit to KZG quotient polynomial q = (f - v) / (X - kappa)
    auto quotient = batched_polynomial;
    quotient[0] -= batched_eval;
    quotient.factor_roots(kappa);

    auto quotient_commitment = pcs_commitment_key->commit(quotient);
    transcript->send_to_verifier("KZG:W", quotient_commitment);

    proof.proof_data = transcript->proof_data;
    return proof;
}

template class MergeProver_<honk::flavor::Ultra>;
template class MergeProver_<honk::flavor::GoblinUltra>;

} // namespace bb::honk