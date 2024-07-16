#pragma once
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class EccOpQueueRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 8> SUBRELATION_PARTIAL_LENGTHS{
        3, // wire - op-queue-wire consistency sub-relation 1
        3, // wire - op-queue-wire consistency sub-relation 2
        3, // wire - op-queue-wire consistency sub-relation 3
        3, // wire - op-queue-wire consistency sub-relation 4
        3, // op-queue-wire vanishes sub-relation 1
        3, // op-queue-wire vanishes sub-relation 2
        3, // op-queue-wire vanishes sub-relation 3
        3  // op-queue-wire vanishes sub-relation 4
    };
    /**
     * @brief For ZK-Flavors: The degrees of subrelations considered as polynomials only in witness polynomials,
     * i.e. all selectors and public polynomials are treated as constants.
     *
     */
    static constexpr std::array<size_t, 8> SUBRELATION_WITNESS_DEGREES{
        1, // wire - op-queue-wire consistency sub-relation 1
        1, // wire - op-queue-wire consistency sub-relation 2
        1, // wire - op-queue-wire consistency sub-relation 3
        1, // wire - op-queue-wire consistency sub-relation 4
        1, // op-queue-wire vanishes sub-relation 1
        1, // op-queue-wire vanishes sub-relation 2
        1, // op-queue-wire vanishes sub-relation 3
        1  // op-queue-wire vanishes sub-relation 4
    };

    template <typename AllEntities> inline static bool skip([[maybe_unused]] const AllEntities& in)
    {
        // The prover can skip execution of this relation altogether since an honest input will lead to a zero
        // contribution at every row, even when the selector lagrange_ecc_op is on
        return true;
    }

    /**
     * @brief Expression for the generalized permutation sort gate.
     * @details The relation is defined as C(in(X)...) =
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
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    inline static void accumulate(ContainerOverSubrelations& accumulators,
                                  const AllEntities& in,
                                  const Parameters&,
                                  const FF& scaling_factor)
    {
        BB_OP_COUNT_TIME_NAME("EccOp::accumulate");
        using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
        using View = typename Accumulator::View;

        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);
        auto w_4 = View(in.w_4);
        auto op_wire_1 = View(in.ecc_op_wire_1);
        auto op_wire_2 = View(in.ecc_op_wire_2);
        auto op_wire_3 = View(in.ecc_op_wire_3);
        auto op_wire_4 = View(in.ecc_op_wire_4);
        auto lagrange_ecc_op = View(in.lagrange_ecc_op);

        // If lagrange_ecc_op is the indicator for ecc_op_gates, this is the indicator for the complement
        auto lagrange_by_scaling = lagrange_ecc_op * scaling_factor;
        auto complement_ecc_op_by_scaling = -lagrange_by_scaling + scaling_factor;

        // Contribution (1)
        auto tmp = op_wire_1 - w_1;
        tmp *= lagrange_by_scaling;
        std::get<0>(accumulators) += tmp;

        // Contribution (2)
        tmp = op_wire_2 - w_2;
        tmp *= lagrange_by_scaling;
        std::get<1>(accumulators) += tmp;

        // Contribution (3)
        tmp = op_wire_3 - w_3;
        tmp *= lagrange_by_scaling;
        std::get<2>(accumulators) += tmp;

        // Contribution (4)
        tmp = op_wire_4 - w_4;
        tmp *= lagrange_by_scaling;
        std::get<3>(accumulators) += tmp;

        // Contribution (5)
        tmp = op_wire_1 * complement_ecc_op_by_scaling;
        std::get<4>(accumulators) += tmp;

        // Contribution (6)
        tmp = op_wire_2 * complement_ecc_op_by_scaling;
        std::get<5>(accumulators) += tmp;

        // Contribution (7)
        tmp = op_wire_3 * complement_ecc_op_by_scaling;
        std::get<6>(accumulators) += tmp;

        // Contribution (8)
        tmp = op_wire_4 * complement_ecc_op_by_scaling;
        std::get<7>(accumulators) += tmp;
    };
};

template <typename FF> using EccOpQueueRelation = Relation<EccOpQueueRelationImpl<FF>>;

} // namespace bb