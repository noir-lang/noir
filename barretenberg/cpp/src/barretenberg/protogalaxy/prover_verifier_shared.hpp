#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <vector>

namespace bb {
/**
 * @brief Compute the gate challenges used in the combiner calculation.
 * @details This is Step 8 of the protocol as written in the paper.
 */
std::vector<fr> update_gate_challenges(const fr& perturbator_challenge,
                                       const std::vector<fr>& gate_challenges,
                                       const std::vector<fr>& init_challenges);

/**
 * @brief For a new round challenge δ at each iteration of the ProtoGalaxy protocol, compute the vector
 * [δ, δ^2,..., δ^t] where t = logn and n is the size of the instance.
 */
std::vector<fr> compute_round_challenge_pows(const size_t log_instance_size, const fr& round_challenge);

} // namespace bb