
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Avm_aluRow {
    FF avm_alu_alu_u16_r7_shift{};
    FF avm_alu_alu_u8_tag{};
    FF avm_alu_alu_u128_tag{};
    FF avm_alu_alu_ib{};
    FF avm_alu_alu_u16_r4{};
    FF avm_alu_alu_u16_r0{};
    FF avm_alu_alu_u16_r1_shift{};
    FF avm_alu_alu_op_mul{};
    FF avm_alu_alu_u16_tag{};
    FF avm_alu_alu_u16_r2{};
    FF avm_alu_alu_u16_r1{};
    FF avm_alu_alu_u16_r3{};
    FF avm_alu_alu_in_tag{};
    FF avm_alu_alu_u8_r1{};
    FF avm_alu_alu_u16_r7{};
    FF avm_alu_alu_u16_r6{};
    FF avm_alu_alu_op_eq{};
    FF avm_alu_alu_u64_tag{};
    FF avm_alu_alu_ia{};
    FF avm_alu_alu_ic{};
    FF avm_alu_alu_u16_r5_shift{};
    FF avm_alu_alu_op_eq_diff_inv{};
    FF avm_alu_alu_op_add{};
    FF avm_alu_alu_ff_tag{};
    FF avm_alu_alu_u16_r5{};
    FF avm_alu_alu_u32_tag{};
    FF avm_alu_alu_u16_r6_shift{};
    FF avm_alu_alu_op_sub{};
    FF avm_alu_alu_u16_r0_shift{};
    FF avm_alu_alu_u16_r4_shift{};
    FF avm_alu_alu_u8_r0{};
    FF avm_alu_alu_op_not{};
    FF avm_alu_alu_u16_r2_shift{};
    FF avm_alu_alu_u16_r3_shift{};
    FF avm_alu_alu_u64_r0{};
    FF avm_alu_alu_sel{};
    FF avm_alu_alu_cf{};
};

inline std::string get_relation_label_avm_alu(int index)
{
    switch (index) {
    case 19:
        return "ALU_RES_IS_BOOL";

    case 20:
        return "ALU_OP_EQ";

    case 13:
        return "ALU_MUL_COMMON_2";

    case 11:
        return "ALU_MULTIPLICATION_FF";

    case 10:
        return "ALU_ADD_SUB_2";

    case 16:
        return "ALU_MULTIPLICATION_OUT_U128";

    case 18:
        return "ALU_OP_NOT";

    case 9:
        return "ALU_ADD_SUB_1";

    case 17:
        return "ALU_FF_NOT_XOR";

    case 12:
        return "ALU_MUL_COMMON_1";
    }
    return std::to_string(index);
}

