
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Avm_aluRow {
    FF avm_alu_a_hi{};
    FF avm_alu_a_hi_shift{};
    FF avm_alu_a_lo{};
    FF avm_alu_a_lo_shift{};
    FF avm_alu_alu_sel{};
    FF avm_alu_b_hi{};
    FF avm_alu_b_hi_shift{};
    FF avm_alu_b_lo{};
    FF avm_alu_b_lo_shift{};
    FF avm_alu_borrow{};
    FF avm_alu_cf{};
    FF avm_alu_cmp_rng_ctr{};
    FF avm_alu_cmp_rng_ctr_shift{};
    FF avm_alu_cmp_sel{};
    FF avm_alu_cmp_sel_shift{};
    FF avm_alu_ff_tag{};
    FF avm_alu_ia{};
    FF avm_alu_ib{};
    FF avm_alu_ic{};
    FF avm_alu_in_tag{};
    FF avm_alu_op_add{};
    FF avm_alu_op_add_shift{};
    FF avm_alu_op_eq{};
    FF avm_alu_op_eq_diff_inv{};
    FF avm_alu_op_lt{};
    FF avm_alu_op_lte{};
    FF avm_alu_op_mul{};
    FF avm_alu_op_mul_shift{};
    FF avm_alu_op_not{};
    FF avm_alu_op_sub{};
    FF avm_alu_op_sub_shift{};
    FF avm_alu_p_a_borrow{};
    FF avm_alu_p_b_borrow{};
    FF avm_alu_p_sub_a_hi{};
    FF avm_alu_p_sub_a_hi_shift{};
    FF avm_alu_p_sub_a_lo{};
    FF avm_alu_p_sub_a_lo_shift{};
    FF avm_alu_p_sub_b_hi{};
    FF avm_alu_p_sub_b_hi_shift{};
    FF avm_alu_p_sub_b_lo{};
    FF avm_alu_p_sub_b_lo_shift{};
    FF avm_alu_res_hi{};
    FF avm_alu_res_lo{};
    FF avm_alu_rng_chk_lookup_selector_shift{};
    FF avm_alu_rng_chk_sel{};
    FF avm_alu_rng_chk_sel_shift{};
    FF avm_alu_u128_tag{};
    FF avm_alu_u16_r0{};
    FF avm_alu_u16_r0_shift{};
    FF avm_alu_u16_r1{};
    FF avm_alu_u16_r10{};
    FF avm_alu_u16_r11{};
    FF avm_alu_u16_r12{};
    FF avm_alu_u16_r13{};
    FF avm_alu_u16_r14{};
    FF avm_alu_u16_r1_shift{};
    FF avm_alu_u16_r2{};
    FF avm_alu_u16_r2_shift{};
    FF avm_alu_u16_r3{};
    FF avm_alu_u16_r3_shift{};
    FF avm_alu_u16_r4{};
    FF avm_alu_u16_r4_shift{};
    FF avm_alu_u16_r5{};
    FF avm_alu_u16_r5_shift{};
    FF avm_alu_u16_r6{};
    FF avm_alu_u16_r6_shift{};
    FF avm_alu_u16_r7{};
    FF avm_alu_u16_r8{};
    FF avm_alu_u16_r9{};
    FF avm_alu_u16_tag{};
    FF avm_alu_u32_tag{};
    FF avm_alu_u64_tag{};
    FF avm_alu_u8_r0{};
    FF avm_alu_u8_r0_shift{};
    FF avm_alu_u8_r1{};
    FF avm_alu_u8_r1_shift{};
    FF avm_alu_u8_tag{};
};

