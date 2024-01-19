
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::AvmMini_vm {

template <typename FF> struct Avm_miniRow {
    FF avmMini_rwa{};
    FF avmMini_mem_op_a{};
    FF avmMini_sel_op_mul{};
    FF avmMini_mem_op_c{};
    FF avmMini_internal_return_ptr_shift{};
    FF avmMini_sel_op_div{};
    FF avmMini_rwb{};
    FF avmMini_pc_shift{};
    FF avmMini_internal_return_ptr{};
    FF avmMini_sel_internal_call{};
    FF avmMini_ia{};
    FF avmMini_mem_idx_a{};
    FF avmMini_sel_op_add{};
    FF avmMini_mem_op_b{};
    FF avmMini_inv{};
    FF avmMini_tag_err{};
    FF avmMini_op_err{};
    FF avmMini_ib{};
    FF avmMini_pc{};
    FF avmMini_sel_internal_return{};
    FF avmMini_sel_jump{};
    FF avmMini_rwc{};
    FF avmMini_first{};
    FF avmMini_sel_halt{};
    FF avmMini_ic{};
    FF avmMini_mem_idx_b{};
    FF avmMini_sel_op_sub{};
};

inline std::string get_relation_label_avm_mini(int index)
{
    switch (index) {
    case 19:
        return "SUBOP_ADDITION_FF";

    case 21:
        return "SUBOP_MULTIPLICATION_FF";

    case 33:
        return "RETURN_POINTER_DECREMENT";

    case 39:
        return "INTERNAL_RETURN_POINTER_CONSISTENCY";

    case 27:
        return "RETURN_POINTER_INCREMENT";

    case 24:
        return "SUBOP_DIVISION_ZERO_ERR2";

    case 20:
        return "SUBOP_SUBTRACTION_FF";

    case 22:
        return "SUBOP_DIVISION_FF";

    case 25:
        return "SUBOP_ERROR_RELEVANT_OP";

    case 23:
        return "SUBOP_DIVISION_ZERO_ERR1";

    case 38:
        return "PC_INCREMENT";
    }
    return std::to_string(index);
}

template <typename FF_> class avm_miniImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 40> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 4, 5, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 5, 3,
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

            auto tmp = (avmMini_sel_op_add * (-avmMini_sel_op_add + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            AvmMini_DECLARE_VIEWS(1);

            auto tmp = (avmMini_sel_op_sub * (-avmMini_sel_op_sub + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            AvmMini_DECLARE_VIEWS(2);

            auto tmp = (avmMini_sel_op_mul * (-avmMini_sel_op_mul + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            AvmMini_DECLARE_VIEWS(3);

            auto tmp = (avmMini_sel_op_div * (-avmMini_sel_op_div + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            AvmMini_DECLARE_VIEWS(4);

            auto tmp = (avmMini_sel_internal_call * (-avmMini_sel_internal_call + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            AvmMini_DECLARE_VIEWS(5);

            auto tmp = (avmMini_sel_internal_return * (-avmMini_sel_internal_return + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            AvmMini_DECLARE_VIEWS(6);

            auto tmp = (avmMini_sel_jump * (-avmMini_sel_jump + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            AvmMini_DECLARE_VIEWS(7);

            auto tmp = (avmMini_sel_halt * (-avmMini_sel_halt + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            AvmMini_DECLARE_VIEWS(8);

            auto tmp = (avmMini_op_err * (-avmMini_op_err + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            AvmMini_DECLARE_VIEWS(9);

            auto tmp = (avmMini_tag_err * (-avmMini_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            AvmMini_DECLARE_VIEWS(10);

            auto tmp = (avmMini_mem_op_a * (-avmMini_mem_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            AvmMini_DECLARE_VIEWS(11);

            auto tmp = (avmMini_mem_op_b * (-avmMini_mem_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            AvmMini_DECLARE_VIEWS(12);

            auto tmp = (avmMini_mem_op_c * (-avmMini_mem_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            AvmMini_DECLARE_VIEWS(13);

            auto tmp = (avmMini_rwa * (-avmMini_rwa + FF(1)));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            AvmMini_DECLARE_VIEWS(14);

            auto tmp = (avmMini_rwb * (-avmMini_rwb + FF(1)));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            AvmMini_DECLARE_VIEWS(15);

            auto tmp = (avmMini_rwc * (-avmMini_rwc + FF(1)));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            AvmMini_DECLARE_VIEWS(16);

            auto tmp = (avmMini_tag_err * avmMini_ia);
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            AvmMini_DECLARE_VIEWS(17);

            auto tmp = (avmMini_tag_err * avmMini_ib);
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            AvmMini_DECLARE_VIEWS(18);

            auto tmp = (avmMini_tag_err * avmMini_ic);
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            AvmMini_DECLARE_VIEWS(19);

            auto tmp = (avmMini_sel_op_add * ((avmMini_ia + avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            AvmMini_DECLARE_VIEWS(20);

            auto tmp = (avmMini_sel_op_sub * ((avmMini_ia - avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
        // Contribution 21
        {
            AvmMini_DECLARE_VIEWS(21);

            auto tmp = (avmMini_sel_op_mul * ((avmMini_ia * avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<21>(evals) += tmp;
        }
        // Contribution 22
        {
            AvmMini_DECLARE_VIEWS(22);

            auto tmp = ((avmMini_sel_op_div * (-avmMini_op_err + FF(1))) * ((avmMini_ic * avmMini_ib) - avmMini_ia));
            tmp *= scaling_factor;
            std::get<22>(evals) += tmp;
        }
        // Contribution 23
        {
            AvmMini_DECLARE_VIEWS(23);

            auto tmp = (avmMini_sel_op_div * (((avmMini_ib * avmMini_inv) - FF(1)) + avmMini_op_err));
            tmp *= scaling_factor;
            std::get<23>(evals) += tmp;
        }
        // Contribution 24
        {
            AvmMini_DECLARE_VIEWS(24);

            auto tmp = ((avmMini_sel_op_div * avmMini_op_err) * (-avmMini_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<24>(evals) += tmp;
        }
        // Contribution 25
        {
            AvmMini_DECLARE_VIEWS(25);

            auto tmp = (avmMini_op_err * (avmMini_sel_op_div - FF(1)));
            tmp *= scaling_factor;
            std::get<25>(evals) += tmp;
        }
        // Contribution 26
        {
            AvmMini_DECLARE_VIEWS(26);

            auto tmp = (avmMini_sel_jump * (avmMini_pc_shift - avmMini_ia));
            tmp *= scaling_factor;
            std::get<26>(evals) += tmp;
        }
        // Contribution 27
        {
            AvmMini_DECLARE_VIEWS(27);

            auto tmp = (avmMini_sel_internal_call *
                        (avmMini_internal_return_ptr_shift - (avmMini_internal_return_ptr + FF(1))));
            tmp *= scaling_factor;
            std::get<27>(evals) += tmp;
        }
        // Contribution 28
        {
            AvmMini_DECLARE_VIEWS(28);

            auto tmp = (avmMini_sel_internal_call * (avmMini_internal_return_ptr - avmMini_mem_idx_b));
            tmp *= scaling_factor;
            std::get<28>(evals) += tmp;
        }
        // Contribution 29
        {
            AvmMini_DECLARE_VIEWS(29);

            auto tmp = (avmMini_sel_internal_call * (avmMini_pc_shift - avmMini_ia));
            tmp *= scaling_factor;
            std::get<29>(evals) += tmp;
        }
        // Contribution 30
        {
            AvmMini_DECLARE_VIEWS(30);

            auto tmp = (avmMini_sel_internal_call * ((avmMini_pc + FF(1)) - avmMini_ib));
            tmp *= scaling_factor;
            std::get<30>(evals) += tmp;
        }
        // Contribution 31
        {
            AvmMini_DECLARE_VIEWS(31);

            auto tmp = (avmMini_sel_internal_call * (avmMini_rwb - FF(1)));
            tmp *= scaling_factor;
            std::get<31>(evals) += tmp;
        }
        // Contribution 32
        {
            AvmMini_DECLARE_VIEWS(32);

            auto tmp = (avmMini_sel_internal_call * (avmMini_mem_op_b - FF(1)));
            tmp *= scaling_factor;
            std::get<32>(evals) += tmp;
        }
        // Contribution 33
        {
            AvmMini_DECLARE_VIEWS(33);

            auto tmp = (avmMini_sel_internal_return *
                        (avmMini_internal_return_ptr_shift - (avmMini_internal_return_ptr - FF(1))));
            tmp *= scaling_factor;
            std::get<33>(evals) += tmp;
        }
        // Contribution 34
        {
            AvmMini_DECLARE_VIEWS(34);

            auto tmp = (avmMini_sel_internal_return * ((avmMini_internal_return_ptr - FF(1)) - avmMini_mem_idx_a));
            tmp *= scaling_factor;
            std::get<34>(evals) += tmp;
        }
        // Contribution 35
        {
            AvmMini_DECLARE_VIEWS(35);

            auto tmp = (avmMini_sel_internal_return * (avmMini_pc_shift - avmMini_ia));
            tmp *= scaling_factor;
            std::get<35>(evals) += tmp;
        }
        // Contribution 36
        {
            AvmMini_DECLARE_VIEWS(36);

            auto tmp = (avmMini_sel_internal_return * avmMini_rwa);
            tmp *= scaling_factor;
            std::get<36>(evals) += tmp;
        }
        // Contribution 37
        {
            AvmMini_DECLARE_VIEWS(37);

            auto tmp = (avmMini_sel_internal_return * (avmMini_mem_op_a - FF(1)));
            tmp *= scaling_factor;
            std::get<37>(evals) += tmp;
        }
        // Contribution 38
        {
            AvmMini_DECLARE_VIEWS(38);

            auto tmp = ((((-avmMini_first + FF(1)) * (-avmMini_sel_halt + FF(1))) *
                         (((avmMini_sel_op_add + avmMini_sel_op_sub) + avmMini_sel_op_div) + avmMini_sel_op_mul)) *
                        (avmMini_pc_shift - (avmMini_pc + FF(1))));
            tmp *= scaling_factor;
            std::get<38>(evals) += tmp;
        }
        // Contribution 39
        {
            AvmMini_DECLARE_VIEWS(39);

            auto tmp =
                ((-(((avmMini_first + avmMini_sel_internal_call) + avmMini_sel_internal_return) + avmMini_sel_halt) +
                  FF(1)) *
                 (avmMini_internal_return_ptr_shift - avmMini_internal_return_ptr));
            tmp *= scaling_factor;
            std::get<39>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_mini = Relation<avm_miniImpl<FF>>;

} // namespace bb::AvmMini_vm