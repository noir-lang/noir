#pragma once
#include <array>
#include <tuple>

#include "../polynomials/univariate.hpp"
#include "relation_parameters.hpp"

namespace proof_system::honk::sumcheck {

template <typename FF> class EllipticRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5; // degree(q_elliptic * q_beta * x^3) = 5

    static constexpr size_t NUM_CONSTRAINTS = 2;
    static constexpr std::array<size_t, NUM_CONSTRAINTS> CONSTRAINT_LENGTH = { 6, 5 };

    using RelationUnivariates = std::tuple<Univariate<FF, CONSTRAINT_LENGTH[0]>, Univariate<FF, CONSTRAINT_LENGTH[1]>>;
    using RelationValues = std::array<FF, NUM_CONSTRAINTS>;

    /**
     * @brief Expression for the Ultra Arithmetic gate.
     * @details The relation is defined as C(extended_edges(X)...) =
     *    TODO(#429): steal description from elliptic_widget.hpp
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    void add_edge_contribution(RelationUnivariates& evals,
                               const auto& extended_edges,
                               const RelationParameters<FF>&,
                               const FF& scaling_factor) const {
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both
        // TODO(luke): Formatter doesnt properly handle explicit scoping below so turning off. Whats up?
        // clang-format off
        // Contribution (1)
        {
            static constexpr size_t LENGTH = CONSTRAINT_LENGTH[0];
            auto x_1 = UnivariateView<FF, LENGTH>(extended_edges.w_r);
            auto y_1 = UnivariateView<FF, LENGTH>(extended_edges.w_o);

            auto x_2 = UnivariateView<FF, LENGTH>(extended_edges.w_l_shift);
            auto y_2 = UnivariateView<FF, LENGTH>(extended_edges.w_4_shift);
            auto x_3 = UnivariateView<FF, LENGTH>(extended_edges.w_r_shift);

            auto q_sign = UnivariateView<FF, LENGTH>(extended_edges.q_l);
            auto q_beta = UnivariateView<FF, LENGTH>(extended_edges.q_o);
            auto q_beta_sqr = UnivariateView<FF, LENGTH>(extended_edges.q_4);
            auto q_elliptic = UnivariateView<FF, LENGTH>(extended_edges.q_elliptic);

            auto beta_term = x_2 * x_1 * (x_3 + x_3 + x_1) * FF(-1); // -x_1 * x_2 * (2 * x_3 + x_1)
            auto beta_sqr_term = x_2 * x_2;                          // x_2^2
            auto leftovers = beta_sqr_term;                          // x_2^2
            beta_sqr_term *= (x_3 - x_1);                            // x_2^2 * (x_3 - x_1)
            auto sign_term = y_2 * y_1;                              // y_1 * y_2
            sign_term += sign_term;                                  // 2 * y_1 * y_2
            beta_term *= q_beta;                                     // -β * x_1 * x_2 * (2 * x_3 + x_1)
            beta_sqr_term *= q_beta_sqr;                             // β^2 * x_2^2 * (x_3 - x_1)
            sign_term *= q_sign;                                     // 2 * y_1 * y_2 * sign
            leftovers *= x_2;                                        // x_2^3
            leftovers += x_1 * x_1 * (x_3 + x_1);                    // x_2^3 + x_1 * (x_3 + x_1)
            leftovers -= (y_2 * y_2 + y_1 * y_1);                    // x_2^3 + x_1 * (x_3 + x_1) - y_2^2 - y_1^2

            // Can be found in class description
            auto x_identity = beta_term + beta_sqr_term + sign_term + leftovers;
            x_identity *= q_elliptic;
            x_identity *= scaling_factor;
            std::get<0>(evals) += x_identity;
        }
        // Contribution (2)
        {
            static constexpr size_t LENGTH = CONSTRAINT_LENGTH[1];
            auto x_1 = UnivariateView<FF, LENGTH>(extended_edges.w_r);
            auto y_1 = UnivariateView<FF, LENGTH>(extended_edges.w_o);

            auto x_2 = UnivariateView<FF, LENGTH>(extended_edges.w_l_shift);
            auto y_2 = UnivariateView<FF, LENGTH>(extended_edges.w_4_shift);
            auto x_3 = UnivariateView<FF, LENGTH>(extended_edges.w_r_shift);
            auto y_3 = UnivariateView<FF, LENGTH>(extended_edges.w_o_shift);

            auto q_sign = UnivariateView<FF, LENGTH>(extended_edges.q_l);
            auto q_beta = UnivariateView<FF, LENGTH>(extended_edges.q_o);
            auto q_elliptic = UnivariateView<FF, LENGTH>(extended_edges.q_elliptic);

            auto beta_term = x_2 * (y_3 + y_1) * q_beta;          // β * x_2 * (y_3 + y_1)
            auto sign_term = y_2 * (x_1 - x_3) * q_sign * FF(-1); // - signt * y_2 * (x_1 - x_3)
            // TODO: remove extra additions if we decide to stay with this implementation
            auto leftovers = x_1 * (y_3 + y_1) * FF(-1) + y_1 * (x_1 - x_3); // -x_1 * y_3 - x_1 * y_1 + y_1 * x_1 - y_1 * x_3

            auto y_identity = beta_term + sign_term + leftovers;
            y_identity *= q_elliptic;
            y_identity *= scaling_factor;
            std::get<1>(evals) += y_identity;
        }
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
        auto x_1 = purported_evaluations.w_r;
        auto y_1 = purported_evaluations.w_o;

        auto x_2 = purported_evaluations.w_l_shift;
        auto y_2 = purported_evaluations.w_4_shift;
        auto x_3 = purported_evaluations.w_r_shift;
        auto y_3 = purported_evaluations.w_o_shift;

        auto q_sign = purported_evaluations.q_l;
        auto q_beta = purported_evaluations.q_o;
        auto q_beta_sqr = purported_evaluations.q_4;
        auto q_elliptic = purported_evaluations.q_elliptic;

        // Contribution (1)
        auto beta_term = x_2 * x_1 * (x_3 + x_3 + x_1) * FF(-1); // -x_1 * x_2 * (2 * x_3 + x_1)
        auto beta_sqr_term = x_2 * x_2;                          // x_2^2
        auto leftovers = beta_sqr_term;                          // x_2^2
        beta_sqr_term *= (x_3 - x_1);                            // x_2^2 * (x_3 - x_1)
        auto sign_term = y_2 * y_1;                              // y_1 * y_2
        sign_term += sign_term;                                  // 2 * y_1 * y_2
        beta_term *= q_beta;                                     // -β * x_1 * x_2 * (2 * x_3 + x_1)
        beta_sqr_term *= q_beta_sqr;                             // β^2 * x_2^2 * (x_3 - x_1)
        sign_term *= q_sign;                                     // 2 * y_1 * y_2 * sign
        leftovers *= x_2;                                        // x_2^3
        leftovers += x_1 * x_1 * (x_3 + x_1);                    // x_2^3 + x_1 * (x_3 + x_1)
        leftovers -= (y_2 * y_2 + y_1 * y_1);                    // x_2^3 + x_1 * (x_3 + x_1) - y_2^2 - y_1^2

        // Can be found in class description
        auto x_identity = beta_term + beta_sqr_term + sign_term + leftovers;
        x_identity *= q_elliptic;
        std::get<0>(full_honk_relation_value) += x_identity;

        // Contribution (2)
        beta_term = x_2 * (y_3 + y_1) * q_beta;          // β * x_2 * (y_3 + y_1)
        sign_term = y_2 * (x_1 - x_3) * q_sign * FF(-1); // - signt * y_2 * (x_1 - x_3)
        // TODO: remove extra additions if we decide to stay with this implementation
        leftovers = x_1 * (y_3 + y_1) * FF(-1) + y_1 * (x_1 - x_3); // -x_1 * y_3 - x_1 * y_1 + y_1 * x_1 - y_1 * x_3

        auto y_identity = beta_term + sign_term + leftovers;
        y_identity *= q_elliptic;
        std::get<1>(full_honk_relation_value) += y_identity;
    };
};
// clang-format on
} // namespace proof_system::honk::sumcheck
