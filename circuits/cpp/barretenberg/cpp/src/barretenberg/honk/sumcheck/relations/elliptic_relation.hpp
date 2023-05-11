#pragma once
#include <array>
#include <tuple>

#include "../polynomials/univariate.hpp"
#include "relation.hpp"

namespace proof_system::honk::sumcheck {

template <typename FF> class EllipticRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6; // degree(q_arith^2 * q_m * w_r * w_l) = 5

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
    void add_edge_contribution(Univariate<FF, RELATION_LENGTH>& evals,
                               const auto& extended_edges,
                               const RelationParameters<FF>&,
                               const FF& scaling_factor) const
    {
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both

        auto x_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_r);
        auto y_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_o);

        auto x_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_l_shift);
        auto y_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_4_shift);
        auto x_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_r_shift);
        auto y_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_o_shift);

        auto q_sign = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_l);
        auto q_beta = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_o);
        auto q_beta_sqr = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_4);
        auto q_elliptic = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_elliptic);

        static const FF fake_alpha_1 = FF(1);
        static const FF fake_alpha_2 = fake_alpha_1 * fake_alpha_1;

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
        x_identity *= fake_alpha_1;

        beta_term = x_2 * (y_3 + y_1) * q_beta;          // β * x_2 * (y_3 + y_1)
        sign_term = y_2 * (x_1 - x_3) * q_sign * FF(-1); // - signt * y_2 * (x_1 - x_3)
        // TODO: remove extra additions if we decide to stay with this implementation
        leftovers = x_1 * (y_3 + y_1) * FF(-1) + y_1 * (x_1 - x_3); // -x_1 * y_3 - x_1 * y_1 + y_1 * x_1 - y_1 * x_3

        auto y_identity = beta_term + sign_term + leftovers;
        y_identity *= fake_alpha_2;

        auto tmp = x_identity + y_identity;
        tmp *= q_elliptic;

        tmp *= scaling_factor;

        evals += tmp;
    };

    void add_full_relation_value_contribution(FF& full_honk_relation_value,
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

        static const FF fake_alpha_1 = FF(1);
        static const FF fake_alpha_2 = fake_alpha_1 * fake_alpha_1;

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
        x_identity *= fake_alpha_1;

        beta_term = x_2 * (y_3 + y_1) * q_beta;          // β * x_2 * (y_3 + y_1)
        sign_term = y_2 * (x_1 - x_3) * q_sign * FF(-1); // - signt * y_2 * (x_1 - x_3)
        // TODO: remove extra additions if we decide to stay with this implementation
        leftovers = x_1 * (y_3 + y_1) * FF(-1) + y_1 * (x_1 - x_3); // -x_1 * y_3 - x_1 * y_1 + y_1 * x_1 - y_1 * x_3

        auto y_identity = beta_term + sign_term + leftovers;
        y_identity *= fake_alpha_2;

        auto tmp = x_identity + y_identity;
        tmp *= q_elliptic;

        full_honk_relation_value += tmp;
    };
};
} // namespace proof_system::honk::sumcheck
