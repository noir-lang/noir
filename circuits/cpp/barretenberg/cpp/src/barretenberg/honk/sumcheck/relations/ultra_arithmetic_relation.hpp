#pragma once
#include <array>
#include <tuple>

#include "../polynomials/univariate.hpp"
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system::honk::sumcheck {

template <typename FF> class UltraArithmeticRelationBase {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6; // degree(q_arith^2 * q_m * w_r * w_l) = 5

    static constexpr size_t LEN_1 = 6; // primary arithmetic sub-relation
    static constexpr size_t LEN_2 = 5; // secondary arithmetic sub-relation
    template <template <size_t...> typename AccumulatorTypesContainer>
    using AccumulatorTypesBase = AccumulatorTypesContainer<LEN_1, LEN_2>;

    /**
     * @brief Expression for the Ultra Arithmetic gate.
     * @details This relation encapsulates several idenitities, toggled by the value of q_arith in [0, 1, 2, 3, ...].
     * The following description is reproduced from the Plonk analog 'plookup_arithmetic_widget':
     * The whole formula is:
     *
     * q_arith * ( ( (-1/2) * (q_arith - 3) * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c ) +
     * (q_arith - 1)*( α * (q_arith - 2) * (w_1 + w_4 - w_1_omega + q_m) + w_4_omega) ) = 0
     *
     * This formula results in several cases depending on q_arith:
     * 1. q_arith == 0: Arithmetic gate is completely disabled
     *
     * 2. q_arith == 1: Everything in the minigate on the right is disabled. The equation is just a standard plonk
     * equation with extra wires: q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c = 0
     *
     * 3. q_arith == 2: The (w_1 + w_4 - ...) term is disabled. THe equation is:
     * (1/2) * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c + w_4_omega = 0
     * It allows defining w_4 at next index (w_4_omega) in terms of current wire values
     *
     * 4. q_arith == 3: The product of w_1 and w_2 is disabled, but a mini addition gate is enabled. α² allows us to
     * split the equation into two:
     *
     * q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c + 2 * w_4_omega = 0
     *
     * w_1 + w_4 - w_1_omega + q_m = 0  (we are reusing q_m here)
     *
     * 5. q_arith > 3: The product of w_1 and w_2 is scaled by (q_arith - 3), while the w_4_omega term is scaled by
     * (q_arith
     * - 1). The equation can be split into two:
     *
     * (q_arith - 3)* q_m * w_1 * w_ 2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c + (q_arith - 1) * w_4_omega
     * = 0
     *
     * w_1 + w_4 - w_1_omega + q_m = 0
     *
     * The problem that q_m is used both in both equations can be dealt with by appropriately changing selector values
     * at the next gate. Then we can treat (q_arith - 1) as a simulated q_6 selector and scale q_m to handle (q_arith -
     * 3) at product.
     *
     * The The relation is
     * defined as C(extended_edges(X)...) = q_arith * [ -1/2(q_arith - 3)(q_m * w_r * w_l) + (q_l * w_l) + (q_r * w_r) +
     * (q_o * w_o) + (q_4 * w_4) + q_c + (q_arith - 1)w_4_shift ]
     *
     *    q_arith *
     *      (q_arith - 2) * (q_arith - 1) * (w_l + w_4 - w_l_shift + q_m)
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename AccumulatorTypes>
    void static add_edge_contribution_impl(typename AccumulatorTypes::Accumulators& evals,
                                           const auto& extended_edges,
                                           const RelationParameters<FF>&,
                                           const FF& scaling_factor){
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both
        // clang-format off
        // Contribution 1
        {   
            using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
            auto w_l = View(extended_edges.w_l);
            auto w_r = View(extended_edges.w_r);
            auto w_o = View(extended_edges.w_o);
            auto w_4 = View(extended_edges.w_4);
            auto w_4_shift = View(extended_edges.w_4_shift);
            auto q_m = View(extended_edges.q_m);
            auto q_l = View(extended_edges.q_l);
            auto q_r = View(extended_edges.q_r);
            auto q_o = View(extended_edges.q_o);
            auto q_4 = View(extended_edges.q_4);
            auto q_c = View(extended_edges.q_c);
            auto q_arith = View(extended_edges.q_arith);

            static const FF neg_half = FF(-2).invert();

            auto tmp = (q_arith - 3) * (q_m * w_r * w_l) * neg_half;
            tmp += (q_l * w_l) + (q_r * w_r) + (q_o * w_o) + (q_4 * w_4) + q_c;
            tmp += (q_arith - 1) * w_4_shift;
            tmp *= q_arith;
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 2
        {
            using View = typename std::tuple_element<1, typename AccumulatorTypes::AccumulatorViews>::type;
            auto w_l = View(extended_edges.w_l);
            auto w_4 = View(extended_edges.w_4);
            auto w_l_shift = View(extended_edges.w_l_shift);
            auto q_m = View(extended_edges.q_m);
            auto q_arith = View(extended_edges.q_arith);

            auto tmp = w_l + w_4 - w_l_shift + q_m;
            tmp *= (q_arith - 2);
            tmp *= (q_arith - 1);
            tmp *= q_arith;
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
    }; // namespace proof_system::honk::sumcheck
};

template <typename FF>
using UltraArithmeticRelation = RelationWrapper<FF, UltraArithmeticRelationBase>;

// clang-format on
} // namespace proof_system::honk::sumcheck
