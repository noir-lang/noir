#pragma once
#include <array>
#include <tuple>

#include "../polynomials/univariate.hpp"
#include "relation_parameters.hpp"

namespace proof_system::honk::sumcheck {

template <typename FF> class ArithmeticRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 4;

    static constexpr size_t NUM_CONSTRAINTS = 1;
    static constexpr std::array<size_t, NUM_CONSTRAINTS> CONSTRAINT_LENGTH = { 4 };

    using RelationUnivariates = std::tuple<Univariate<FF, CONSTRAINT_LENGTH[0]>>;
    using RelationValues = std::array<FF, NUM_CONSTRAINTS>;

    /**
     * @brief Expression for the StandardArithmetic gate.
     * @details The relation is defined as C(extended_edges(X)...) =
     *    (q_m * w_r * w_l) + (q_l * w_l) + (q_r * w_r) + (q_o * w_o) + q_c
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    void add_edge_contribution(RelationUnivariates& evals,
                               const auto& extended_edges,
                               const RelationParameters<FF>&,
                               const FF& scaling_factor) const
    {
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both

        auto w_l = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_l);
        auto w_r = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_r);
        auto w_o = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_o);
        auto q_m = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_m);
        auto q_l = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_l);
        auto q_r = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_r);
        auto q_o = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_o);
        auto q_c = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_c);

        auto tmp = w_l * (q_m * w_r + q_l);
        tmp += q_r * w_r;
        tmp += q_o * w_o;
        tmp += q_c;
        tmp *= scaling_factor;
        std::get<0>(evals) += tmp;
    };

    /**
     * @brief Add the result of each identity in this relation evaluated at the multivariate evaluations produced by the
     * Sumcheck Prover.
     *
     * @param full_honk_relation_value
     * @param purported_evaluations
     */
    void add_full_relation_value_contribution(RelationValues& full_honk_relation_value,
                                              const auto& purported_evaluations,
                                              const RelationParameters<FF>&) const
    {
        auto w_l = purported_evaluations.w_l;
        auto w_r = purported_evaluations.w_r;
        auto w_o = purported_evaluations.w_o;
        auto q_m = purported_evaluations.q_m;
        auto q_l = purported_evaluations.q_l;
        auto q_r = purported_evaluations.q_r;
        auto q_o = purported_evaluations.q_o;
        auto q_c = purported_evaluations.q_c;

        std::get<0>(full_honk_relation_value) += w_l * (q_m * w_r + q_l);
        std::get<0>(full_honk_relation_value) += q_r * w_r;
        std::get<0>(full_honk_relation_value) += q_o * w_o;
        std::get<0>(full_honk_relation_value) += q_c;
    };
};
} // namespace proof_system::honk::sumcheck
