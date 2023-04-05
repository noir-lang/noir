#pragma once

#include <array>
#include <vector>
namespace proof_system::honk::sumcheck {

/**
 * @brief Contains the multi-linear evaluations of the polynomials at the challenge point 'u'.
 * These are computed by the prover and need to be checked using a multi-linear PCS like Gemini.
 */
template <typename FF> struct SumcheckOutput {
    // u = (u_0, ..., u_{d-1})
    std::vector<FF> challenge_point;
    // Evaluations in `u` of the polynomials used in Sumcheck
    std::array<FF, honk::StandardArithmetization::NUM_POLYNOMIALS> evaluations;

    bool operator==(const SumcheckOutput& other) const = default;
};
} // namespace proof_system::honk::sumcheck