inline std::string get_relation_label_avm_alu(int index)
{
    switch (index) {
    case 11:
        return "ALU_ADD_SUB_1";

    case 12:
        return "ALU_ADD_SUB_2";

    case 13:
        return "ALU_MULTIPLICATION_FF";

    case 14:
        return "ALU_MUL_COMMON_1";

    case 15:
        return "ALU_MUL_COMMON_2";

    case 18:
        return "ALU_MULTIPLICATION_OUT_U128";

    case 19:
        return "ALU_FF_NOT_XOR";

    case 20:
        return "ALU_OP_NOT";

    case 21:
        return "ALU_RES_IS_BOOL";

    case 22:
        return "ALU_OP_EQ";

    case 23:
        return "INPUT_DECOMP_1";

    case 24:
        return "INPUT_DECOMP_2";

    case 26:
        return "SUB_LO_1";

    case 27:
        return "SUB_HI_1";

    case 29:
        return "SUB_LO_2";

    case 30:
        return "SUB_HI_2";

    case 31:
        return "RES_LO";

    case 32:
        return "RES_HI";

    case 33:
        return "CMP_CTR_REL_1";

    case 34:
        return "CMP_CTR_REL_2";

    case 37:
        return "CTR_NON_ZERO_REL";

    case 38:
        return "RNG_CHK_LOOKUP_SELECTOR";

    case 39:
        return "LOWER_CMP_RNG_CHK";

    case 40:
        return "UPPER_CMP_RNG_CHK";

    case 41:
        return "SHIFT_RELS_0";

    case 43:
        return "SHIFT_RELS_1";

    case 45:
        return "SHIFT_RELS_2";

    case 47:
        return "SHIFT_RELS_3";
    }
    return std::to_string(index);
}

