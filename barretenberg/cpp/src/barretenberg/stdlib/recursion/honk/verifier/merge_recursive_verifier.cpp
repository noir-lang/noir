#include "barretenberg/stdlib/recursion/honk/verifier/merge_recursive_verifier.hpp"

namespace proof_system::plonk::stdlib::recursion::goblin {

template <typename CircuitBuilder>
MergeRecursiveVerifier_<CircuitBuilder>::MergeRecursiveVerifier_(CircuitBuilder* builder)
    : builder(builder)
{}

/**
 * @brief Construct recursive verifier for Goblin Merge protocol, up to but not including the pairing
 *
 * @tparam Flavor
 * @param proof
 * @return std::array<typename Flavor::GroupElement, 2> Inputs to final pairing
 */
template <typename CircuitBuilder>
std::array<typename bn254<CircuitBuilder>::Element, 2> MergeRecursiveVerifier_<CircuitBuilder>::verify_proof(
    const plonk::proof& proof)
{
    transcript = std::make_shared<Transcript>(builder, proof.proof_data);

    // Receive commitments [t_i^{shift}], [T_{i-1}], and [T_i]
    std::array<Commitment, NUM_WIRES> C_T_prev;
    std::array<Commitment, NUM_WIRES> C_t_shift;
    std::array<Commitment, NUM_WIRES> C_T_current;
    for (size_t idx = 0; idx < NUM_WIRES; ++idx) {
        C_T_prev[idx] = transcript->template receive_from_prover<Commitment>("T_PREV_" + std::to_string(idx + 1));
        C_t_shift[idx] = transcript->template receive_from_prover<Commitment>("t_SHIFT_" + std::to_string(idx + 1));
        C_T_current[idx] = transcript->template receive_from_prover<Commitment>("T_CURRENT_" + std::to_string(idx + 1));
    }

    FF kappa = transcript->get_challenge("kappa");

    // Receive transcript poly evaluations and add corresponding univariate opening claims {(\kappa, p(\kappa), [p(X)]}
    std::array<FF, NUM_WIRES> T_prev_evals;
    std::array<FF, NUM_WIRES> t_shift_evals;
    std::array<FF, NUM_WIRES> T_current_evals;
    std::vector<OpeningClaim> opening_claims;
    for (size_t idx = 0; idx < NUM_WIRES; ++idx) {
        T_prev_evals[idx] = transcript->template receive_from_prover<FF>("T_prev_eval_" + std::to_string(idx + 1));
        opening_claims.emplace_back(OpeningClaim{ { kappa, T_prev_evals[idx] }, C_T_prev[idx] });
    }
    for (size_t idx = 0; idx < NUM_WIRES; ++idx) {
        t_shift_evals[idx] = transcript->template receive_from_prover<FF>("t_shift_eval_" + std::to_string(idx + 1));
        opening_claims.emplace_back(OpeningClaim{ { kappa, t_shift_evals[idx] }, C_t_shift[idx] });
    }
    for (size_t idx = 0; idx < NUM_WIRES; ++idx) {
        T_current_evals[idx] =
            transcript->template receive_from_prover<FF>("T_current_eval_" + std::to_string(idx + 1));
        opening_claims.emplace_back(OpeningClaim{ { kappa, T_current_evals[idx] }, C_T_current[idx] });
    }

    // Check the identity T_i(\kappa) = T_{i-1}(\kappa) + t_i^{shift}(\kappa)
    for (size_t idx = 0; idx < NUM_WIRES; ++idx) {
        T_current_evals[idx].assert_equal(T_prev_evals[idx] + t_shift_evals[idx]);
    }

    FF alpha = transcript->get_challenge("alpha");

    // Constuct batched commitment and batched evaluation from constituents using batching challenge \alpha
    std::vector<FF> scalars;
    std::vector<Commitment> commitments;
    scalars.emplace_back(FF(builder, 1));
    commitments.emplace_back(opening_claims[0].commitment);
    auto batched_eval = opening_claims[0].opening_pair.evaluation;
    auto alpha_pow = alpha;
    for (size_t idx = 1; idx < opening_claims.size(); ++idx) {
        auto& claim = opening_claims[idx];
        scalars.emplace_back(alpha_pow);
        commitments.emplace_back(claim.commitment);
        batched_eval += alpha_pow * claim.opening_pair.evaluation;
        alpha_pow *= alpha;
    }

    auto batched_commitment = Commitment::batch_mul(commitments, scalars);

    OpeningClaim batched_claim = { { kappa, batched_eval }, batched_commitment };

    auto pairing_points = KZG::compute_pairing_points(batched_claim, transcript);

    return pairing_points;
}

template class MergeRecursiveVerifier_<GoblinUltraCircuitBuilder>;

} // namespace proof_system::plonk::stdlib::recursion::goblin
