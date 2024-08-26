#include "barretenberg/protogalaxy/prover_verifier_shared.hpp"

namespace bb {
std::vector<fr> update_gate_challenges(const fr& perturbator_challenge,
                                       const std::vector<fr>& gate_challenges,
                                       const std::vector<fr>& init_challenges)
{
    auto log_instance_size = gate_challenges.size();
    std::vector<fr> next_gate_challenges(log_instance_size);

    for (size_t idx = 0; idx < log_instance_size; idx++) {
        next_gate_challenges[idx] = gate_challenges[idx] + perturbator_challenge * init_challenges[idx];
    }
    return next_gate_challenges;
}

/**
 * @brief For a new round challenge δ at each iteration of the ProtoGalaxy protocol, compute the vector
 * [δ, δ^2,..., δ^t] where t = logn and n is the size of the instance.
 */
std::vector<fr> compute_round_challenge_pows(const size_t log_instance_size, const fr& round_challenge)
{
    std::vector<fr> pows(log_instance_size);
    pows[0] = round_challenge;
    for (size_t i = 1; i < log_instance_size; i++) {
        pows[i] = pows[i - 1].sqr();
    }
    return pows;
}

} // namespace bb