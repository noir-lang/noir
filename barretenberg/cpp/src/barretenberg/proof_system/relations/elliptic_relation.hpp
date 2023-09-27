#pragma once
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system {

template <typename FF_> class EllipticRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6; // degree(q_elliptic * q_beta * x^3) = 5

    static constexpr size_t LEN_1 = 6; // x-coordinate sub-relation
    static constexpr size_t LEN_2 = 5; // y-coordinate sub-relation
    template <template <size_t...> typename SubrelationAccumulatorsTemplate>
    using GetAccumulatorTypes = SubrelationAccumulatorsTemplate<LEN_1, LEN_2>;

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
    template <typename AccumulatorTypes>
    static void accumulate(typename AccumulatorTypes::Accumulators& accumulators,
                           const auto& extended_edges,
                           const RelationParameters<FF>&,
                           const FF& scaling_factor){
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both
        // Note: Formatter turned off since it doesnt properly handle the explicit scoping below.
        // clang-format off
        // Contribution (1)
        {
            using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
            auto x_1 = View(extended_edges.w_r);
            auto y_1 = View(extended_edges.w_o);

            auto x_2 = View(extended_edges.w_l_shift);
            auto y_2 = View(extended_edges.w_4_shift);
            auto x_3 = View(extended_edges.w_r_shift);

            auto q_sign = View(extended_edges.q_l);
            auto q_beta = View(extended_edges.q_o);
            auto q_beta_sqr = View(extended_edges.q_4);
            auto q_elliptic = View(extended_edges.q_elliptic);

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
            std::get<0>(accumulators) += x_identity;
        }
        // Contribution (2)
        {
            using View = typename std::tuple_element<1, typename AccumulatorTypes::AccumulatorViews>::type;
            auto x_1 = View(extended_edges.w_r);
            auto y_1 = View(extended_edges.w_o);

            auto x_2 = View(extended_edges.w_l_shift);
            auto y_2 = View(extended_edges.w_4_shift);
            auto x_3 = View(extended_edges.w_r_shift);
            auto y_3 = View(extended_edges.w_o_shift);

            auto q_sign = View(extended_edges.q_l);
            auto q_beta = View(extended_edges.q_o);
            auto q_elliptic = View(extended_edges.q_elliptic);

            auto beta_term = x_2 * (y_3 + y_1) * q_beta;          // β * x_2 * (y_3 + y_1)
            auto sign_term = y_2 * (x_1 - x_3) * q_sign * FF(-1); // - signt * y_2 * (x_1 - x_3)
            // TODO: remove extra additions if we decide to stay with this implementation
            auto leftovers = x_1 * (y_3 + y_1) * FF(-1) + y_1 * (x_1 - x_3); // -x_1 * y_3 - x_1 * y_1 + y_1 * x_1 - y_1 * x_3

            auto y_identity = beta_term + sign_term + leftovers;
            y_identity *= q_elliptic;
            y_identity *= scaling_factor;
            std::get<1>(accumulators) += y_identity;
        }
    };
};

template <typename FF>
using EllipticRelation = Relation<EllipticRelationImpl<FF>>;
// clang-format on
} // namespace proof_system
