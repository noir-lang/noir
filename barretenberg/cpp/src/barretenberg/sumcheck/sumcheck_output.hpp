#pragma once

#include <array>
#include <optional>
#include <vector>

namespace bb {

/**
 * @brief Contains the multi-linear evaluations of the polynomials at the challenge point 'u'.
 * These are computed by the prover and need to be checked using a multi-linear PCS like Gemini.
 */
template <typename Flavor> struct SumcheckOutput {
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::AllValues;
    // u = (u_0, ..., u_{d-1})
    std::vector<FF> challenge;
    // Evaluations in `u` of the polynomials used in Sumcheck
    ClaimedEvaluations claimed_evaluations;
    // Whether or not the claimed multilinear evaluations and final sumcheck evaluation have been confirmed
    std::optional<bool> verified = false; // optional b/c this struct is shared by the Prover/Verifier
};
} // namespace bb
