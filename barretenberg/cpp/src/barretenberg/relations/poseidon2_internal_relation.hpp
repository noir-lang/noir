#pragma once
#include "barretenberg/crypto/poseidon2/poseidon2_params.hpp"
#include "relation_types.hpp"

namespace bb {

template <typename FF_> class Poseidon2InternalRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 4> SUBRELATION_PARTIAL_LENGTHS{
        7, // internal poseidon2 round sub-relation for first value
        7, // internal poseidon2 round sub-relation for second value
        7, // internal poseidon2 round sub-relation for third value
        7, // internal poseidon2 round sub-relation for fourth value
    };
    /**
     * @brief For ZK-Flavors: The degrees of subrelations considered as polynomials only in witness polynomials,
     * i.e. all selectors and public polynomials are treated as constants.
     *
     */
    static constexpr std::array<size_t, 4> SUBRELATION_WITNESS_DEGREES{
        5, // external poseidon2 round sub-relation for first value
        5, // external poseidon2 round sub-relation for second value
        5, // external poseidon2 round sub-relation for third value
        5, // external poseidon2 round sub-relation for fourth value
    };

    /**
     * @brief Returns true if the contribution from all subrelations for the provided inputs is identically zero
     *
     */
    template <typename AllEntities> inline static bool skip(const AllEntities& in)
    {
        return in.q_poseidon2_internal.is_zero();
    }

    /**
     * @brief Expression for the poseidon2 internal round relation, based on I_i in Section 6 of
     * https://eprint.iacr.org/2023/323.pdf.
     * @details This relation is defined as C(in(X)...) :=
     * q_poseidon2_internal * ( (v1 - w_1_shift) + \alpha * (v2 - w_2_shift) +
     * \alpha^2 * (v3 - w_3_shift) + \alpha^3 * (v4 - w_4_shift) ) = 0 where:
     *      u1 := (w_1 + q_1)^5
     *      sum := u1 + w_2 + w_3 + w_4
     *      v1 := u1 * D1 + sum
     *      v2 := w_2 * D2 + sum
     *      v3 := w_3 * D3 + sum
     *      v4 := w_4 * D4 + sum
     *      Di is the ith internal diagonal value - 1 of the internal matrix M_I
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& in,
                           const Parameters&,
                           const FF& scaling_factor)
    {
        BB_OP_COUNT_TIME_NAME("PoseidonInt::accumulate");
        using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
        using View = typename Accumulator::View;
        auto w_l = View(in.w_l);
        auto w_r = View(in.w_r);
        auto w_o = View(in.w_o);
        auto w_4 = View(in.w_4);
        auto w_l_shift = View(in.w_l_shift);
        auto w_r_shift = View(in.w_r_shift);
        auto w_o_shift = View(in.w_o_shift);
        auto w_4_shift = View(in.w_4_shift);
        auto q_l = View(in.q_l);
        auto q_poseidon2_internal = View(in.q_poseidon2_internal);

        // add round constants
        auto s1 = w_l + q_l;

        // apply s-box round
        auto u1 = s1.sqr();
        u1 = u1.sqr();
        u1 *= s1;
        auto u2 = w_r;
        auto u3 = w_o;
        auto u4 = w_4;

        // matrix mul with v = M_I * u 4 muls and 7 additions
        auto sum = u1 + u2 + u3 + u4;

        auto q_pos_by_scaling = q_poseidon2_internal * scaling_factor;

        auto v1 = u1 * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[0];
        v1 += sum;
        auto tmp = q_pos_by_scaling * (v1 - w_l_shift);
        std::get<0>(evals) += tmp;

        auto v2 = u2 * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[1];
        v2 += sum;
        tmp = q_pos_by_scaling * (v2 - w_r_shift);
        std::get<1>(evals) += tmp;

        auto v3 = u3 * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[2];
        v3 += sum;
        tmp = q_pos_by_scaling * (v3 - w_o_shift);
        std::get<2>(evals) += tmp;

        auto v4 = u4 * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[3];
        v4 += sum;
        tmp = q_pos_by_scaling * (v4 - w_4_shift);
        std::get<3>(evals) += tmp;
    };
}; // namespace bb

template <typename FF> using Poseidon2InternalRelation = Relation<Poseidon2InternalRelationImpl<FF>>;
} // namespace bb