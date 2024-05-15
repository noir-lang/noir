#pragma once

#include <array>
#include <optional>
#include <vector>

namespace bb {

/**
 * @brief Contains the evaluations of multilinear polynomials \f$ P_1, \ldots, P_N\f$ at the challenge point \f$\vec u
 * =(u_0,\ldots, u_{d-1})\f$. These are computed by \ref bb::SumcheckProver< Flavor > "Sumcheck Prover" and need to be
 * checked using Zeromorph.
 */
template <typename Flavor> struct SumcheckOutput {
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::AllValues;
    // \f$ \vec u = (u_0, ..., u_{d-1}) \f$
    std::vector<FF> challenge;
    // Evaluations in \f$ \vec u \f$ of the polynomials used in Sumcheck
    ClaimedEvaluations claimed_evaluations;
    // Whether or not the evaluations of multilinear polynomials \f$ P_1, \ldots, P_N \f$  and final Sumcheck evaluation
    // have been confirmed
    std::optional<bool> verified = false; // optional b/c this struct is shared by the Prover/Verifier
};
} // namespace bb
