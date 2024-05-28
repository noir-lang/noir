
#pragma once
#include <array>
#include <tuple>

#include "./ecc_bools_relation.hpp"

namespace bb {

/**
 * @brief ECCVMBoolsRelationImpl evaluates the correctness of ECCVM boolean checks
 *
 * @details There are a lot of columns in ECCVM that are boolean. As these are all low-degree we place them in a
 * separate relation class
 * @tparam FF
 * @tparam ContainerOverSubrelations
 * @tparam AllEntities
 * @tparam Parameters
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMBoolsRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                            const AllEntities& in,
                                            const Parameters& /*unused*/,
                                            const FF& scaling_factor)
{
    using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    auto z1_zero = View(in.transcript_z1zero);
    auto z2_zero = View(in.transcript_z2zero);
    auto msm_count_zero_at_transition = View(in.transcript_msm_count_zero_at_transition);
    auto q_add = View(in.transcript_add);
    auto q_mul = View(in.transcript_mul);
    auto q_eq = View(in.transcript_eq);
    auto transcript_msm_transition = View(in.transcript_msm_transition);
    auto is_accumulator_empty = View(in.transcript_accumulator_empty);
    auto q_reset_accumulator = View(in.transcript_reset_accumulator);
    auto transcript_Pinfinity = View(in.transcript_base_infinity);
    auto transcript_msm_infinity = View(in.transcript_msm_infinity);
    auto transcript_add_x_equal = View(in.transcript_add_x_equal);
    auto transcript_add_y_equal = View(in.transcript_add_y_equal);
    auto precompute_point_transition = View(in.precompute_point_transition);
    auto msm_transition = View(in.msm_transition);
    auto msm_add = View(in.msm_add);
    auto msm_double = View(in.msm_double);
    auto msm_skew = View(in.msm_skew);
    auto precompute_select = View(in.precompute_select);

    std::get<0>(accumulator) += q_eq * (q_eq - 1) * scaling_factor;
    std::get<1>(accumulator) += q_add * (q_add - 1) * scaling_factor;
    std::get<2>(accumulator) += q_mul * (q_mul - 1) * scaling_factor;
    std::get<3>(accumulator) += q_reset_accumulator * (q_reset_accumulator - 1) * scaling_factor;
    std::get<4>(accumulator) += transcript_msm_transition * (transcript_msm_transition - 1) * scaling_factor;
    std::get<5>(accumulator) += is_accumulator_empty * (is_accumulator_empty - 1) * scaling_factor;
    std::get<6>(accumulator) += z1_zero * (z1_zero - 1) * scaling_factor;
    std::get<7>(accumulator) += z2_zero * (z2_zero - 1) * scaling_factor;
    std::get<8>(accumulator) += transcript_add_x_equal * (transcript_add_x_equal - 1) * scaling_factor;
    std::get<9>(accumulator) += transcript_add_y_equal * (transcript_add_y_equal - 1) * scaling_factor;
    std::get<10>(accumulator) += transcript_Pinfinity * (transcript_Pinfinity - 1) * scaling_factor;
    std::get<11>(accumulator) += transcript_msm_infinity * (transcript_msm_infinity - 1) * scaling_factor;
    std::get<12>(accumulator) += msm_count_zero_at_transition * (msm_count_zero_at_transition - 1) * scaling_factor;
    std::get<13>(accumulator) += msm_transition * (msm_transition - 1) * scaling_factor;
    std::get<14>(accumulator) += precompute_point_transition * (precompute_point_transition - 1) * scaling_factor;
    std::get<15>(accumulator) += msm_add * (msm_add - 1) * scaling_factor;
    std::get<16>(accumulator) += msm_double * (msm_double - 1) * scaling_factor;
    std::get<17>(accumulator) += msm_skew * (msm_skew - 1) * scaling_factor;
    std::get<18>(accumulator) += precompute_select * (precompute_select - 1) * scaling_factor;
}
} // namespace bb
