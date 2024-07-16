#pragma once
#include "barretenberg/relations/relation_types.hpp"
namespace bb {

template <typename FF_> class Poseidon2ExternalRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 4> SUBRELATION_PARTIAL_LENGTHS{
        7, // external poseidon2 round sub-relation for first value
        7, // external poseidon2 round sub-relation for second value
        7, // external poseidon2 round sub-relation for third value
        7, // external poseidon2 round sub-relation for fourth value
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
        return in.q_poseidon2_external.is_zero();
    }

    /**
     * @brief Expression for the poseidon2 external round relation, based on E_i in Section 6 of
     * https://eprint.iacr.org/2023/323.pdf.
     * @details This relation is defined as C(in(X)...) :=
     * q_poseidon2_external * ( (v1 - w_1_shift) + \alpha * (v2 - w_2_shift) +
     * \alpha^2 * (v3 - w_3_shift) + \alpha^3 * (v4 - w_4_shift) ) = 0 where:
     *      u1 := (w_1 + q_1)^5
     *      u2 := (w_2 + q_2)^5
     *      u3 := (w_3 + q_3)^5
     *      u4 := (w_4 + q_4)^5
     *      t0 := u1 + u2                                           (1, 1, 0, 0)
     *      t1 := u3 + u4                                           (0, 0, 1, 1)
     *      t2 := 2 * u2 + t1 = 2 * u2 + u3 + u4                    (0, 2, 1, 1)
     *      t3 := 2 * u4 + t0 = u1 + u2 + 2 * u4                    (1, 1, 0, 2)
     *      v4 := 4 * t1 + t3 = u1 + u2 + 4 * u3 + 6 * u4           (1, 1, 4, 6)
     *      v2 := 4 * t0 + t2 = 4 * u1 + 6 * u2 + u3 + u4           (4, 6, 1, 1)
     *      v1 := t3 + v2 = 5 * u1 + 7 * u2 + 1 * u3 + 3 * u4       (5, 7, 1, 3)
     *      v3 := t2 + v4                                           (1, 3, 5, 7)
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
        BB_OP_COUNT_TIME_NAME("PoseidonExt::accumulate");
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
        auto q_r = View(in.q_r);
        auto q_o = View(in.q_o);
        auto q_4 = View(in.q_4);
        auto q_poseidon2_external = View(in.q_poseidon2_external);

        // add round constants which are loaded in selectors
        auto s1 = w_l + q_l;
        auto s2 = w_r + q_r;
        auto s3 = w_o + q_o;
        auto s4 = w_4 + q_4;

        // apply s-box round
        auto u1 = s1.sqr();
        u1 = u1.sqr();
        u1 *= s1;
        auto u2 = s2.sqr();
        u2 = u2.sqr();
        u2 *= s2;
        auto u3 = s3.sqr();
        u3 = u3.sqr();
        u3 *= s3;
        auto u4 = s4.sqr();
        u4 = u4.sqr();
        u4 *= s4;

        // matrix mul v = M_E * u with 14 additions
        auto t0 = u1 + u2; // u_1 + u_2
        auto t1 = u3 + u4; // u_3 + u_4
        auto t2 = u2 + u2; // 2u_2
        t2 += t1;          // 2u_2 + u_3 + u_4
        auto t3 = u4 + u4; // 2u_4
        t3 += t0;          // u_1 + u_2 + 2u_4
        auto v4 = t1 + t1;
        v4 += v4;
        v4 += t3; // u_1 + u_2 + 4u_3 + 6u_4
        auto v2 = t0 + t0;
        v2 += v2;
        v2 += t2;          // 4u_1 + 6u_2 + u_3 + u_4
        auto v1 = t3 + v2; // 5u_1 + 7u_2 + u_3 + 3u_4
        auto v3 = t2 + v4; // u_1 + 3u_2 + 5u_3 + 7u_4

        auto q_pos_by_scaling = q_poseidon2_external * scaling_factor;
        auto tmp = q_pos_by_scaling * (v1 - w_l_shift);
        std::get<0>(evals) += tmp;

        tmp = q_pos_by_scaling * (v2 - w_r_shift);
        std::get<1>(evals) += tmp;

        tmp = q_pos_by_scaling * (v3 - w_o_shift);
        std::get<2>(evals) += tmp;

        tmp = q_pos_by_scaling * (v4 - w_4_shift);
        std::get<3>(evals) += tmp;
    };
};

template <typename FF> using Poseidon2ExternalRelation = Relation<Poseidon2ExternalRelationImpl<FF>>;
} // namespace bb