#pragma once

#include <array>
#include <vector>
namespace proof_system::honk::sumcheck {

/**
 * @brief Contains the multi-linear evaluations of the polynomials at the challenge point 'u'.
 * These are computed by the prover and need to be checked using a multi-linear PCS like Gemini.
 */
template <typename Flavor> struct SumcheckOutput {
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;
    // u = (u_0, ..., u_{d-1})
    std::vector<FF> challenge_point;
    // Evaluations in `u` of the polynomials used in Sumcheck
    ClaimedEvaluations purported_evaluations;

    SumcheckOutput()
        : purported_evaluations(std::array<FF, Flavor::NUM_ALL_ENTITIES>()){};

    SumcheckOutput(const std::vector<FF>& _challenge_point, const ClaimedEvaluations& _purported_evaluations)
        : challenge_point(_challenge_point)
        , purported_evaluations(_purported_evaluations){};

    SumcheckOutput& operator=(SumcheckOutput&& other)
    {
        challenge_point = other.challenge_point;
        purported_evaluations = other.purported_evaluations;
        return *this;
    };

    SumcheckOutput(const SumcheckOutput& other)
        : challenge_point(other.challenge_point)
        , purported_evaluations(other.purported_evaluations){};

    bool operator==(const SumcheckOutput& other) const
    {
        bool result{ false };
        result = challenge_point == other.challenge_point;
        result = purported_evaluations._data == other.purported_evaluations._data;
        return result;
    };
};
} // namespace proof_system::honk::sumcheck