template <typename FF_> class avm_aluImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 49> SUBRELATION_PARTIAL_LENGTHS{
        2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 5, 5, 5, 5, 6, 6, 8, 3, 4, 4, 5, 4, 4,
        3, 4, 3, 3, 4, 3, 6, 5, 3, 3, 3, 3, 4, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3,
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

            auto tmp = (avm_alu_alu_sel -
                        ((((((avm_alu_op_add + avm_alu_op_sub) + avm_alu_op_mul) + avm_alu_op_not) + avm_alu_op_eq) +
                          avm_alu_op_lt) +
                         avm_alu_op_lte));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);

            auto tmp = (avm_alu_cmp_sel - (avm_alu_op_lt + avm_alu_op_lte));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);

            auto tmp = (avm_alu_cf * (-avm_alu_cf + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);

            auto tmp = (avm_alu_ff_tag * (-avm_alu_ff_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);

            auto tmp = (avm_alu_u8_tag * (-avm_alu_u8_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);

            auto tmp = (avm_alu_u16_tag * (-avm_alu_u16_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);

            auto tmp = (avm_alu_u32_tag * (-avm_alu_u32_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);

            auto tmp = (avm_alu_u64_tag * (-avm_alu_u64_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);

            auto tmp = (avm_alu_u128_tag * (-avm_alu_u128_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);

            auto tmp =
                (avm_alu_alu_sel *
                 ((((((avm_alu_ff_tag + avm_alu_u8_tag) + avm_alu_u16_tag) + avm_alu_u32_tag) + avm_alu_u64_tag) +
                   avm_alu_u128_tag) -
                  FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);

            auto tmp = (avm_alu_in_tag - (((((avm_alu_u8_tag + (avm_alu_u16_tag * FF(2))) + (avm_alu_u32_tag * FF(3))) +
                                            (avm_alu_u64_tag * FF(4))) +
                                           (avm_alu_u128_tag * FF(5))) +
                                          (avm_alu_ff_tag * FF(6))));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            Avm_DECLARE_VIEWS(11);

            auto tmp = (((avm_alu_op_add + avm_alu_op_sub) *
                         ((((((((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                                 (avm_alu_u16_r1 * FF(4294967296UL))) +
                                (avm_alu_u16_r2 * FF(281474976710656UL))) +
                               (avm_alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                              (avm_alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                             (avm_alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                            (avm_alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) -
                           avm_alu_ia) +
                          (avm_alu_ff_tag * avm_alu_ic))) +
                        ((avm_alu_op_add - avm_alu_op_sub) *
                         ((avm_alu_cf * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })) - avm_alu_ib)));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            Avm_DECLARE_VIEWS(12);

            auto tmp = (((avm_alu_op_add + avm_alu_op_sub) *
                         (((((((avm_alu_u8_tag * avm_alu_u8_r0) +
                               (avm_alu_u16_tag * (avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))))) +
                              (avm_alu_u32_tag *
                               ((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))))) +
                             (avm_alu_u64_tag *
                              ((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                                (avm_alu_u16_r1 * FF(4294967296UL))) +
                               (avm_alu_u16_r2 * FF(281474976710656UL))))) +
                            (avm_alu_u128_tag *
                             ((((((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                                   (avm_alu_u16_r1 * FF(4294967296UL))) +
                                  (avm_alu_u16_r2 * FF(281474976710656UL))) +
                                 (avm_alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                (avm_alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                               (avm_alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                              (avm_alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))))) +
                           (avm_alu_ff_tag * avm_alu_ia)) -
                          avm_alu_ic)) +
                        ((avm_alu_ff_tag * (avm_alu_op_add - avm_alu_op_sub)) * avm_alu_ib));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            Avm_DECLARE_VIEWS(13);

            auto tmp = ((avm_alu_ff_tag * avm_alu_op_mul) * ((avm_alu_ia * avm_alu_ib) - avm_alu_ic));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            Avm_DECLARE_VIEWS(14);

            auto tmp = ((((-avm_alu_ff_tag + FF(1)) - avm_alu_u128_tag) * avm_alu_op_mul) *
                        (((((((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                               (avm_alu_u16_r1 * FF(4294967296UL))) +
                              (avm_alu_u16_r2 * FF(281474976710656UL))) +
                             (avm_alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                            (avm_alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                           (avm_alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                          (avm_alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) -
                         (avm_alu_ia * avm_alu_ib)));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            Avm_DECLARE_VIEWS(15);

            auto tmp =
                (avm_alu_op_mul *
                 (((((avm_alu_u8_tag * avm_alu_u8_r0) +
                     (avm_alu_u16_tag * (avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))))) +
                    (avm_alu_u32_tag * ((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))))) +
                   (avm_alu_u64_tag * ((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                                        (avm_alu_u16_r1 * FF(4294967296UL))) +
                                       (avm_alu_u16_r2 * FF(281474976710656UL))))) -
                  (((-avm_alu_ff_tag + FF(1)) - avm_alu_u128_tag) * avm_alu_ic)));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            Avm_DECLARE_VIEWS(16);

            auto tmp = ((avm_alu_u128_tag * avm_alu_op_mul) *
                        ((((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                            (avm_alu_u16_r1 * FF(4294967296UL))) +
                           (avm_alu_u16_r2 * FF(281474976710656UL))) +
                          ((((avm_alu_u16_r3 + (avm_alu_u16_r4 * FF(65536))) + (avm_alu_u16_r5 * FF(4294967296UL))) +
                            (avm_alu_u16_r6 * FF(281474976710656UL))) *
                           FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                         avm_alu_ia));
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            Avm_DECLARE_VIEWS(17);

            auto tmp =
                ((avm_alu_u128_tag * avm_alu_op_mul) *
                 ((((((avm_alu_u8_r0_shift + (avm_alu_u8_r1_shift * FF(256))) + (avm_alu_u16_r0_shift * FF(65536))) +
                     (avm_alu_u16_r1_shift * FF(4294967296UL))) +
                    (avm_alu_u16_r2_shift * FF(281474976710656UL))) +
                   ((((avm_alu_u16_r3_shift + (avm_alu_u16_r4_shift * FF(65536))) +
                      (avm_alu_u16_r5_shift * FF(4294967296UL))) +
                     (avm_alu_u16_r6_shift * FF(281474976710656UL))) *
                    FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                  avm_alu_ib));
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            Avm_DECLARE_VIEWS(18);

            auto tmp =
                ((avm_alu_u128_tag * avm_alu_op_mul) *
                 ((((avm_alu_ia *
                     ((((avm_alu_u8_r0_shift + (avm_alu_u8_r1_shift * FF(256))) + (avm_alu_u16_r0_shift * FF(65536))) +
                       (avm_alu_u16_r1_shift * FF(4294967296UL))) +
                      (avm_alu_u16_r2_shift * FF(281474976710656UL)))) +
                    ((((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                        (avm_alu_u16_r1 * FF(4294967296UL))) +
                       (avm_alu_u16_r2 * FF(281474976710656UL))) *
                      (((avm_alu_u16_r3_shift + (avm_alu_u16_r4_shift * FF(65536))) +
                        (avm_alu_u16_r5_shift * FF(4294967296UL))) +
                       (avm_alu_u16_r6_shift * FF(281474976710656UL)))) *
                     FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) -
                   (((avm_alu_cf * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL })) +
                     (((avm_alu_u16_r7 + (avm_alu_u16_r8 * FF(65536))) + (avm_alu_u16_r9 * FF(4294967296UL))) +
                      (avm_alu_u16_r10 * FF(281474976710656UL)))) *
                    FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                  avm_alu_ic));
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            Avm_DECLARE_VIEWS(19);

            auto tmp = (avm_alu_op_not * avm_alu_ff_tag);
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            Avm_DECLARE_VIEWS(20);

            auto tmp = (avm_alu_op_not *
                        ((avm_alu_ia + avm_alu_ic) - ((((((avm_alu_u8_tag * FF(256)) + (avm_alu_u16_tag * FF(65536))) +
                                                         (avm_alu_u32_tag * FF(4294967296UL))) +
                                                        (avm_alu_u64_tag * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                                       (avm_alu_u128_tag * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) -
                                                      FF(1))));
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
        // Contribution 21
        {
            Avm_DECLARE_VIEWS(21);

            auto tmp = ((avm_alu_cmp_sel + avm_alu_op_eq) * (avm_alu_ic * (-avm_alu_ic + FF(1))));
            tmp *= scaling_factor;
            std::get<21>(evals) += tmp;
        }
        // Contribution 22
        {
            Avm_DECLARE_VIEWS(22);

            auto tmp =
                (avm_alu_op_eq * ((((avm_alu_ia - avm_alu_ib) *
                                    ((avm_alu_ic * (-avm_alu_op_eq_diff_inv + FF(1))) + avm_alu_op_eq_diff_inv)) -
                                   FF(1)) +
                                  avm_alu_ic));
            tmp *= scaling_factor;
            std::get<22>(evals) += tmp;
        }
        // Contribution 23
        {
            Avm_DECLARE_VIEWS(23);

            auto tmp = (((avm_alu_op_lt * avm_alu_ib) + (avm_alu_op_lte * avm_alu_ia)) -
                        ((avm_alu_a_lo + (avm_alu_a_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) * avm_alu_cmp_sel));
            tmp *= scaling_factor;
            std::get<23>(evals) += tmp;
        }
        // Contribution 24
        {
            Avm_DECLARE_VIEWS(24);

            auto tmp = (((avm_alu_op_lt * avm_alu_ia) + (avm_alu_op_lte * avm_alu_ib)) -
                        ((avm_alu_b_lo + (avm_alu_b_hi * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) * avm_alu_cmp_sel));
            tmp *= scaling_factor;
            std::get<24>(evals) += tmp;
        }
        // Contribution 25
        {
            Avm_DECLARE_VIEWS(25);

            auto tmp = (avm_alu_p_a_borrow * (-avm_alu_p_a_borrow + FF(1)));
            tmp *= scaling_factor;
            std::get<25>(evals) += tmp;
        }
        // Contribution 26
        {
            Avm_DECLARE_VIEWS(26);

            auto tmp = ((avm_alu_p_sub_a_lo -
                         ((-avm_alu_a_lo + FF(uint256_t{ 4891460686036598784UL, 2896914383306846353UL, 0UL, 0UL })) +
                          (avm_alu_p_a_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))) *
                        avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<26>(evals) += tmp;
        }
        // Contribution 27
        {
            Avm_DECLARE_VIEWS(27);

            auto tmp = ((avm_alu_p_sub_a_hi -
                         ((-avm_alu_a_hi + FF(uint256_t{ 13281191951274694749UL, 3486998266802970665UL, 0UL, 0UL })) -
                          avm_alu_p_a_borrow)) *
                        avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<27>(evals) += tmp;
        }
        // Contribution 28
        {
            Avm_DECLARE_VIEWS(28);

            auto tmp = (avm_alu_p_b_borrow * (-avm_alu_p_b_borrow + FF(1)));
            tmp *= scaling_factor;
            std::get<28>(evals) += tmp;
        }
        // Contribution 29
        {
            Avm_DECLARE_VIEWS(29);

            auto tmp = ((avm_alu_p_sub_b_lo -
                         ((-avm_alu_b_lo + FF(uint256_t{ 4891460686036598784UL, 2896914383306846353UL, 0UL, 0UL })) +
                          (avm_alu_p_b_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL })))) *
                        avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<29>(evals) += tmp;
        }
        // Contribution 30
        {
            Avm_DECLARE_VIEWS(30);

            auto tmp = ((avm_alu_p_sub_b_hi -
                         ((-avm_alu_b_hi + FF(uint256_t{ 13281191951274694749UL, 3486998266802970665UL, 0UL, 0UL })) -
                          avm_alu_p_b_borrow)) *
                        avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<30>(evals) += tmp;
        }
        // Contribution 31
        {
            Avm_DECLARE_VIEWS(31);

            auto tmp =
                ((avm_alu_res_lo -
                  (((((avm_alu_a_lo - avm_alu_b_lo) - FF(1)) + (avm_alu_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                    ((avm_alu_op_lt * avm_alu_ic) + ((-avm_alu_ic + FF(1)) * avm_alu_op_lte))) +
                   (((avm_alu_b_lo - avm_alu_a_lo) + (avm_alu_borrow * FF(uint256_t{ 0UL, 0UL, 1UL, 0UL }))) *
                    (-((avm_alu_op_lt * avm_alu_ic) + ((-avm_alu_ic + FF(1)) * avm_alu_op_lte)) + FF(1))))) *
                 avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<31>(evals) += tmp;
        }
        // Contribution 32
        {
            Avm_DECLARE_VIEWS(32);

            auto tmp = ((avm_alu_res_hi -
                         ((((avm_alu_a_hi - avm_alu_b_hi) - avm_alu_borrow) *
                           ((avm_alu_op_lt * avm_alu_ic) + ((-avm_alu_ic + FF(1)) * avm_alu_op_lte))) +
                          (((avm_alu_b_hi - avm_alu_a_hi) - avm_alu_borrow) *
                           (-((avm_alu_op_lt * avm_alu_ic) + ((-avm_alu_ic + FF(1)) * avm_alu_op_lte)) + FF(1))))) *
                        avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<32>(evals) += tmp;
        }
        // Contribution 33
        {
            Avm_DECLARE_VIEWS(33);

            auto tmp = (((avm_alu_cmp_rng_ctr_shift - avm_alu_cmp_rng_ctr) + FF(1)) * avm_alu_cmp_rng_ctr);
            tmp *= scaling_factor;
            std::get<33>(evals) += tmp;
        }
        // Contribution 34
        {
            Avm_DECLARE_VIEWS(34);

            auto tmp = ((avm_alu_cmp_rng_ctr_shift - FF(4)) * avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<34>(evals) += tmp;
        }
        // Contribution 35
        {
            Avm_DECLARE_VIEWS(35);

            auto tmp = (avm_alu_rng_chk_sel * (-avm_alu_rng_chk_sel + FF(1)));
            tmp *= scaling_factor;
            std::get<35>(evals) += tmp;
        }
        // Contribution 36
        {
            Avm_DECLARE_VIEWS(36);

            auto tmp = (avm_alu_rng_chk_sel * avm_alu_cmp_sel);
            tmp *= scaling_factor;
            std::get<36>(evals) += tmp;
        }
        // Contribution 37
        {
            Avm_DECLARE_VIEWS(37);

            auto tmp = ((avm_alu_cmp_rng_ctr * (((-avm_alu_rng_chk_sel + FF(1)) * (-avm_alu_op_eq_diff_inv + FF(1))) +
                                                avm_alu_op_eq_diff_inv)) -
                        avm_alu_rng_chk_sel);
            tmp *= scaling_factor;
            std::get<37>(evals) += tmp;
        }
        // Contribution 38
        {
            Avm_DECLARE_VIEWS(38);

            auto tmp = (avm_alu_rng_chk_lookup_selector_shift -
                        (((((avm_alu_cmp_sel_shift + avm_alu_rng_chk_sel_shift) + avm_alu_op_add_shift) +
                           avm_alu_op_sub_shift) +
                          avm_alu_op_mul_shift) +
                         (avm_alu_op_mul * avm_alu_u128_tag)));
            tmp *= scaling_factor;
            std::get<38>(evals) += tmp;
        }
        // Contribution 39
        {
            Avm_DECLARE_VIEWS(39);

            auto tmp =
                (avm_alu_a_lo - (((((((((avm_alu_u8_r0 + (avm_alu_u8_r1 * FF(256))) + (avm_alu_u16_r0 * FF(65536))) +
                                       (avm_alu_u16_r1 * FF(4294967296UL))) +
                                      (avm_alu_u16_r2 * FF(281474976710656UL))) +
                                     (avm_alu_u16_r3 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                                    (avm_alu_u16_r4 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                                   (avm_alu_u16_r5 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                                  (avm_alu_u16_r6 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) *
                                 (avm_alu_rng_chk_sel + avm_alu_cmp_sel)));
            tmp *= scaling_factor;
            std::get<39>(evals) += tmp;
        }
        // Contribution 40
        {
            Avm_DECLARE_VIEWS(40);

            auto tmp = (avm_alu_a_hi -
                        ((((((((avm_alu_u16_r7 + (avm_alu_u16_r8 * FF(65536))) + (avm_alu_u16_r9 * FF(4294967296UL))) +
                              (avm_alu_u16_r10 * FF(281474976710656UL))) +
                             (avm_alu_u16_r11 * FF(uint256_t{ 0UL, 1UL, 0UL, 0UL }))) +
                            (avm_alu_u16_r12 * FF(uint256_t{ 0UL, 65536UL, 0UL, 0UL }))) +
                           (avm_alu_u16_r13 * FF(uint256_t{ 0UL, 4294967296UL, 0UL, 0UL }))) +
                          (avm_alu_u16_r14 * FF(uint256_t{ 0UL, 281474976710656UL, 0UL, 0UL }))) *
                         (avm_alu_rng_chk_sel + avm_alu_cmp_sel)));
            tmp *= scaling_factor;
            std::get<40>(evals) += tmp;
        }
        // Contribution 41
        {
            Avm_DECLARE_VIEWS(41);

            auto tmp = ((avm_alu_a_lo_shift - avm_alu_b_lo) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<41>(evals) += tmp;
        }
        // Contribution 42
        {
            Avm_DECLARE_VIEWS(42);

            auto tmp = ((avm_alu_a_hi_shift - avm_alu_b_hi) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<42>(evals) += tmp;
        }
        // Contribution 43
        {
            Avm_DECLARE_VIEWS(43);

            auto tmp = ((avm_alu_b_lo_shift - avm_alu_p_sub_a_lo) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<43>(evals) += tmp;
        }
        // Contribution 44
        {
            Avm_DECLARE_VIEWS(44);

            auto tmp = ((avm_alu_b_hi_shift - avm_alu_p_sub_a_hi) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<44>(evals) += tmp;
        }
        // Contribution 45
        {
            Avm_DECLARE_VIEWS(45);

            auto tmp = ((avm_alu_p_sub_a_lo_shift - avm_alu_p_sub_b_lo) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<45>(evals) += tmp;
        }
        // Contribution 46
        {
            Avm_DECLARE_VIEWS(46);

            auto tmp = ((avm_alu_p_sub_a_hi_shift - avm_alu_p_sub_b_hi) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<46>(evals) += tmp;
        }
        // Contribution 47
        {
            Avm_DECLARE_VIEWS(47);

            auto tmp = ((avm_alu_p_sub_b_lo_shift - avm_alu_res_lo) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<47>(evals) += tmp;
        }
        // Contribution 48
        {
            Avm_DECLARE_VIEWS(48);

            auto tmp = ((avm_alu_p_sub_b_hi_shift - avm_alu_res_hi) * avm_alu_rng_chk_sel_shift);
            tmp *= scaling_factor;
            std::get<48>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_alu = Relation<avm_aluImpl<FF>>;

} // namespace bb::Avm_vm