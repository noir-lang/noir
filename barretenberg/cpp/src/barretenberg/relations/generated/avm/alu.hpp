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

inline std::string get_relation_label_alu(int index)
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
        // Contribution 0
        {
            Avm_DECLARE_VIEWS(0);
            auto tmp = (alu_sel_alu -
                        ((((((((((alu_op_add + alu_op_sub) + alu_op_mul) + alu_op_not) + alu_op_eq) + alu_op_cast) +
                             alu_op_lt) +
                            alu_op_lte) +
                           alu_op_shr) +
                          alu_op_shl) +
                         alu_op_div));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);
            auto tmp = (alu_sel_cmp - (alu_op_lt + alu_op_lte));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);
            auto tmp = (alu_sel_shift_which - (alu_op_shl + alu_op_shr));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);
            auto tmp = (alu_cf * (-alu_cf + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);
            auto tmp = (alu_ff_tag * (-alu_ff_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);
            auto tmp = (alu_u8_tag * (-alu_u8_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);
            auto tmp = (alu_u16_tag * (-alu_u16_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);
            auto tmp = (alu_u32_tag * (-alu_u32_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);
            auto tmp = (alu_u64_tag * (-alu_u64_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);
            auto tmp = (alu_u128_tag * (-alu_u128_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);
            auto tmp =
                (alu_sel_alu *
                 ((((((alu_ff_tag + alu_u8_tag) + alu_u16_tag) + alu_u32_tag) + alu_u64_tag) + alu_u128_tag) - FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            Avm_DECLARE_VIEWS(11);
            auto tmp = (alu_in_tag -
                        (((((alu_u8_tag + (alu_u16_tag * FF(2))) + (alu_u32_tag * FF(3))) + (alu_u64_tag * FF(4))) +
                          (alu_u128_tag * FF(5))) +
                         (alu_ff_tag * FF(6))));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            Avm_DECLARE_VIEWS(12);
            auto tmp =
                (((alu_op_add + alu_op_sub) * ((((((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                                                      (alu_u16_r1 * FF(4294967296UL))) +
                                                     (alu_u16_r2 * FF(281474976710656UL))) +
                                                    (alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                                   (alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                                                  (alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                                                 (alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) -
                                                alu_ia) +
                                               (alu_ff_tag * alu_ic))) +
                 ((alu_op_add - alu_op_sub) * ((alu_cf * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })) - alu_ib)));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            Avm_DECLARE_VIEWS(13);
            auto tmp = (((alu_op_add + alu_op_sub) *
                         (((((((alu_u8_tag * alu_u8_r0) + (alu_u16_tag * (alu_u8_r0 + (alu_u8_r1 * FF(256))))) +
                              (alu_u32_tag * ((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))))) +
                             (alu_u64_tag * ((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                                              (alu_u16_r1 * FF(4294967296UL))) +
                                             (alu_u16_r2 * FF(281474976710656UL))))) +
                            (alu_u128_tag * ((((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                                                  (alu_u16_r1 * FF(4294967296UL))) +
                                                 (alu_u16_r2 * FF(281474976710656UL))) +
                                                (alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                               (alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                                              (alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                                             (alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))))) +
                           (alu_ff_tag * alu_ia)) -
                          alu_ic)) +
                        ((alu_ff_tag * (alu_op_add - alu_op_sub)) * alu_ib));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            Avm_DECLARE_VIEWS(14);
            auto tmp = ((alu_ff_tag * alu_op_mul) * ((alu_ia * alu_ib) - alu_ic));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            Avm_DECLARE_VIEWS(15);
            auto tmp = ((((-alu_ff_tag + FF(1)) - alu_u128_tag) * alu_op_mul) *
                        (((((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                               (alu_u16_r1 * FF(4294967296UL))) +
                              (alu_u16_r2 * FF(281474976710656UL))) +
                             (alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                            (alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                           (alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                          (alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) -
                         (alu_ia * alu_ib)));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            Avm_DECLARE_VIEWS(16);
            auto tmp =
                (alu_op_mul * (((((alu_u8_tag * alu_u8_r0) + (alu_u16_tag * (alu_u8_r0 + (alu_u8_r1 * FF(256))))) +
                                 (alu_u32_tag * ((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))))) +
                                (alu_u64_tag * ((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                                                 (alu_u16_r1 * FF(4294967296UL))) +
                                                (alu_u16_r2 * FF(281474976710656UL))))) -
                               (((-alu_ff_tag + FF(1)) - alu_u128_tag) * alu_ic)));
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            Avm_DECLARE_VIEWS(17);
            auto tmp = ((alu_u128_tag * alu_op_mul) *
                        ((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                            (alu_u16_r1 * FF(4294967296UL))) +
                           (alu_u16_r2 * FF(281474976710656UL))) +
                          ((((alu_u16_r3 + (alu_u16_r4 * FF(65536))) + (alu_u16_r5 * FF(4294967296UL))) +
                            (alu_u16_r6 * FF(281474976710656UL))) *
                           FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                         alu_ia));
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            Avm_DECLARE_VIEWS(18);
            auto tmp =
                ((alu_u128_tag * alu_op_mul) *
                 ((((((alu_u8_r0_shift + (alu_u8_r1_shift * FF(256))) + (alu_u16_r0_shift * FF(65536))) +
                     (alu_u16_r1_shift * FF(4294967296UL))) +
                    (alu_u16_r2_shift * FF(281474976710656UL))) +
                   ((((alu_u16_r3_shift + (alu_u16_r4_shift * FF(65536))) + (alu_u16_r5_shift * FF(4294967296UL))) +
                     (alu_u16_r6_shift * FF(281474976710656UL))) *
                    FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                  alu_ib));
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            Avm_DECLARE_VIEWS(19);
            auto tmp =
                ((alu_u128_tag * alu_op_mul) *
                 ((((alu_ia * ((((alu_u8_r0_shift + (alu_u8_r1_shift * FF(256))) + (alu_u16_r0_shift * FF(65536))) +
                                (alu_u16_r1_shift * FF(4294967296UL))) +
                               (alu_u16_r2_shift * FF(281474976710656UL)))) +
                    ((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                        (alu_u16_r1 * FF(4294967296UL))) +
                       (alu_u16_r2 * FF(281474976710656UL))) *
                      (((alu_u16_r3_shift + (alu_u16_r4_shift * FF(65536))) + (alu_u16_r5_shift * FF(4294967296UL))) +
                       (alu_u16_r6_shift * FF(281474976710656UL)))) *
                     FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                   (((alu_cf * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL })) +
                     (((alu_u16_r7 + (alu_u16_r8 * FF(65536))) + (alu_u16_r9 * FF(4294967296UL))) +
                      (alu_u16_r10 * FF(281474976710656UL)))) *
                    FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                  alu_ic));
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            Avm_DECLARE_VIEWS(20);
            auto tmp = (alu_op_not * alu_ff_tag);
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
        // Contribution 21
        {
            Avm_DECLARE_VIEWS(21);
            auto tmp = (alu_op_not *
                        ((alu_ia + alu_ic) -
                         ((((((alu_u8_tag * FF(256)) + (alu_u16_tag * FF(65536))) + (alu_u32_tag * FF(4294967296UL))) +
                            (alu_u64_tag * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                           (alu_u128_tag * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                          FF(1))));
            tmp *= scaling_factor;
            std::get<21>(evals) += tmp;
        }
        // Contribution 22
        {
            Avm_DECLARE_VIEWS(22);
            auto tmp = ((alu_sel_cmp + alu_op_eq) * (alu_ic * (-alu_ic + FF(1))));
            tmp *= scaling_factor;
            std::get<22>(evals) += tmp;
        }
        // Contribution 23
        {
            Avm_DECLARE_VIEWS(23);
            auto tmp =
                (alu_op_eq *
                 ((((alu_ia - alu_ib) * ((alu_ic * (-alu_op_eq_diff_inv + FF(1))) + alu_op_eq_diff_inv)) - FF(1)) +
                  alu_ic));
            tmp *= scaling_factor;
            std::get<23>(evals) += tmp;
        }
        // Contribution 24
        {
            Avm_DECLARE_VIEWS(24);
            auto tmp = (((alu_op_lt * alu_ib) + ((alu_op_lte + alu_op_cast) * alu_ia)) -
                        ((alu_a_lo + (alu_a_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) * (alu_sel_cmp + alu_op_cast)));
            tmp *= scaling_factor;
            std::get<24>(evals) += tmp;
        }
        // Contribution 25
        {
            Avm_DECLARE_VIEWS(25);
            auto tmp = (((alu_op_lt * alu_ia) + (alu_op_lte * alu_ib)) -
                        ((alu_b_lo + (alu_b_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) * alu_sel_cmp));
            tmp *= scaling_factor;
            std::get<25>(evals) += tmp;
        }
        // Contribution 26
        {
            Avm_DECLARE_VIEWS(26);
            auto tmp = (alu_p_a_borrow * (-alu_p_a_borrow + FF(1)));
            tmp *= scaling_factor;
            std::get<26>(evals) += tmp;
        }
        // Contribution 27
        {
            Avm_DECLARE_VIEWS(27);
            auto tmp = ((alu_p_sub_a_lo -
                         ((-alu_a_lo + FF(uint256_t{ 4891460686036598784UL, 2896914383306846353UL, 0UL, 0UL })) +
                          (alu_p_a_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))) *
                        ((alu_sel_cmp + alu_op_cast) + alu_op_div_std));
            tmp *= scaling_factor;
            std::get<27>(evals) += tmp;
        }
        // Contribution 28
        {
            Avm_DECLARE_VIEWS(28);
            auto tmp = ((alu_p_sub_a_hi -
                         ((-alu_a_hi + FF(uint256_t{ 13281191951274694749UL, 3486998266802970665UL, 0UL, 0UL })) -
                          alu_p_a_borrow)) *
                        ((alu_sel_cmp + alu_op_cast) + alu_op_div_std));
            tmp *= scaling_factor;
            std::get<28>(evals) += tmp;
        }
        // Contribution 29
        {
            Avm_DECLARE_VIEWS(29);
            auto tmp = (alu_p_b_borrow * (-alu_p_b_borrow + FF(1)));
            tmp *= scaling_factor;
            std::get<29>(evals) += tmp;
        }
        // Contribution 30
        {
            Avm_DECLARE_VIEWS(30);
            auto tmp = ((alu_p_sub_b_lo -
                         ((-alu_b_lo + FF(uint256_t{ 4891460686036598784UL, 2896914383306846353UL, 0UL, 0UL })) +
                          (alu_p_b_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))) *
                        alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<30>(evals) += tmp;
        }
        // Contribution 31
        {
            Avm_DECLARE_VIEWS(31);
            auto tmp = ((alu_p_sub_b_hi -
                         ((-alu_b_hi + FF(uint256_t{ 13281191951274694749UL, 3486998266802970665UL, 0UL, 0UL })) -
                          alu_p_b_borrow)) *
                        alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<31>(evals) += tmp;
        }
        // Contribution 32
        {
            Avm_DECLARE_VIEWS(32);
            auto tmp = ((alu_res_lo -
                         (((((alu_a_lo - alu_b_lo) - FF(1)) + (alu_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                           ((alu_op_lt * alu_ic) + ((-alu_ic + FF(1)) * alu_op_lte))) +
                          (((alu_b_lo - alu_a_lo) + (alu_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                           (-((alu_op_lt * alu_ic) + ((-alu_ic + FF(1)) * alu_op_lte)) + FF(1))))) *
                        alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<32>(evals) += tmp;
        }
        // Contribution 33
        {
            Avm_DECLARE_VIEWS(33);
            auto tmp =
                ((alu_res_hi -
                  ((((alu_a_hi - alu_b_hi) - alu_borrow) * ((alu_op_lt * alu_ic) + ((-alu_ic + FF(1)) * alu_op_lte))) +
                   (((alu_b_hi - alu_a_hi) - alu_borrow) *
                    (-((alu_op_lt * alu_ic) + ((-alu_ic + FF(1)) * alu_op_lte)) + FF(1))))) *
                 alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<33>(evals) += tmp;
        }
        // Contribution 34
        {
            Avm_DECLARE_VIEWS(34);
            auto tmp = (((alu_cmp_rng_ctr_shift - alu_cmp_rng_ctr) + FF(1)) * alu_cmp_rng_ctr);
            tmp *= scaling_factor;
            std::get<34>(evals) += tmp;
        }
        // Contribution 35
        {
            Avm_DECLARE_VIEWS(35);
            auto tmp = ((alu_cmp_rng_ctr_shift - FF(4)) * alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<35>(evals) += tmp;
        }
        // Contribution 36
        {
            Avm_DECLARE_VIEWS(36);
            auto tmp = (alu_sel_rng_chk * (-alu_sel_rng_chk + FF(1)));
            tmp *= scaling_factor;
            std::get<36>(evals) += tmp;
        }
        // Contribution 37
        {
            Avm_DECLARE_VIEWS(37);
            auto tmp = (alu_sel_rng_chk * alu_sel_cmp);
            tmp *= scaling_factor;
            std::get<37>(evals) += tmp;
        }
        // Contribution 38
        {
            Avm_DECLARE_VIEWS(38);
            auto tmp = ((alu_cmp_rng_ctr *
                         (((-alu_sel_rng_chk + FF(1)) * (-alu_op_eq_diff_inv + FF(1))) + alu_op_eq_diff_inv)) -
                        alu_sel_rng_chk);
            tmp *= scaling_factor;
            std::get<38>(evals) += tmp;
        }
        // Contribution 39
        {
            Avm_DECLARE_VIEWS(39);
            auto tmp = (alu_sel_rng_chk_lookup_shift -
                        ((((((((((alu_sel_cmp_shift + alu_sel_rng_chk_shift) + alu_op_add_shift) + alu_op_sub_shift) +
                               alu_op_mul_shift) +
                              (alu_op_mul * alu_u128_tag)) +
                             alu_op_cast_shift) +
                            alu_op_cast_prev_shift) +
                           alu_op_shl_shift) +
                          alu_op_shr_shift) +
                         alu_op_div_shift));
            tmp *= scaling_factor;
            std::get<39>(evals) += tmp;
        }
        // Contribution 40
        {
            Avm_DECLARE_VIEWS(40);
            auto tmp =
                (alu_a_lo -
                 (((((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                        (alu_u16_r1 * FF(4294967296UL))) +
                       (alu_u16_r2 * FF(281474976710656UL))) +
                      (alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                     (alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                    (alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                   (alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) *
                  (((((alu_sel_rng_chk + alu_sel_cmp) + alu_op_cast) + alu_op_cast_prev) + alu_shift_lt_bit_len) +
                   alu_op_div)));
            tmp *= scaling_factor;
            std::get<40>(evals) += tmp;
        }
        // Contribution 41
        {
            Avm_DECLARE_VIEWS(41);
            auto tmp =
                (alu_a_hi -
                 ((((((((alu_u16_r7 + (alu_u16_r8 * FF(65536))) + (alu_u16_r9 * FF(4294967296UL))) +
                       (alu_u16_r10 * FF(281474976710656UL))) +
                      (alu_u16_r11 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                     (alu_u16_r12 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                    (alu_u16_r13 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                   (alu_u16_r14 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) *
                  (((((alu_sel_rng_chk + alu_sel_cmp) + alu_op_cast) + alu_op_cast_prev) + alu_shift_lt_bit_len) +
                   alu_op_div)));
            tmp *= scaling_factor;
            std::get<41>(evals) += tmp;
        }
        // Contribution 42
        {
            Avm_DECLARE_VIEWS(42);
            auto tmp = ((alu_a_lo_shift - alu_b_lo) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<42>(evals) += tmp;
        }
        // Contribution 43
        {
            Avm_DECLARE_VIEWS(43);
            auto tmp = ((alu_a_hi_shift - alu_b_hi) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<43>(evals) += tmp;
        }
        // Contribution 44
        {
            Avm_DECLARE_VIEWS(44);
            auto tmp = ((alu_b_lo_shift - alu_p_sub_a_lo) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<44>(evals) += tmp;
        }
        // Contribution 45
        {
            Avm_DECLARE_VIEWS(45);
            auto tmp = ((alu_b_hi_shift - alu_p_sub_a_hi) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<45>(evals) += tmp;
        }
        // Contribution 46
        {
            Avm_DECLARE_VIEWS(46);
            auto tmp = ((alu_p_sub_a_lo_shift - alu_p_sub_b_lo) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<46>(evals) += tmp;
        }
        // Contribution 47
        {
            Avm_DECLARE_VIEWS(47);
            auto tmp = ((alu_p_sub_a_hi_shift - alu_p_sub_b_hi) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<47>(evals) += tmp;
        }
        // Contribution 48
        {
            Avm_DECLARE_VIEWS(48);
            auto tmp = ((alu_p_sub_b_lo_shift - alu_res_lo) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<48>(evals) += tmp;
        }
        // Contribution 49
        {
            Avm_DECLARE_VIEWS(49);
            auto tmp = ((alu_p_sub_b_hi_shift - alu_res_hi) * alu_sel_rng_chk_shift);
            tmp *= scaling_factor;
            std::get<49>(evals) += tmp;
        }
        // Contribution 50
        {
            Avm_DECLARE_VIEWS(50);
            auto tmp = (alu_op_cast_prev_shift - alu_op_cast);
            tmp *= scaling_factor;
            std::get<50>(evals) += tmp;
        }
        // Contribution 51
        {
            Avm_DECLARE_VIEWS(51);
            auto tmp = (alu_op_cast *
                        (((((((alu_u8_tag * alu_u8_r0) + (alu_u16_tag * (alu_u8_r0 + (alu_u8_r1 * FF(256))))) +
                             (alu_u32_tag * ((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))))) +
                            (alu_u64_tag * ((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                                             (alu_u16_r1 * FF(4294967296UL))) +
                                            (alu_u16_r2 * FF(281474976710656UL))))) +
                           (alu_u128_tag * ((((((((alu_u8_r0 + (alu_u8_r1 * FF(256))) + (alu_u16_r0 * FF(65536))) +
                                                 (alu_u16_r1 * FF(4294967296UL))) +
                                                (alu_u16_r2 * FF(281474976710656UL))) +
                                               (alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                              (alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                                             (alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                                            (alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))))) +
                          (alu_ff_tag * alu_ia)) -
                         alu_ic));
            tmp *= scaling_factor;
            std::get<51>(evals) += tmp;
        }
        // Contribution 52
        {
            Avm_DECLARE_VIEWS(52);
            auto tmp = (alu_op_cast * (alu_a_lo_shift - alu_p_sub_a_lo));
            tmp *= scaling_factor;
            std::get<52>(evals) += tmp;
        }
        // Contribution 53
        {
            Avm_DECLARE_VIEWS(53);
            auto tmp = (alu_op_cast * (alu_a_hi_shift - alu_p_sub_a_hi));
            tmp *= scaling_factor;
            std::get<53>(evals) += tmp;
        }
        // Contribution 54
        {
            Avm_DECLARE_VIEWS(54);
            auto tmp = (((alu_op_mul * alu_u128_tag) + alu_op_cast) * alu_sel_alu_shift);
            tmp *= scaling_factor;
            std::get<54>(evals) += tmp;
        }
        // Contribution 55
        {
            Avm_DECLARE_VIEWS(55);
            auto tmp = ((alu_shift_lt_bit_len * alu_op_shr) * (alu_a_lo - ((alu_two_pow_s - alu_b_lo) - FF(1))));
            tmp *= scaling_factor;
            std::get<55>(evals) += tmp;
        }
        // Contribution 56
        {
            Avm_DECLARE_VIEWS(56);
            auto tmp = ((alu_shift_lt_bit_len * alu_op_shr) * (alu_a_hi - ((alu_two_pow_t_sub_s - alu_b_hi) - FF(1))));
            tmp *= scaling_factor;
            std::get<56>(evals) += tmp;
        }
        // Contribution 57
        {
            Avm_DECLARE_VIEWS(57);
            auto tmp = ((alu_shift_lt_bit_len * alu_op_shl) * (alu_a_lo - ((alu_two_pow_t_sub_s - alu_b_lo) - FF(1))));
            tmp *= scaling_factor;
            std::get<57>(evals) += tmp;
        }
        // Contribution 58
        {
            Avm_DECLARE_VIEWS(58);
            auto tmp = ((alu_shift_lt_bit_len * alu_op_shl) * (alu_a_hi - ((alu_two_pow_s - alu_b_hi) - FF(1))));
            tmp *= scaling_factor;
            std::get<58>(evals) += tmp;
        }
        // Contribution 59
        {
            Avm_DECLARE_VIEWS(59);
            auto tmp = (alu_shift_lt_bit_len * (-alu_shift_lt_bit_len + FF(1)));
            tmp *= scaling_factor;
            std::get<59>(evals) += tmp;
        }
        // Contribution 60
        {
            Avm_DECLARE_VIEWS(60);
            auto tmp = (alu_t_sub_s_bits -
                        (alu_sel_shift_which *
                         ((alu_shift_lt_bit_len *
                           ((((((alu_u8_tag * FF(8)) + (alu_u16_tag * FF(16))) + (alu_u32_tag * FF(32))) +
                              (alu_u64_tag * FF(64))) +
                             (alu_u128_tag * FF(128))) -
                            alu_ib)) +
                          ((-alu_shift_lt_bit_len + FF(1)) *
                           (alu_ib - (((((alu_u8_tag * FF(8)) + (alu_u16_tag * FF(16))) + (alu_u32_tag * FF(32))) +
                                       (alu_u64_tag * FF(64))) +
                                      (alu_u128_tag * FF(128))))))));
            tmp *= scaling_factor;
            std::get<60>(evals) += tmp;
        }
        // Contribution 61
        {
            Avm_DECLARE_VIEWS(61);
            auto tmp = ((alu_shift_lt_bit_len * alu_op_shr) * (((alu_b_hi * alu_two_pow_s) + alu_b_lo) - alu_ia));
            tmp *= scaling_factor;
            std::get<61>(evals) += tmp;
        }
        // Contribution 62
        {
            Avm_DECLARE_VIEWS(62);
            auto tmp = (alu_op_shr * (alu_ic - (alu_b_hi * alu_shift_lt_bit_len)));
            tmp *= scaling_factor;
            std::get<62>(evals) += tmp;
        }
        // Contribution 63
        {
            Avm_DECLARE_VIEWS(63);
            auto tmp = ((alu_shift_lt_bit_len * alu_op_shl) * (((alu_b_hi * alu_two_pow_t_sub_s) + alu_b_lo) - alu_ia));
            tmp *= scaling_factor;
            std::get<63>(evals) += tmp;
        }
        // Contribution 64
        {
            Avm_DECLARE_VIEWS(64);
            auto tmp = (alu_op_shl * (alu_ic - ((alu_b_lo * alu_two_pow_s) * alu_shift_lt_bit_len)));
            tmp *= scaling_factor;
            std::get<64>(evals) += tmp;
        }
        // Contribution 65
        {
            Avm_DECLARE_VIEWS(65);
            auto tmp = (alu_op_div - (alu_op_div_std + alu_op_div_a_lt_b));
            tmp *= scaling_factor;
            std::get<65>(evals) += tmp;
        }
        // Contribution 66
        {
            Avm_DECLARE_VIEWS(66);
            auto tmp = (alu_op_div_a_lt_b * (-alu_op_div_a_lt_b + FF(1)));
            tmp *= scaling_factor;
            std::get<66>(evals) += tmp;
        }
        // Contribution 67
        {
            Avm_DECLARE_VIEWS(67);
            auto tmp = (alu_op_div_a_lt_b * (alu_a_lo - ((alu_ib - alu_ia) - FF(1))));
            tmp *= scaling_factor;
            std::get<67>(evals) += tmp;
        }
        // Contribution 68
        {
            Avm_DECLARE_VIEWS(68);
            auto tmp = (alu_op_div_a_lt_b * alu_ic);
            tmp *= scaling_factor;
            std::get<68>(evals) += tmp;
        }
        // Contribution 69
        {
            Avm_DECLARE_VIEWS(69);
            auto tmp = (alu_op_div_a_lt_b * (alu_ia - alu_remainder));
            tmp *= scaling_factor;
            std::get<69>(evals) += tmp;
        }
        // Contribution 70
        {
            Avm_DECLARE_VIEWS(70);
            auto tmp = (alu_op_div_std * (-alu_op_div_std + FF(1)));
            tmp *= scaling_factor;
            std::get<70>(evals) += tmp;
        }
        // Contribution 71
        {
            Avm_DECLARE_VIEWS(71);
            auto tmp =
                (alu_op_div_std * ((alu_ib - alu_divisor_lo) - (alu_divisor_hi * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))));
            tmp *= scaling_factor;
            std::get<71>(evals) += tmp;
        }
        // Contribution 72
        {
            Avm_DECLARE_VIEWS(72);
            auto tmp = (alu_op_div_std *
                        ((alu_ic - alu_quotient_lo) - (alu_quotient_hi * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))));
            tmp *= scaling_factor;
            std::get<72>(evals) += tmp;
        }
        // Contribution 73
        {
            Avm_DECLARE_VIEWS(73);
            auto tmp = (((alu_divisor_hi * alu_quotient_lo) + (alu_divisor_lo * alu_quotient_hi)) -
                        (alu_partial_prod_lo + (alu_partial_prod_hi * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))));
            tmp *= scaling_factor;
            std::get<73>(evals) += tmp;
        }
        // Contribution 74
        {
            Avm_DECLARE_VIEWS(74);
            auto tmp =
                (alu_op_div_std *
                 ((((alu_divisor_lo * alu_quotient_lo) + (alu_partial_prod_lo * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                   ((alu_partial_prod_hi + (alu_divisor_hi * alu_quotient_hi)) * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                  (alu_a_lo + (alu_a_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))));
            tmp *= scaling_factor;
            std::get<74>(evals) += tmp;
        }
        // Contribution 75
        {
            Avm_DECLARE_VIEWS(75);
            auto tmp = (alu_op_div_std * (alu_b_hi - ((alu_ib - alu_remainder) - FF(1))));
            tmp *= scaling_factor;
            std::get<75>(evals) += tmp;
        }
        // Contribution 76
        {
            Avm_DECLARE_VIEWS(76);
            auto tmp = ((alu_cmp_rng_ctr_shift - FF(2)) * alu_op_div_std);
            tmp *= scaling_factor;
            std::get<76>(evals) += tmp;
        }
        // Contribution 77
        {
            Avm_DECLARE_VIEWS(77);
            auto tmp = (alu_sel_rng_chk * alu_op_div_std);
            tmp *= scaling_factor;
            std::get<77>(evals) += tmp;
        }
        // Contribution 78
        {
            Avm_DECLARE_VIEWS(78);
            auto tmp =
                (alu_op_div_std *
                 ((((alu_divisor_lo * alu_quotient_lo) + (alu_partial_prod_lo * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                   ((alu_partial_prod_hi + (alu_divisor_hi * alu_quotient_hi)) * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                  (alu_ia - alu_remainder)));
            tmp *= scaling_factor;
            std::get<78>(evals) += tmp;
        }
        // Contribution 79
        {
            Avm_DECLARE_VIEWS(79);
            auto tmp = (alu_sel_div_rng_chk * (-alu_sel_div_rng_chk + FF(1)));
            tmp *= scaling_factor;
            std::get<79>(evals) += tmp;
        }
        // Contribution 80
        {
            Avm_DECLARE_VIEWS(80);
            auto tmp = ((alu_sel_div_rng_chk * alu_sel_div_rng_chk_shift) - alu_op_div_std);
            tmp *= scaling_factor;
            std::get<80>(evals) += tmp;
        }
        // Contribution 81
        {
            Avm_DECLARE_VIEWS(81);
            auto tmp = (alu_divisor_lo - (alu_op_div_std * (((alu_div_u16_r0 + (alu_div_u16_r1 * FF(65536))) +
                                                             (alu_div_u16_r2 * FF(4294967296UL))) +
                                                            (alu_div_u16_r3 * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<81>(evals) += tmp;
        }
        // Contribution 82
        {
            Avm_DECLARE_VIEWS(82);
            auto tmp = (alu_divisor_hi - (alu_op_div_std * (((alu_div_u16_r4 + (alu_div_u16_r5 * FF(65536))) +
                                                             (alu_div_u16_r6 * FF(4294967296UL))) +
                                                            (alu_div_u16_r7 * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<82>(evals) += tmp;
        }
        // Contribution 83
        {
            Avm_DECLARE_VIEWS(83);
            auto tmp =
                (alu_quotient_lo - (alu_op_div_std * (((alu_div_u16_r0_shift + (alu_div_u16_r1_shift * FF(65536))) +
                                                       (alu_div_u16_r2_shift * FF(4294967296UL))) +
                                                      (alu_div_u16_r3_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<83>(evals) += tmp;
        }
        // Contribution 84
        {
            Avm_DECLARE_VIEWS(84);
            auto tmp =
                (alu_quotient_hi - (alu_op_div_std * (((alu_div_u16_r4_shift + (alu_div_u16_r5_shift * FF(65536))) +
                                                       (alu_div_u16_r6_shift * FF(4294967296UL))) +
                                                      (alu_div_u16_r7_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<84>(evals) += tmp;
        }
        // Contribution 85
        {
            Avm_DECLARE_VIEWS(85);
            auto tmp = (alu_partial_prod_lo - (alu_op_div_std * ((((alu_u8_r0_shift + (alu_u8_r1_shift * FF(256))) +
                                                                   (alu_u16_r0_shift * FF(65536))) +
                                                                  (alu_u16_r1_shift * FF(4294967296UL))) +
                                                                 (alu_u16_r2_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<85>(evals) += tmp;
        }
        // Contribution 86
        {
            Avm_DECLARE_VIEWS(86);
            auto tmp = (alu_partial_prod_hi - (alu_op_div_std * (((alu_u16_r3_shift + (alu_u16_r4_shift * FF(65536))) +
                                                                  (alu_u16_r5_shift * FF(4294967296UL))) +
                                                                 (alu_u16_r6_shift * FF(281474976710656UL)))));
            tmp *= scaling_factor;
            std::get<86>(evals) += tmp;
        }
    }
};

template <typename FF> using alu = Relation<aluImpl<FF>>;

} // namespace bb::Avm_vm