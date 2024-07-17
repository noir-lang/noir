#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct AluRow {
    FF alu_a_hi{};
    FF alu_a_hi_shift{};
    FF alu_a_lo{};
    FF alu_a_lo_shift{};
    FF alu_b_hi{};
    FF alu_b_hi_shift{};
    FF alu_b_lo{};
    FF alu_b_lo_shift{};
    FF alu_borrow{};
    FF alu_cf{};
    FF alu_cmp_rng_ctr{};
    FF alu_cmp_rng_ctr_shift{};
    FF alu_div_u16_r0{};
    FF alu_div_u16_r0_shift{};
    FF alu_div_u16_r1{};
    FF alu_div_u16_r1_shift{};
    FF alu_div_u16_r2{};
    FF alu_div_u16_r2_shift{};
    FF alu_div_u16_r3{};
    FF alu_div_u16_r3_shift{};
    FF alu_div_u16_r4{};
    FF alu_div_u16_r4_shift{};
    FF alu_div_u16_r5{};
    FF alu_div_u16_r5_shift{};
    FF alu_div_u16_r6{};
    FF alu_div_u16_r6_shift{};
    FF alu_div_u16_r7{};
    FF alu_div_u16_r7_shift{};
    FF alu_divisor_hi{};
    FF alu_divisor_lo{};
    FF alu_ff_tag{};
    FF alu_ia{};
    FF alu_ib{};
    FF alu_ic{};
    FF alu_in_tag{};
    FF alu_op_add{};
    FF alu_op_add_shift{};
    FF alu_op_cast{};
    FF alu_op_cast_prev{};
    FF alu_op_cast_prev_shift{};
    FF alu_op_cast_shift{};
    FF alu_op_div{};
    FF alu_op_div_a_lt_b{};
    FF alu_op_div_shift{};
    FF alu_op_div_std{};
    FF alu_op_eq{};
    FF alu_op_eq_diff_inv{};
    FF alu_op_lt{};
    FF alu_op_lte{};
    FF alu_op_mul{};
    FF alu_op_mul_shift{};
    FF alu_op_not{};
    FF alu_op_shl{};
    FF alu_op_shl_shift{};
    FF alu_op_shr{};
    FF alu_op_shr_shift{};
    FF alu_op_sub{};
    FF alu_op_sub_shift{};
    FF alu_p_a_borrow{};
    FF alu_p_b_borrow{};
    FF alu_p_sub_a_hi{};
    FF alu_p_sub_a_hi_shift{};
    FF alu_p_sub_a_lo{};
    FF alu_p_sub_a_lo_shift{};
    FF alu_p_sub_b_hi{};
    FF alu_p_sub_b_hi_shift{};
    FF alu_p_sub_b_lo{};
    FF alu_p_sub_b_lo_shift{};
    FF alu_partial_prod_hi{};
    FF alu_partial_prod_lo{};
    FF alu_quotient_hi{};
    FF alu_quotient_lo{};
    FF alu_remainder{};
    FF alu_res_hi{};
    FF alu_res_lo{};
    FF alu_sel_alu{};
    FF alu_sel_alu_shift{};
    FF alu_sel_cmp{};
    FF alu_sel_cmp_shift{};
    FF alu_sel_div_rng_chk{};
    FF alu_sel_div_rng_chk_shift{};
    FF alu_sel_rng_chk{};
    FF alu_sel_rng_chk_lookup_shift{};
    FF alu_sel_rng_chk_shift{};
    FF alu_sel_shift_which{};
    FF alu_shift_lt_bit_len{};
    FF alu_t_sub_s_bits{};
    FF alu_two_pow_s{};
    FF alu_two_pow_t_sub_s{};
    FF alu_u128_tag{};
    FF alu_u16_r0{};
    FF alu_u16_r0_shift{};
    FF alu_u16_r1{};
    FF alu_u16_r10{};
    FF alu_u16_r11{};
    FF alu_u16_r12{};
    FF alu_u16_r13{};
    FF alu_u16_r14{};
    FF alu_u16_r1_shift{};
    FF alu_u16_r2{};
    FF alu_u16_r2_shift{};
    FF alu_u16_r3{};
    FF alu_u16_r3_shift{};
    FF alu_u16_r4{};
    FF alu_u16_r4_shift{};
    FF alu_u16_r5{};
    FF alu_u16_r5_shift{};
    FF alu_u16_r6{};
    FF alu_u16_r6_shift{};
    FF alu_u16_r7{};
    FF alu_u16_r8{};
    FF alu_u16_r9{};
    FF alu_u16_tag{};
    FF alu_u32_tag{};
    FF alu_u64_tag{};
    FF alu_u8_r0{};
    FF alu_u8_r0_shift{};
    FF alu_u8_r1{};
    FF alu_u8_r1_shift{};
    FF alu_u8_tag{};
};

