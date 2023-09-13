#pragma once
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system {

template <typename FF_> class ArithmeticRelationImpl {
  public:
    using FF = FF_;

    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 4;

    static constexpr size_t LEN_1 = 4; // arithmetic sub-relation
    template <template <size_t...> typename SubrelationAccumulatorsTemplate>
    using GetAccumulatorTypes = SubrelationAccumulatorsTemplate<LEN_1>;

    /**
     * @brief Expression for the StandardArithmetic gate.
     * @details The relation is defined as
     *    (q_m * w_r * w_l) + (q_l * w_l) + (q_r * w_r) + (q_o * w_o) + q_c
     *
     * @param accumulator the term being calculated by a sequence of calls to this function
     * @param new_term the term added to the accumulator in this iteration of the function
     * @param parameters inputs not varying between successive executions of this function
     * @param scaling_factor scales the new_term before incorporating it into the accumulator
     */
    template <typename AccumulatorTypes>
    void static accumulate(typename AccumulatorTypes::Accumulators& accumulators,
                           const auto& new_term,
                           [[maybe_unused]] const RelationParameters<FF>& parameters,
                           const FF& scaling_factor)
    {
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both

        using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
        auto w_l = View(new_term.w_l);
        auto w_r = View(new_term.w_r);
        auto w_o = View(new_term.w_o);
        auto q_m = View(new_term.q_m);
        auto q_l = View(new_term.q_l);
        auto q_r = View(new_term.q_r);
        auto q_o = View(new_term.q_o);
        auto q_c = View(new_term.q_c);

        auto tmp = w_l * (q_m * w_r + q_l);
        tmp += q_r * w_r;
        tmp += q_o * w_o;
        tmp += q_c;
        tmp *= scaling_factor;
        std::get<0>(accumulators) += tmp;
    };
};

template <typename FF> using ArithmeticRelation = Relation<ArithmeticRelationImpl<FF>>;
} // namespace proof_system