template <typename FF_> class avm_aluImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 21> SUBRELATION_PARTIAL_LENGTHS{
        2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 5, 5, 5, 5, 6, 6, 8, 3, 4, 4, 5,
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
                        ((((avm_alu_alu_op_add + avm_alu_alu_op_sub) + avm_alu_alu_op_mul) + avm_alu_alu_op_not) +
                         avm_alu_alu_op_eq));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);

            auto tmp = (avm_alu_alu_ff_tag * (-avm_alu_alu_ff_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);

            auto tmp = (avm_alu_alu_u8_tag * (-avm_alu_alu_u8_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);

            auto tmp = (avm_alu_alu_u16_tag * (-avm_alu_alu_u16_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);

            auto tmp = (avm_alu_alu_u32_tag * (-avm_alu_alu_u32_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);

            auto tmp = (avm_alu_alu_u64_tag * (-avm_alu_alu_u64_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);

            auto tmp = (avm_alu_alu_u128_tag * (-avm_alu_alu_u128_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);

            auto tmp = (avm_alu_alu_sel *
                        ((((((avm_alu_alu_ff_tag + avm_alu_alu_u8_tag) + avm_alu_alu_u16_tag) + avm_alu_alu_u32_tag) +
                           avm_alu_alu_u64_tag) +
                          avm_alu_alu_u128_tag) -
                         FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);

            auto tmp = (avm_alu_alu_in_tag -
                        (((((avm_alu_alu_u8_tag + (avm_alu_alu_u16_tag * FF(2))) + (avm_alu_alu_u32_tag * FF(3))) +
                           (avm_alu_alu_u64_tag * FF(4))) +
                          (avm_alu_alu_u128_tag * FF(5))) +
                         (avm_alu_alu_ff_tag * FF(6))));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);

            auto tmp =
                (((avm_alu_alu_op_add + avm_alu_alu_op_sub) *
                  ((((((((((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))) +
                          (avm_alu_alu_u16_r1 * FF(4294967296UL))) +
                         (avm_alu_alu_u16_r2 * FF(281474976710656UL))) +
                        (avm_alu_alu_u16_r3 * FF(uint256_t{ 0, 1, 0, 0 }))) +
                       (avm_alu_alu_u16_r4 * FF(uint256_t{ 0, 65536, 0, 0 }))) +
                      (avm_alu_alu_u16_r5 * FF(uint256_t{ 0, 4294967296, 0, 0 }))) +
                     (avm_alu_alu_u16_r6 * FF(uint256_t{ 0, 281474976710656, 0, 0 }))) -
                    avm_alu_alu_ia) +
                   (avm_alu_alu_ff_tag * avm_alu_alu_ic))) +
                 ((avm_alu_alu_op_add - avm_alu_alu_op_sub) *
                  ((avm_alu_alu_cf * FF(uint256_t{ 0, 0, 1, 0 })) - avm_alu_alu_ib)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);

            auto tmp =
                (((avm_alu_alu_op_add + avm_alu_alu_op_sub) *
                  (((((((avm_alu_alu_u8_tag * avm_alu_alu_u8_r0) +
                        (avm_alu_alu_u16_tag * (avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))))) +
                       (avm_alu_alu_u32_tag *
                        ((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))))) +
                      (avm_alu_alu_u64_tag *
                       ((((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))) +
                         (avm_alu_alu_u16_r1 * FF(4294967296UL))) +
                        (avm_alu_alu_u16_r2 * FF(281474976710656UL))))) +
                     (avm_alu_alu_u128_tag *
                      ((((((((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))) +
                            (avm_alu_alu_u16_r1 * FF(4294967296UL))) +
                           (avm_alu_alu_u16_r2 * FF(281474976710656UL))) +
                          (avm_alu_alu_u16_r3 * FF(uint256_t{ 0, 1, 0, 0 }))) +
                         (avm_alu_alu_u16_r4 * FF(uint256_t{ 0, 65536, 0, 0 }))) +
                        (avm_alu_alu_u16_r5 * FF(uint256_t{ 0, 4294967296, 0, 0 }))) +
                       (avm_alu_alu_u16_r6 * FF(uint256_t{ 0, 281474976710656, 0, 0 }))))) +
                    (avm_alu_alu_ff_tag * avm_alu_alu_ia)) -
                   avm_alu_alu_ic)) +
                 ((avm_alu_alu_ff_tag * (avm_alu_alu_op_add - avm_alu_alu_op_sub)) * avm_alu_alu_ib));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            Avm_DECLARE_VIEWS(11);

            auto tmp =
                ((avm_alu_alu_ff_tag * avm_alu_alu_op_mul) * ((avm_alu_alu_ia * avm_alu_alu_ib) - avm_alu_alu_ic));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            Avm_DECLARE_VIEWS(12);

            auto tmp =
                ((((-avm_alu_alu_ff_tag + FF(1)) - avm_alu_alu_u128_tag) * avm_alu_alu_op_mul) *
                 (((((((((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))) +
                        (avm_alu_alu_u16_r1 * FF(4294967296UL))) +
                       (avm_alu_alu_u16_r2 * FF(281474976710656UL))) +
                      (avm_alu_alu_u16_r3 * FF(uint256_t{ 0, 1, 0, 0 }))) +
                     (avm_alu_alu_u16_r4 * FF(uint256_t{ 0, 65536, 0, 0 }))) +
                    (avm_alu_alu_u16_r5 * FF(uint256_t{ 0, 4294967296, 0, 0 }))) +
                   (avm_alu_alu_u16_r6 * FF(uint256_t{ 0, 281474976710656, 0, 0 }))) -
                  (avm_alu_alu_ia * avm_alu_alu_ib)));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            Avm_DECLARE_VIEWS(13);

            auto tmp = (avm_alu_alu_op_mul *
                        (((((avm_alu_alu_u8_tag * avm_alu_alu_u8_r0) +
                            (avm_alu_alu_u16_tag * (avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))))) +
                           (avm_alu_alu_u32_tag *
                            ((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))))) +
                          (avm_alu_alu_u64_tag *
                           ((((avm_alu_alu_u8_r0 + (avm_alu_alu_u8_r1 * FF(256))) + (avm_alu_alu_u16_r0 * FF(65536))) +
                             (avm_alu_alu_u16_r1 * FF(4294967296UL))) +
                            (avm_alu_alu_u16_r2 * FF(281474976710656UL))))) -
                         (((-avm_alu_alu_ff_tag + FF(1)) - avm_alu_alu_u128_tag) * avm_alu_alu_ic)));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            Avm_DECLARE_VIEWS(14);

            auto tmp = ((avm_alu_alu_u128_tag * avm_alu_alu_op_mul) *
                        (((((avm_alu_alu_u16_r0 + (avm_alu_alu_u16_r1 * FF(65536))) +
                            (avm_alu_alu_u16_r2 * FF(4294967296UL))) +
                           (avm_alu_alu_u16_r3 * FF(281474976710656UL))) +
                          ((((avm_alu_alu_u16_r4 + (avm_alu_alu_u16_r5 * FF(65536))) +
                             (avm_alu_alu_u16_r6 * FF(4294967296UL))) +
                            (avm_alu_alu_u16_r7 * FF(281474976710656UL))) *
                           FF(uint256_t{ 0, 1, 0, 0 }))) -
                         avm_alu_alu_ia));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            Avm_DECLARE_VIEWS(15);

            auto tmp = ((avm_alu_alu_u128_tag * avm_alu_alu_op_mul) *
                        (((((avm_alu_alu_u16_r0_shift + (avm_alu_alu_u16_r1_shift * FF(65536))) +
                            (avm_alu_alu_u16_r2_shift * FF(4294967296UL))) +
                           (avm_alu_alu_u16_r3_shift * FF(281474976710656UL))) +
                          ((((avm_alu_alu_u16_r4_shift + (avm_alu_alu_u16_r5_shift * FF(65536))) +
                             (avm_alu_alu_u16_r6_shift * FF(4294967296UL))) +
                            (avm_alu_alu_u16_r7_shift * FF(281474976710656UL))) *
                           FF(uint256_t{ 0, 1, 0, 0 }))) -
                         avm_alu_alu_ib));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            Avm_DECLARE_VIEWS(16);

            auto tmp = ((avm_alu_alu_u128_tag * avm_alu_alu_op_mul) *
                        ((((avm_alu_alu_ia * (((avm_alu_alu_u16_r0_shift + (avm_alu_alu_u16_r1_shift * FF(65536))) +
                                               (avm_alu_alu_u16_r2_shift * FF(4294967296UL))) +
                                              (avm_alu_alu_u16_r3_shift * FF(281474976710656UL)))) +
                           (((((avm_alu_alu_u16_r0 + (avm_alu_alu_u16_r1 * FF(65536))) +
                               (avm_alu_alu_u16_r2 * FF(4294967296UL))) +
                              (avm_alu_alu_u16_r3 * FF(281474976710656UL))) *
                             (((avm_alu_alu_u16_r4_shift + (avm_alu_alu_u16_r5_shift * FF(65536))) +
                               (avm_alu_alu_u16_r6_shift * FF(4294967296UL))) +
                              (avm_alu_alu_u16_r7_shift * FF(281474976710656UL)))) *
                            FF(uint256_t{ 0, 1, 0, 0 }))) -
                          (((avm_alu_alu_cf * FF(uint256_t{ 0, 1, 0, 0 })) + avm_alu_alu_u64_r0) *
                           FF(uint256_t{ 0, 0, 1, 0 }))) -
                         avm_alu_alu_ic));
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            Avm_DECLARE_VIEWS(17);

            auto tmp = (avm_alu_alu_op_not * avm_alu_alu_ff_tag);
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            Avm_DECLARE_VIEWS(18);

            auto tmp = (avm_alu_alu_op_not * ((avm_alu_alu_ia + avm_alu_alu_ic) -
                                              ((((((avm_alu_alu_u8_tag * FF(256)) + (avm_alu_alu_u16_tag * FF(65536))) +
                                                  (avm_alu_alu_u32_tag * FF(4294967296UL))) +
                                                 (avm_alu_alu_u64_tag * FF(uint256_t{ 0, 1, 0, 0 }))) +
                                                (avm_alu_alu_u128_tag * FF(uint256_t{ 0, 0, 1, 0 }))) -
                                               FF(1))));
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            Avm_DECLARE_VIEWS(19);

            auto tmp = (avm_alu_alu_op_eq * (avm_alu_alu_ic * (-avm_alu_alu_ic + FF(1))));
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            Avm_DECLARE_VIEWS(20);

            auto tmp = (avm_alu_alu_op_eq *
                        ((((avm_alu_alu_ia - avm_alu_alu_ib) *
                           ((avm_alu_alu_ic * (-avm_alu_alu_op_eq_diff_inv + FF(1))) + avm_alu_alu_op_eq_diff_inv)) -
                          FF(1)) +
                         avm_alu_alu_ic));
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_alu = Relation<avm_aluImpl<FF>>;

} // namespace bb::Avm_vm