template <typename FF_> class aluImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 87> SUBRELATION_PARTIAL_LENGTHS = {
        2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 5, 5, 5, 5, 6, 6, 8, 3, 4, 4, 5, 4, 4, 3, 4, 3,
        3, 4, 3, 6, 5, 3, 3, 3, 3, 4, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 2, 5, 3, 3, 4, 4, 4, 4,
        4, 3, 5, 5, 4, 5, 5, 2, 3, 3, 3, 3, 3, 4, 4, 3, 5, 3, 3, 3, 5, 3, 3, 4, 4, 4, 4, 4, 4
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        {
            using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_sel_alu -
                 ((((((((((new_term.alu_op_add + new_term.alu_op_sub) + new_term.alu_op_mul) + new_term.alu_op_not) +
                        new_term.alu_op_eq) +
                       new_term.alu_op_cast) +
                      new_term.alu_op_lt) +
                     new_term.alu_op_lte) +
                    new_term.alu_op_shr) +
                   new_term.alu_op_shl) +
                  new_term.alu_op_div));
            tmp *= scaling_factor;
            std::get<0>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<1, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_sel_cmp - (new_term.alu_op_lt + new_term.alu_op_lte));
            tmp *= scaling_factor;
            std::get<1>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<2, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_sel_shift_which - (new_term.alu_op_shl + new_term.alu_op_shr));
            tmp *= scaling_factor;
            std::get<2>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<3, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_cf * (-new_term.alu_cf + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<4, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_ff_tag * (-new_term.alu_ff_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<5, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_u8_tag * (-new_term.alu_u8_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<6, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_u16_tag * (-new_term.alu_u16_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<7, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_u32_tag * (-new_term.alu_u32_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<8, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_u64_tag * (-new_term.alu_u64_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<9, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_u128_tag * (-new_term.alu_u128_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<10, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_sel_alu *
                 ((((((new_term.alu_ff_tag + new_term.alu_u8_tag) + new_term.alu_u16_tag) + new_term.alu_u32_tag) +
                    new_term.alu_u64_tag) +
                   new_term.alu_u128_tag) -
                  FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<11, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_in_tag -
                        (((((new_term.alu_u8_tag + (new_term.alu_u16_tag * FF(2))) + (new_term.alu_u32_tag * FF(3))) +
                           (new_term.alu_u64_tag * FF(4))) +
                          (new_term.alu_u128_tag * FF(5))) +
                         (new_term.alu_ff_tag * FF(6))));
            tmp *= scaling_factor;
            std::get<11>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<12, ContainerOverSubrelations>;
            auto tmp =
                (((new_term.alu_op_add + new_term.alu_op_sub) *
                  ((((((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                          (new_term.alu_u16_r1 * FF(4294967296UL))) +
                         (new_term.alu_u16_r2 * FF(281474976710656UL))) +
                        (new_term.alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                       (new_term.alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                      (new_term.alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                     (new_term.alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) -
                    new_term.alu_ia) +
                   (new_term.alu_ff_tag * new_term.alu_ic))) +
                 ((new_term.alu_op_add - new_term.alu_op_sub) *
                  ((new_term.alu_cf * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })) - new_term.alu_ib)));
            tmp *= scaling_factor;
            std::get<12>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<13, ContainerOverSubrelations>;
            auto tmp =
                (((new_term.alu_op_add + new_term.alu_op_sub) *
                  (((((((new_term.alu_u8_tag * new_term.alu_u8_r0) +
                        (new_term.alu_u16_tag * (new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))))) +
                       (new_term.alu_u32_tag *
                        ((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))))) +
                      (new_term.alu_u64_tag *
                       ((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                         (new_term.alu_u16_r1 * FF(4294967296UL))) +
                        (new_term.alu_u16_r2 * FF(281474976710656UL))))) +
                     (new_term.alu_u128_tag *
                      ((((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) +
                             (new_term.alu_u16_r0 * FF(65536))) +
                            (new_term.alu_u16_r1 * FF(4294967296UL))) +
                           (new_term.alu_u16_r2 * FF(281474976710656UL))) +
                          (new_term.alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                         (new_term.alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                        (new_term.alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                       (new_term.alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))))) +
                    (new_term.alu_ff_tag * new_term.alu_ia)) -
                   new_term.alu_ic)) +
                 ((new_term.alu_ff_tag * (new_term.alu_op_add - new_term.alu_op_sub)) * new_term.alu_ib));
            tmp *= scaling_factor;
            std::get<13>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<14, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_ff_tag * new_term.alu_op_mul) * ((new_term.alu_ia * new_term.alu_ib) - new_term.alu_ic));
            tmp *= scaling_factor;
            std::get<14>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<15, ContainerOverSubrelations>;
            auto tmp =
                ((((-new_term.alu_ff_tag + FF(1)) - new_term.alu_u128_tag) * new_term.alu_op_mul) *
                 (((((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                        (new_term.alu_u16_r1 * FF(4294967296UL))) +
                       (new_term.alu_u16_r2 * FF(281474976710656UL))) +
                      (new_term.alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                     (new_term.alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                    (new_term.alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                   (new_term.alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) -
                  (new_term.alu_ia * new_term.alu_ib)));
            tmp *= scaling_factor;
            std::get<15>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<16, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_op_mul *
                 (((((new_term.alu_u8_tag * new_term.alu_u8_r0) +
                     (new_term.alu_u16_tag * (new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))))) +
                    (new_term.alu_u32_tag *
                     ((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))))) +
                   (new_term.alu_u64_tag *
                    ((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                      (new_term.alu_u16_r1 * FF(4294967296UL))) +
                     (new_term.alu_u16_r2 * FF(281474976710656UL))))) -
                  (((-new_term.alu_ff_tag + FF(1)) - new_term.alu_u128_tag) * new_term.alu_ic)));
            tmp *= scaling_factor;
            std::get<16>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<17, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_u128_tag * new_term.alu_op_mul) *
                 ((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                     (new_term.alu_u16_r1 * FF(4294967296UL))) +
                    (new_term.alu_u16_r2 * FF(281474976710656UL))) +
                   ((((new_term.alu_u16_r3 + (new_term.alu_u16_r4 * FF(65536))) +
                      (new_term.alu_u16_r5 * FF(4294967296UL))) +
                     (new_term.alu_u16_r6 * FF(281474976710656UL))) *
                    FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                  new_term.alu_ia));
            tmp *= scaling_factor;
            std::get<17>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<18, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_u128_tag * new_term.alu_op_mul) *
                        ((((((new_term.alu_u8_r0_shift + (new_term.alu_u8_r1_shift * FF(256))) +
                             (new_term.alu_u16_r0_shift * FF(65536))) +
                            (new_term.alu_u16_r1_shift * FF(4294967296UL))) +
                           (new_term.alu_u16_r2_shift * FF(281474976710656UL))) +
                          ((((new_term.alu_u16_r3_shift + (new_term.alu_u16_r4_shift * FF(65536))) +
                             (new_term.alu_u16_r5_shift * FF(4294967296UL))) +
                            (new_term.alu_u16_r6_shift * FF(281474976710656UL))) *
                           FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                         new_term.alu_ib));
            tmp *= scaling_factor;
            std::get<18>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<19, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_u128_tag * new_term.alu_op_mul) *
                 ((((new_term.alu_ia * ((((new_term.alu_u8_r0_shift + (new_term.alu_u8_r1_shift * FF(256))) +
                                          (new_term.alu_u16_r0_shift * FF(65536))) +
                                         (new_term.alu_u16_r1_shift * FF(4294967296UL))) +
                                        (new_term.alu_u16_r2_shift * FF(281474976710656UL)))) +
                    ((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                        (new_term.alu_u16_r1 * FF(4294967296UL))) +
                       (new_term.alu_u16_r2 * FF(281474976710656UL))) *
                      (((new_term.alu_u16_r3_shift + (new_term.alu_u16_r4_shift * FF(65536))) +
                        (new_term.alu_u16_r5_shift * FF(4294967296UL))) +
                       (new_term.alu_u16_r6_shift * FF(281474976710656UL)))) *
                     FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                   (((new_term.alu_cf * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL })) +
                     (((new_term.alu_u16_r7 + (new_term.alu_u16_r8 * FF(65536))) +
                       (new_term.alu_u16_r9 * FF(4294967296UL))) +
                      (new_term.alu_u16_r10 * FF(281474976710656UL)))) *
                    FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                  new_term.alu_ic));
            tmp *= scaling_factor;
            std::get<19>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<20, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_not * new_term.alu_ff_tag);
            tmp *= scaling_factor;
            std::get<20>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<21, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_op_not * ((new_term.alu_ia + new_term.alu_ic) -
                                        ((((((new_term.alu_u8_tag * FF(256)) + (new_term.alu_u16_tag * FF(65536))) +
                                            (new_term.alu_u32_tag * FF(4294967296UL))) +
                                           (new_term.alu_u64_tag * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                          (new_term.alu_u128_tag * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                                         FF(1))));
            tmp *= scaling_factor;
            std::get<21>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<22, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_sel_cmp + new_term.alu_op_eq) * (new_term.alu_ic * (-new_term.alu_ic + FF(1))));
            tmp *= scaling_factor;
            std::get<22>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<23, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_eq *
                        ((((new_term.alu_ia - new_term.alu_ib) *
                           ((new_term.alu_ic * (-new_term.alu_op_eq_diff_inv + FF(1))) + new_term.alu_op_eq_diff_inv)) -
                          FF(1)) +
                         new_term.alu_ic));
            tmp *= scaling_factor;
            std::get<23>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<24, ContainerOverSubrelations>;
            auto tmp = (((new_term.alu_op_lt * new_term.alu_ib) +
                         ((new_term.alu_op_lte + new_term.alu_op_cast) * new_term.alu_ia)) -
                        ((new_term.alu_a_lo + (new_term.alu_a_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                         (new_term.alu_sel_cmp + new_term.alu_op_cast)));
            tmp *= scaling_factor;
            std::get<24>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<25, ContainerOverSubrelations>;
            auto tmp = (((new_term.alu_op_lt * new_term.alu_ia) + (new_term.alu_op_lte * new_term.alu_ib)) -
                        ((new_term.alu_b_lo + (new_term.alu_b_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                         new_term.alu_sel_cmp));
            tmp *= scaling_factor;
            std::get<25>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<26, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_p_a_borrow * (-new_term.alu_p_a_borrow + FF(1)));
            tmp *= scaling_factor;
            std::get<26>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<27, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_p_sub_a_lo -
                  ((-new_term.alu_a_lo + FF(uint256_t{ 4891460686036598784UL, 2896914383306846353UL, 0UL, 0UL })) +
                   (new_term.alu_p_a_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))) *
                 ((new_term.alu_sel_cmp + new_term.alu_op_cast) + new_term.alu_op_div_std));
            tmp *= scaling_factor;
            std::get<27>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<28, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_p_sub_a_hi -
                  ((-new_term.alu_a_hi + FF(uint256_t{ 13281191951274694749UL, 3486998266802970665UL, 0UL, 0UL })) -
                   new_term.alu_p_a_borrow)) *
                 ((new_term.alu_sel_cmp + new_term.alu_op_cast) + new_term.alu_op_div_std));
            tmp *= scaling_factor;
            std::get<28>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<29, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_p_b_borrow * (-new_term.alu_p_b_borrow + FF(1)));
            tmp *= scaling_factor;
            std::get<29>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<30, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_p_sub_b_lo -
                  ((-new_term.alu_b_lo + FF(uint256_t{ 4891460686036598784UL, 2896914383306846353UL, 0UL, 0UL })) +
                   (new_term.alu_p_b_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))) *
                 new_term.alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<30>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<31, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_p_sub_b_hi -
                  ((-new_term.alu_b_hi + FF(uint256_t{ 13281191951274694749UL, 3486998266802970665UL, 0UL, 0UL })) -
                   new_term.alu_p_b_borrow)) *
                 new_term.alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<31>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<32, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_res_lo -
                  (((((new_term.alu_a_lo - new_term.alu_b_lo) - FF(1)) +
                     (new_term.alu_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                    ((new_term.alu_op_lt * new_term.alu_ic) + ((-new_term.alu_ic + FF(1)) * new_term.alu_op_lte))) +
                   (((new_term.alu_b_lo - new_term.alu_a_lo) +
                     (new_term.alu_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                    (-((new_term.alu_op_lt * new_term.alu_ic) + ((-new_term.alu_ic + FF(1)) * new_term.alu_op_lte)) +
                     FF(1))))) *
                 new_term.alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<32>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<33, ContainerOverSubrelations>;
            auto tmp =
                ((new_term.alu_res_hi -
                  ((((new_term.alu_a_hi - new_term.alu_b_hi) - new_term.alu_borrow) *
                    ((new_term.alu_op_lt * new_term.alu_ic) + ((-new_term.alu_ic + FF(1)) * new_term.alu_op_lte))) +
                   (((new_term.alu_b_hi - new_term.alu_a_hi) - new_term.alu_borrow) *
                    (-((new_term.alu_op_lt * new_term.alu_ic) + ((-new_term.alu_ic + FF(1)) * new_term.alu_op_lte)) +
                     FF(1))))) *
                 new_term.alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<33>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<34, ContainerOverSubrelations>;
            auto tmp =
                (((new_term.alu_cmp_rng_ctr_shift - new_term.alu_cmp_rng_ctr) + FF(1)) * new_term.alu_cmp_rng_ctr);
            tmp *= scaling_factor;
            std::get<34>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<35, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_cmp_rng_ctr_shift - FF(4)) * new_term.alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<35>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<36, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_sel_rng_chk * (-new_term.alu_sel_rng_chk + FF(1)));
            tmp *= scaling_factor;
            std::get<36>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<37, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_sel_rng_chk * new_term.alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<37>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<38, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_cmp_rng_ctr *
                         (((-new_term.alu_sel_rng_chk + FF(1)) * (-new_term.alu_op_eq_diff_inv + FF(1))) +
                          new_term.alu_op_eq_diff_inv)) -
                        new_term.alu_sel_rng_chk);
            tmp *= scaling_factor;
            std::get<38>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<39, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_sel_rng_chk_lookup_shift -
                 ((((((((((new_term.alu_sel_cmp_shift + new_term.alu_sel_rng_chk_shift) + new_term.alu_op_add_shift) +
                         new_term.alu_op_sub_shift) +
                        new_term.alu_op_mul_shift) +
                       (new_term.alu_op_mul * new_term.alu_u128_tag)) +
                      new_term.alu_op_cast_shift) +
                     new_term.alu_op_cast_prev_shift) +
                    new_term.alu_op_shl_shift) +
                   new_term.alu_op_shr_shift) +
                  new_term.alu_op_div_shift));
            tmp *= scaling_factor;
            std::get<39>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<40, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_a_lo -
                 (((((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                        (new_term.alu_u16_r1 * FF(4294967296UL))) +
                       (new_term.alu_u16_r2 * FF(281474976710656UL))) +
                      (new_term.alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                     (new_term.alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                    (new_term.alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                   (new_term.alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) *
                  (((((new_term.alu_sel_rng_chk + new_term.alu_sel_cmp) + new_term.alu_op_cast) +
                     new_term.alu_op_cast_prev) +
                    new_term.alu_shift_lt_bit_len) +
                   new_term.alu_op_div)));
            tmp *= scaling_factor;
            std::get<40>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<41, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_a_hi - ((((((((new_term.alu_u16_r7 + (new_term.alu_u16_r8 * FF(65536))) +
                                            (new_term.alu_u16_r9 * FF(4294967296UL))) +
                                           (new_term.alu_u16_r10 * FF(281474976710656UL))) +
                                          (new_term.alu_u16_r11 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                         (new_term.alu_u16_r12 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                                        (new_term.alu_u16_r13 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                                       (new_term.alu_u16_r14 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) *
                                      (((((new_term.alu_sel_rng_chk + new_term.alu_sel_cmp) + new_term.alu_op_cast) +
                                         new_term.alu_op_cast_prev) +
                                        new_term.alu_shift_lt_bit_len) +
                                       new_term.alu_op_div)));
            tmp *= scaling_factor;
            std::get<41>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<42, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_a_lo_shift - new_term.alu_b_lo) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<42>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<43, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_a_hi_shift - new_term.alu_b_hi) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<43>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<44, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_b_lo_shift - new_term.alu_p_sub_a_lo) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<44>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<45, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_b_hi_shift - new_term.alu_p_sub_a_hi) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<45>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<46, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_p_sub_a_lo_shift - new_term.alu_p_sub_b_lo) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<46>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<47, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_p_sub_a_hi_shift - new_term.alu_p_sub_b_hi) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<47>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<48, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_p_sub_b_lo_shift - new_term.alu_res_lo) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<48>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<49, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_p_sub_b_hi_shift - new_term.alu_res_hi) * new_term.alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<49>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<50, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_cast_prev_shift - new_term.alu_op_cast);
            tmp *= scaling_factor;
            std::get<50>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<51, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_op_cast *
                 (((((((new_term.alu_u8_tag * new_term.alu_u8_r0) +
                       (new_term.alu_u16_tag * (new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))))) +
                      (new_term.alu_u32_tag *
                       ((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))))) +
                     (new_term.alu_u64_tag *
                      ((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                        (new_term.alu_u16_r1 * FF(4294967296UL))) +
                       (new_term.alu_u16_r2 * FF(281474976710656UL))))) +
                    (new_term.alu_u128_tag *
                     ((((((((new_term.alu_u8_r0 + (new_term.alu_u8_r1 * FF(256))) + (new_term.alu_u16_r0 * FF(65536))) +
                           (new_term.alu_u16_r1 * FF(4294967296UL))) +
                          (new_term.alu_u16_r2 * FF(281474976710656UL))) +
                         (new_term.alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                        (new_term.alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                       (new_term.alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                      (new_term.alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))))) +
                   (new_term.alu_ff_tag * new_term.alu_ia)) -
                  new_term.alu_ic));
            tmp *= scaling_factor;
            std::get<51>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<52, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_cast * (new_term.alu_a_lo_shift - new_term.alu_p_sub_a_lo));
            tmp *= scaling_factor;
            std::get<52>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<53, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_cast * (new_term.alu_a_hi_shift - new_term.alu_p_sub_a_hi));
            tmp *= scaling_factor;
            std::get<53>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<54, ContainerOverSubrelations>;
            auto tmp =
                (((new_term.alu_op_mul * new_term.alu_u128_tag) + new_term.alu_op_cast) * new_term.alu_sel_alu_shift);
            tmp *= scaling_factor;
            std::get<54>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<55, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_shift_lt_bit_len * new_term.alu_op_shr) *
                        (new_term.alu_a_lo - ((new_term.alu_two_pow_s - new_term.alu_b_lo) - FF(1))));
            tmp *= scaling_factor;
            std::get<55>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<56, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_shift_lt_bit_len * new_term.alu_op_shr) *
                        (new_term.alu_a_hi - ((new_term.alu_two_pow_t_sub_s - new_term.alu_b_hi) - FF(1))));
            tmp *= scaling_factor;
            std::get<56>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<57, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_shift_lt_bit_len * new_term.alu_op_shl) *
                        (new_term.alu_a_lo - ((new_term.alu_two_pow_t_sub_s - new_term.alu_b_lo) - FF(1))));
            tmp *= scaling_factor;
            std::get<57>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<58, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_shift_lt_bit_len * new_term.alu_op_shl) *
                        (new_term.alu_a_hi - ((new_term.alu_two_pow_s - new_term.alu_b_hi) - FF(1))));
            tmp *= scaling_factor;
            std::get<58>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<59, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_shift_lt_bit_len * (-new_term.alu_shift_lt_bit_len + FF(1)));
            tmp *= scaling_factor;
            std::get<59>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<60, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_t_sub_s_bits -
                        (new_term.alu_sel_shift_which *
                         ((new_term.alu_shift_lt_bit_len *
                           ((((((new_term.alu_u8_tag * FF(8)) + (new_term.alu_u16_tag * FF(16))) +
                               (new_term.alu_u32_tag * FF(32))) +
                              (new_term.alu_u64_tag * FF(64))) +
                             (new_term.alu_u128_tag * FF(128))) -
                            new_term.alu_ib)) +
                          ((-new_term.alu_shift_lt_bit_len + FF(1)) *
                           (new_term.alu_ib - (((((new_term.alu_u8_tag * FF(8)) + (new_term.alu_u16_tag * FF(16))) +
                                                 (new_term.alu_u32_tag * FF(32))) +
                                                (new_term.alu_u64_tag * FF(64))) +
                                               (new_term.alu_u128_tag * FF(128))))))));
            tmp *= scaling_factor;
            std::get<60>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<61, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_shift_lt_bit_len * new_term.alu_op_shr) *
                        (((new_term.alu_b_hi * new_term.alu_two_pow_s) + new_term.alu_b_lo) - new_term.alu_ia));
            tmp *= scaling_factor;
            std::get<61>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<62, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_shr * (new_term.alu_ic - (new_term.alu_b_hi * new_term.alu_shift_lt_bit_len)));
            tmp *= scaling_factor;
            std::get<62>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<63, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_shift_lt_bit_len * new_term.alu_op_shl) *
                        (((new_term.alu_b_hi * new_term.alu_two_pow_t_sub_s) + new_term.alu_b_lo) - new_term.alu_ia));
            tmp *= scaling_factor;
            std::get<63>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<64, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_op_shl *
                 (new_term.alu_ic - ((new_term.alu_b_lo * new_term.alu_two_pow_s) * new_term.alu_shift_lt_bit_len)));
            tmp *= scaling_factor;
            std::get<64>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<65, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div - (new_term.alu_op_div_std + new_term.alu_op_div_a_lt_b));
            tmp *= scaling_factor;
            std::get<65>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<66, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_a_lt_b * (-new_term.alu_op_div_a_lt_b + FF(1)));
            tmp *= scaling_factor;
            std::get<66>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<67, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_op_div_a_lt_b * (new_term.alu_a_lo - ((new_term.alu_ib - new_term.alu_ia) - FF(1))));
            tmp *= scaling_factor;
            std::get<67>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<68, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_a_lt_b * new_term.alu_ic);
            tmp *= scaling_factor;
            std::get<68>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<69, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_a_lt_b * (new_term.alu_ia - new_term.alu_remainder));
            tmp *= scaling_factor;
            std::get<69>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<70, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_std * (-new_term.alu_op_div_std + FF(1)));
            tmp *= scaling_factor;
            std::get<70>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<71, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_std * ((new_term.alu_ib - new_term.alu_divisor_lo) -
                                                   (new_term.alu_divisor_hi * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))));
            tmp *= scaling_factor;
            std::get<71>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<72, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_std * ((new_term.alu_ic - new_term.alu_quotient_lo) -
                                                   (new_term.alu_quotient_hi * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))));
            tmp *= scaling_factor;
            std::get<72>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<73, ContainerOverSubrelations>;
            auto tmp =
                (((new_term.alu_divisor_hi * new_term.alu_quotient_lo) +
                  (new_term.alu_divisor_lo * new_term.alu_quotient_hi)) -
                 (new_term.alu_partial_prod_lo + (new_term.alu_partial_prod_hi * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))));
            tmp *= scaling_factor;
            std::get<73>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<74, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_std *
                        ((((new_term.alu_divisor_lo * new_term.alu_quotient_lo) +
                           (new_term.alu_partial_prod_lo * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                          ((new_term.alu_partial_prod_hi + (new_term.alu_divisor_hi * new_term.alu_quotient_hi)) *
                           FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                         (new_term.alu_a_lo + (new_term.alu_a_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))));
            tmp *= scaling_factor;
            std::get<74>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<75, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_op_div_std * (new_term.alu_b_hi - ((new_term.alu_ib - new_term.alu_remainder) - FF(1))));
            tmp *= scaling_factor;
            std::get<75>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<76, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_cmp_rng_ctr_shift - FF(2)) * new_term.alu_op_div_std);
            tmp *= scaling_factor;
            std::get<76>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<77, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_sel_rng_chk * new_term.alu_op_div_std);
            tmp *= scaling_factor;
            std::get<77>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<78, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_op_div_std *
                        ((((new_term.alu_divisor_lo * new_term.alu_quotient_lo) +
                           (new_term.alu_partial_prod_lo * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                          ((new_term.alu_partial_prod_hi + (new_term.alu_divisor_hi * new_term.alu_quotient_hi)) *
                           FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                         (new_term.alu_ia - new_term.alu_remainder)));
            tmp *= scaling_factor;
            std::get<78>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<79, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_sel_div_rng_chk * (-new_term.alu_sel_div_rng_chk + FF(1)));
            tmp *= scaling_factor;
            std::get<79>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<80, ContainerOverSubrelations>;
            auto tmp = ((new_term.alu_sel_div_rng_chk * new_term.alu_sel_div_rng_chk_shift) - new_term.alu_op_div_std);
            tmp *= scaling_factor;
            std::get<80>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<81, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_divisor_lo -
                        (new_term.alu_op_div_std * (((new_term.alu_div_u16_r0 + (new_term.alu_div_u16_r1 * FF(65536))) +
                                                     (new_term.alu_div_u16_r2 * FF(4294967296UL))) +
                                                    (new_term.alu_div_u16_r3 * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<81>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<82, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_divisor_hi -
                        (new_term.alu_op_div_std * (((new_term.alu_div_u16_r4 + (new_term.alu_div_u16_r5 * FF(65536))) +
                                                     (new_term.alu_div_u16_r6 * FF(4294967296UL))) +
                                                    (new_term.alu_div_u16_r7 * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<82>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<83, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_quotient_lo -
                        (new_term.alu_op_div_std *
                         (((new_term.alu_div_u16_r0_shift + (new_term.alu_div_u16_r1_shift * FF(65536))) +
                           (new_term.alu_div_u16_r2_shift * FF(4294967296UL))) +
                          (new_term.alu_div_u16_r3_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<83>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<84, ContainerOverSubrelations>;
            auto tmp = (new_term.alu_quotient_hi -
                        (new_term.alu_op_div_std *
                         (((new_term.alu_div_u16_r4_shift + (new_term.alu_div_u16_r5_shift * FF(65536))) +
                           (new_term.alu_div_u16_r6_shift * FF(4294967296UL))) +
                          (new_term.alu_div_u16_r7_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<84>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<85, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_partial_prod_lo -
                 (new_term.alu_op_div_std * ((((new_term.alu_u8_r0_shift + (new_term.alu_u8_r1_shift * FF(256))) +
                                               (new_term.alu_u16_r0_shift * FF(65536))) +
                                              (new_term.alu_u16_r1_shift * FF(4294967296UL))) +
                                             (new_term.alu_u16_r2_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<85>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<86, ContainerOverSubrelations>;
            auto tmp =
                (new_term.alu_partial_prod_hi -
                 (new_term.alu_op_div_std * (((new_term.alu_u16_r3_shift + (new_term.alu_u16_r4_shift * FF(65536))) +
                                              (new_term.alu_u16_r5_shift * FF(4294967296UL))) +
                                             (new_term.alu_u16_r6_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<86>(evals) += typename Accumulator::View(tmp);
        }
    }
};

template <typename FF> class alu : public Relation<aluImpl<FF>> {
  public:
    static constexpr const char* NAME = "alu";

    static std::string get_subrelation_label(size_t index)
    {
        switch (index) {
        case 12:
            return "ALU_ADD_SUB_1";
        case 13:
            return "ALU_ADD_SUB_2";
        case 14:
            return "ALU_MULTIPLICATION_FF";
        case 15:
            return "ALU_MUL_COMMON_1";
        case 16:
            return "ALU_MUL_COMMON_2";
        case 19:
            return "ALU_MULTIPLICATION_OUT_U128";
        case 20:
            return "ALU_FF_NOT_XOR";
        case 21:
            return "ALU_OP_NOT";
        case 22:
            return "ALU_RES_IS_BOOL";
        case 23:
            return "ALU_OP_EQ";
        case 24:
            return "INPUT_DECOMP_1";
        case 25:
            return "INPUT_DECOMP_2";
        case 27:
            return "SUB_LO_1";
        case 28:
            return "SUB_HI_1";
        case 30:
            return "SUB_LO_2";
        case 31:
            return "SUB_HI_2";
        case 32:
            return "RES_LO";
        case 33:
            return "RES_HI";
        case 34:
            return "CMP_CTR_REL_1";
        case 35:
            return "CMP_CTR_REL_2";
        case 38:
            return "CTR_NON_ZERO_REL";
        case 39:
            return "RNG_CHK_LOOKUP_SELECTOR";
        case 40:
            return "LOWER_CMP_RNG_CHK";
        case 41:
            return "UPPER_CMP_RNG_CHK";
        case 42:
            return "SHIFT_RELS_0";
        case 44:
            return "SHIFT_RELS_1";
        case 46:
            return "SHIFT_RELS_2";
        case 48:
            return "SHIFT_RELS_3";
        case 50:
            return "OP_CAST_PREV_LINE";
        case 51:
            return "ALU_OP_CAST";
        case 52:
            return "OP_CAST_RNG_CHECK_P_SUB_A_LOW";
        case 53:
            return "OP_CAST_RNG_CHECK_P_SUB_A_HIGH";
        case 54:
            return "TWO_LINE_OP_NO_OVERLAP";
        case 55:
            return "SHR_RANGE_0";
        case 56:
            return "SHR_RANGE_1";
        case 57:
            return "SHL_RANGE_0";
        case 58:
            return "SHL_RANGE_1";
        case 60:
            return "SHIFT_LT_BIT_LEN";
        case 61:
            return "SHR_INPUT_DECOMPOSITION";
        case 62:
            return "SHR_OUTPUT";
        case 63:
            return "SHL_INPUT_DECOMPOSITION";
        case 64:
            return "SHL_OUTPUT";
        case 74:
            return "ALU_PROD_DIV";
        case 75:
            return "REMAINDER_RANGE_CHK";
        case 76:
            return "CMP_CTR_REL_3";
        case 78:
            return "DIVISION_RELATION";
        }
        return std::to_string(index);
    }
};

} // namespace bb::Avm_vm