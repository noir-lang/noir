#pragma once
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system {

template <typename FF_> class EccOpQueueRelationImpl {
  public:
    using FF = FF_;
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 3; // degree(q * (w - w_op_queue)) = 2

    static constexpr size_t LEN_1 = 3; // wire - op-queue-wire consistency sub-relation 1
    static constexpr size_t LEN_2 = 3; // wire - op-queue-wire consistency sub-relation 2
    static constexpr size_t LEN_3 = 3; // wire - op-queue-wire consistency sub-relation 3
    static constexpr size_t LEN_4 = 3; // wire - op-queue-wire consistency sub-relation 4
    static constexpr size_t LEN_5 = 3; // op-queue-wire vanishes sub-relation 1
    static constexpr size_t LEN_6 = 3; // op-queue-wire vanishes sub-relation 2
    static constexpr size_t LEN_7 = 3; // op-queue-wire vanishes sub-relation 3
    static constexpr size_t LEN_8 = 3; // op-queue-wire vanishes sub-relation 4
    template <template <size_t...> typename SubrelationAccumulatorsTemplate>
    using GetAccumulatorTypes = SubrelationAccumulatorsTemplate<LEN_1, LEN_2, LEN_3, LEN_4, LEN_5, LEN_6, LEN_7, LEN_8>;

    /**
     * @brief Expression for the generalized permutation sort gate.
     * @details The relation is defined as C(extended_edges(X)...) =
     *    \alpha_{base} *
     *       ( \Sum_{i=0}^3 \alpha^i * (w_i - w_{op,i}) * \chi_{ecc_op} +
     *         \Sum_{i=0}^3 \alpha^{i+4} w_{op,i} * \bar{\chi}_{ecc_op} )
     *
     * where w_{op,i} are the ecc op gate wires, \chi_{ecc_op} is the indicator for the portion of the domain
     * representing ecc op gates and \bar{\chi} is the indicator on the complementary domain.
     *
     * The first four sub-relations check that the values in the conventional wires are identical to the values in the
     * ecc op wires over the portion of the execution trace representing ECC op queue gates. The next four check
     * that the op wire polynomials are identically zero everywhere else.
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename AccumulatorTypes>
    void static accumulate(typename AccumulatorTypes::Accumulators& accumulators,
                           const auto& extended_edges,
                           const RelationParameters<FF>&,
                           const FF& scaling_factor)
    {
        // OPTIMIZATION?: Karatsuba in general, at least for some degrees?
        //       See https://hackmd.io/xGLuj6biSsCjzQnYN-pEiA?both
        using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;
        auto w_1 = View(extended_edges.w_l);
        auto w_2 = View(extended_edges.w_r);
        auto w_3 = View(extended_edges.w_o);
        auto w_4 = View(extended_edges.w_4);
        auto op_wire_1 = View(extended_edges.ecc_op_wire_1);
        auto op_wire_2 = View(extended_edges.ecc_op_wire_2);
        auto op_wire_3 = View(extended_edges.ecc_op_wire_3);
        auto op_wire_4 = View(extended_edges.ecc_op_wire_4);
        auto lagrange_ecc_op = View(extended_edges.lagrange_ecc_op);

        // If lagrange_ecc_op is the indicator for ecc_op_gates, this is the indicator for the complement
        auto complement_ecc_op = lagrange_ecc_op * FF(-1) + FF(1);

        // Contribution (1)
        auto tmp = op_wire_1 - w_1;
        tmp *= lagrange_ecc_op;
        tmp *= scaling_factor;
        std::get<0>(accumulators) += tmp;

        // Contribution (2)
        tmp = op_wire_2 - w_2;
        tmp *= lagrange_ecc_op;
        tmp *= scaling_factor;
        std::get<1>(accumulators) += tmp;

        // Contribution (3)
        tmp = op_wire_3 - w_3;
        tmp *= lagrange_ecc_op;
        tmp *= scaling_factor;
        std::get<2>(accumulators) += tmp;

        // Contribution (4)
        tmp = op_wire_4 - w_4;
        tmp *= lagrange_ecc_op;
        tmp *= scaling_factor;
        std::get<3>(accumulators) += tmp;

        // Contribution (5)
        tmp = op_wire_1 * complement_ecc_op;
        tmp *= scaling_factor;
        std::get<4>(accumulators) += tmp;

        // Contribution (6)
        tmp = op_wire_2 * complement_ecc_op;
        tmp *= scaling_factor;
        std::get<5>(accumulators) += tmp;

        // Contribution (7)
        tmp = op_wire_3 * complement_ecc_op;
        tmp *= scaling_factor;
        std::get<6>(accumulators) += tmp;

        // Contribution (8)
        tmp = op_wire_4 * complement_ecc_op;
        tmp *= scaling_factor;
        std::get<7>(accumulators) += tmp;
    };
};

template <typename FF> using EccOpQueueRelation = Relation<EccOpQueueRelationImpl<FF>>;

} // namespace proof_system
