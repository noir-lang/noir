#pragma once

#include <array>
#include <optional>
#include <vector>

namespace bb {

/**
 * @brief This structure is created to contain various polynomials and constants required by ZK Sumcheck.
 *
 */
template <typename Flavor> struct ZKSumcheckData {
    using FF = typename Flavor::FF;
    /**
     * @brief The total algebraic degree of the Sumcheck relation \f$ F \f$ as a polynomial in Prover Polynomials
     * \f$P_1,\ldots, P_N\f$.
     */
    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = Flavor::MAX_PARTIAL_RELATION_LENGTH;
    // The number of all witnesses including shifts and derived witnesses from flavors that have ZK,
    // otherwise, set this constant to 0.
    /**
     * @brief The total algebraic degree of the Sumcheck relation \f$ F \f$ as a polynomial in Prover Polynomials
     * \f$P_1,\ldots, P_N\f$ <b> incremented by </b> 1, i.e. it is equal \ref MAX_PARTIAL_RELATION_LENGTH
     * "MAX_PARTIAL_RELATION_LENGTH + 1".
     */
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;
    // Initialize the length of the array of evaluation masking scalars as 0 for non-ZK Flavors and as
    // NUM_ALL_WITNESS_ENTITIES for ZK FLavors
    static constexpr size_t MASKING_SCALARS_LENGTH = Flavor::HasZK ? Flavor::NUM_ALL_WITNESS_ENTITIES : 0;
    // Array of random scalars used to hide the witness info from leaking through the claimed evaluations
    using EvalMaskingScalars = std::array<FF, MASKING_SCALARS_LENGTH>;
    // Auxiliary table that represents the evaluations of quadratic polynomials r_j * X(1-X) at 0,...,
    // MAX_PARTIAL_RELATION_LENGTH - 1
    using EvaluationMaskingTable = std::array<bb::Univariate<FF, MAX_PARTIAL_RELATION_LENGTH>, MASKING_SCALARS_LENGTH>;
    // The size of the LibraUnivariates. We ensure that they do not take extra space when Flavor runs non-ZK
    // Sumcheck.
    static constexpr size_t LIBRA_UNIVARIATES_LENGTH = Flavor::HasZK ? Flavor::BATCHED_RELATION_PARTIAL_LENGTH : 0;
    // Container for the Libra Univariates. Their number depends on the size of the circuit.
    using LibraUnivariates = std::vector<bb::Univariate<FF, LIBRA_UNIVARIATES_LENGTH>>;
    // Container for the evaluations of Libra Univariates that have to be proven.
    using ClaimedLibraEvaluations = std::vector<FF>;

    EvalMaskingScalars eval_masking_scalars;
    EvaluationMaskingTable masking_terms_evaluations;
    LibraUnivariates libra_univariates;
    FF libra_scaling_factor{ 1 };
    FF libra_challenge;
    FF libra_running_sum;
    ClaimedLibraEvaluations libra_evaluations;
};

} // namespace bb
