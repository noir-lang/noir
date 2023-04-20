#pragma once
#include <array>
#include <tuple>

#include "barretenberg/honk/flavor/flavor.hpp"
#include "../polynomials/univariate.hpp"
#include "relation.hpp"

// TODO(luke): Move this into ultra_arithmetic_relation.hpp.
namespace proof_system::honk::sumcheck {

template <typename FF> class UltraArithmeticRelationSecondary {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5; // degree(q_arith^3 * w_l) = 4
    using MULTIVARIATE = UltraArithmetization::POLYNOMIAL;

    /**
     * @brief Expression for the Ultra Arithmetic gate.
     * @details The relation is defined as C(extended_edges(X)...) =
     *    q_arith *
     *      (q_arith - 2) * (q_arith - 1) * (w_l + w_4 - w_l_shift + q_m)
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    void add_edge_contribution(Univariate<FF, RELATION_LENGTH>& evals,
                               const auto& extended_edges,
                               const RelationParameters<FF>&,
                               const FF& scaling_factor) const
    {
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both

        auto w_l = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
        auto w_4 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_4]);
        auto w_l_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_1_SHIFT]);
        auto q_m = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_M]);
        auto q_arith = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::QARITH]);

        auto tmp = w_l + w_4 - w_l_shift + q_m;
        tmp *= (q_arith - 2);
        tmp *= (q_arith - 1);
        tmp *= q_arith;
        tmp *= scaling_factor;
        evals += tmp;
    };

    void add_full_relation_value_contribution(FF& full_honk_relation_value,
                                              const auto& purported_evaluations,
                                              const RelationParameters<FF>&) const
    {
        auto w_l = purported_evaluations[MULTIVARIATE::W_L];
        auto w_4 = purported_evaluations[MULTIVARIATE::W_4];
        auto w_l_shift = purported_evaluations[MULTIVARIATE::W_1_SHIFT];
        auto q_m = purported_evaluations[MULTIVARIATE::Q_M];
        auto q_arith = purported_evaluations[MULTIVARIATE::QARITH];

        auto tmp = w_l + w_4 - w_l_shift + q_m;
        tmp *= (q_arith - 2);
        tmp *= (q_arith - 1);
        tmp *= q_arith;
        full_honk_relation_value += tmp;
    };
};
} // namespace proof_system::honk::sumcheck
