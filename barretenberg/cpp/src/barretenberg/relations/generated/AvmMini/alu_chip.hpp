
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::AvmMini_vm {

template <typename FF> struct Alu_chipRow {
    FF aluChip_alu_u16_r0{};
    FF aluChip_alu_u16_r3{};
    FF aluChip_alu_u16_r5{};
    FF aluChip_alu_u16_r1_shift{};
    FF aluChip_alu_u16_r0_shift{};
    FF aluChip_alu_op_add{};
    FF aluChip_alu_ia{};
    FF aluChip_alu_u16_r4_shift{};
    FF aluChip_alu_ib{};
    FF aluChip_alu_u16_r7{};
    FF aluChip_alu_u8_r0{};
    FF aluChip_alu_u16_r7_shift{};
    FF aluChip_alu_op_sub{};
    FF aluChip_alu_u16_r6{};
    FF aluChip_alu_u16_r5_shift{};
    FF aluChip_alu_op_mul{};
    FF aluChip_alu_u64_tag{};
    FF aluChip_alu_u16_r2_shift{};
    FF aluChip_alu_u64_r0{};
    FF aluChip_alu_ff_tag{};
    FF aluChip_alu_u32_tag{};
    FF aluChip_alu_u16_tag{};
    FF aluChip_alu_u16_r4{};
    FF aluChip_alu_u16_r6_shift{};
    FF aluChip_alu_u16_r2{};
    FF aluChip_alu_ic{};
    FF aluChip_alu_u8_tag{};
    FF aluChip_alu_cf{};
    FF aluChip_alu_u16_r3_shift{};
    FF aluChip_alu_u8_r1{};
    FF aluChip_alu_u16_r1{};
    FF aluChip_alu_u128_tag{};
};

inline std::string get_relation_label_alu_chip(int index)
{
    switch (index) {
    case 9:
        return "ALU_MUL_COMMON_1";

    case 8:
        return "ALU_MULTIPLICATION_FF";

    case 6:
        return "ALU_ADD_SUB_1";

    case 7:
        return "ALU_ADD_SUB_2";

    case 10:
        return "ALU_MUL_COMMON_2";

    case 13:
        return "ALU_MULTIPLICATION_OUT_U128";
    }
    return std::to_string(index);
}

template <typename FF_> class alu_chipImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 14> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 4, 5, 5, 5, 5, 6, 6, 8,
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {

        // Contribution 0
        {
            AvmMini_DECLARE_VIEWS(0);

            auto tmp = (aluChip_alu_ff_tag * (-aluChip_alu_ff_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            AvmMini_DECLARE_VIEWS(1);

            auto tmp = (aluChip_alu_u8_tag * (-aluChip_alu_u8_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            AvmMini_DECLARE_VIEWS(2);

            auto tmp = (aluChip_alu_u16_tag * (-aluChip_alu_u16_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            AvmMini_DECLARE_VIEWS(3);

            auto tmp = (aluChip_alu_u32_tag * (-aluChip_alu_u32_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            AvmMini_DECLARE_VIEWS(4);

            auto tmp = (aluChip_alu_u64_tag * (-aluChip_alu_u64_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            AvmMini_DECLARE_VIEWS(5);

            auto tmp = (aluChip_alu_u128_tag * (-aluChip_alu_u128_tag + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            AvmMini_DECLARE_VIEWS(6);

            auto tmp =
                (((aluChip_alu_op_add + aluChip_alu_op_sub) *
                  ((((((((((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))) +
                          (aluChip_alu_u16_r1 * FF(4294967296UL))) +
                         (aluChip_alu_u16_r2 * FF(281474976710656UL))) +
                        (aluChip_alu_u16_r3 * FF(uint256_t{ 0, 1, 0, 0 }))) +
                       (aluChip_alu_u16_r4 * FF(uint256_t{ 0, 65536, 0, 0 }))) +
                      (aluChip_alu_u16_r5 * FF(uint256_t{ 0, 4294967296, 0, 0 }))) +
                     (aluChip_alu_u16_r6 * FF(uint256_t{ 0, 281474976710656, 0, 0 }))) -
                    aluChip_alu_ia) +
                   (aluChip_alu_ff_tag * aluChip_alu_ic))) +
                 ((aluChip_alu_op_add - aluChip_alu_op_sub) *
                  ((aluChip_alu_cf * FF(uint256_t{ 0, 0, 1, 0 })) - aluChip_alu_ib)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            AvmMini_DECLARE_VIEWS(7);

            auto tmp =
                (((aluChip_alu_op_add + aluChip_alu_op_sub) *
                  (((((((aluChip_alu_u8_tag * aluChip_alu_u8_r0) +
                        (aluChip_alu_u16_tag * (aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))))) +
                       (aluChip_alu_u32_tag *
                        ((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))))) +
                      (aluChip_alu_u64_tag *
                       ((((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))) +
                         (aluChip_alu_u16_r1 * FF(4294967296UL))) +
                        (aluChip_alu_u16_r2 * FF(281474976710656UL))))) +
                     (aluChip_alu_u128_tag *
                      ((((((((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))) +
                            (aluChip_alu_u16_r1 * FF(4294967296UL))) +
                           (aluChip_alu_u16_r2 * FF(281474976710656UL))) +
                          (aluChip_alu_u16_r3 * FF(uint256_t{ 0, 1, 0, 0 }))) +
                         (aluChip_alu_u16_r4 * FF(uint256_t{ 0, 65536, 0, 0 }))) +
                        (aluChip_alu_u16_r5 * FF(uint256_t{ 0, 4294967296, 0, 0 }))) +
                       (aluChip_alu_u16_r6 * FF(uint256_t{ 0, 281474976710656, 0, 0 }))))) +
                    (aluChip_alu_ff_tag * aluChip_alu_ia)) -
                   aluChip_alu_ic)) +
                 ((aluChip_alu_ff_tag * (aluChip_alu_op_add - aluChip_alu_op_sub)) * aluChip_alu_ib));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            AvmMini_DECLARE_VIEWS(8);

            auto tmp =
                ((aluChip_alu_ff_tag * aluChip_alu_op_mul) * ((aluChip_alu_ia * aluChip_alu_ib) - aluChip_alu_ic));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            AvmMini_DECLARE_VIEWS(9);

            auto tmp =
                ((((-aluChip_alu_ff_tag + FF(1)) - aluChip_alu_u128_tag) * aluChip_alu_op_mul) *
                 (((((((((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))) +
                        (aluChip_alu_u16_r1 * FF(4294967296UL))) +
                       (aluChip_alu_u16_r2 * FF(281474976710656UL))) +
                      (aluChip_alu_u16_r3 * FF(uint256_t{ 0, 1, 0, 0 }))) +
                     (aluChip_alu_u16_r4 * FF(uint256_t{ 0, 65536, 0, 0 }))) +
                    (aluChip_alu_u16_r5 * FF(uint256_t{ 0, 4294967296, 0, 0 }))) +
                   (aluChip_alu_u16_r6 * FF(uint256_t{ 0, 281474976710656, 0, 0 }))) -
                  (aluChip_alu_ia * aluChip_alu_ib)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            AvmMini_DECLARE_VIEWS(10);

            auto tmp = (aluChip_alu_op_mul *
                        (((((aluChip_alu_u8_tag * aluChip_alu_u8_r0) +
                            (aluChip_alu_u16_tag * (aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))))) +
                           (aluChip_alu_u32_tag *
                            ((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))))) +
                          (aluChip_alu_u64_tag *
                           ((((aluChip_alu_u8_r0 + (aluChip_alu_u8_r1 * FF(256))) + (aluChip_alu_u16_r0 * FF(65536))) +
                             (aluChip_alu_u16_r1 * FF(4294967296UL))) +
                            (aluChip_alu_u16_r2 * FF(281474976710656UL))))) -
                         (((-aluChip_alu_ff_tag + FF(1)) - aluChip_alu_u128_tag) * aluChip_alu_ic)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            AvmMini_DECLARE_VIEWS(11);

            auto tmp = ((aluChip_alu_u128_tag * aluChip_alu_op_mul) *
                        (((((aluChip_alu_u16_r0 + (aluChip_alu_u16_r1 * FF(65536))) +
                            (aluChip_alu_u16_r2 * FF(4294967296UL))) +
                           (aluChip_alu_u16_r3 * FF(281474976710656UL))) +
                          ((((aluChip_alu_u16_r4 + (aluChip_alu_u16_r5 * FF(65536))) +
                             (aluChip_alu_u16_r6 * FF(4294967296UL))) +
                            (aluChip_alu_u16_r7 * FF(281474976710656UL))) *
                           FF(uint256_t{ 0, 1, 0, 0 }))) -
                         aluChip_alu_ia));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            AvmMini_DECLARE_VIEWS(12);

            auto tmp = ((aluChip_alu_u128_tag * aluChip_alu_op_mul) *
                        (((((aluChip_alu_u16_r0_shift + (aluChip_alu_u16_r1_shift * FF(65536))) +
                            (aluChip_alu_u16_r2_shift * FF(4294967296UL))) +
                           (aluChip_alu_u16_r3_shift * FF(281474976710656UL))) +
                          ((((aluChip_alu_u16_r4_shift + (aluChip_alu_u16_r5_shift * FF(65536))) +
                             (aluChip_alu_u16_r6_shift * FF(4294967296UL))) +
                            (aluChip_alu_u16_r7_shift * FF(281474976710656UL))) *
                           FF(uint256_t{ 0, 1, 0, 0 }))) -
                         aluChip_alu_ib));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            AvmMini_DECLARE_VIEWS(13);

            auto tmp = ((aluChip_alu_u128_tag * aluChip_alu_op_mul) *
                        ((((aluChip_alu_ia * (((aluChip_alu_u16_r0_shift + (aluChip_alu_u16_r1_shift * FF(65536))) +
                                               (aluChip_alu_u16_r2_shift * FF(4294967296UL))) +
                                              (aluChip_alu_u16_r3_shift * FF(281474976710656UL)))) +
                           (((((aluChip_alu_u16_r0 + (aluChip_alu_u16_r1 * FF(65536))) +
                               (aluChip_alu_u16_r2 * FF(4294967296UL))) +
                              (aluChip_alu_u16_r3 * FF(281474976710656UL))) *
                             (((aluChip_alu_u16_r4_shift + (aluChip_alu_u16_r5_shift * FF(65536))) +
                               (aluChip_alu_u16_r6_shift * FF(4294967296UL))) +
                              (aluChip_alu_u16_r7_shift * FF(281474976710656UL)))) *
                            FF(uint256_t{ 0, 1, 0, 0 }))) -
                          (((aluChip_alu_cf * FF(uint256_t{ 0, 1, 0, 0 })) + aluChip_alu_u64_r0) *
                           FF(uint256_t{ 0, 0, 1, 0 }))) -
                         aluChip_alu_ic));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
    }
};

template <typename FF> using alu_chip = Relation<alu_chipImpl<FF>>;

} // namespace bb::AvmMini_vm