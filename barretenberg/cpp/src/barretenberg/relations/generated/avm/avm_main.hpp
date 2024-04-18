
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Avm_mainRow {
    FF avm_main_alu_in_tag{};
    FF avm_main_alu_sel{};
    FF avm_main_bin_op_id{};
    FF avm_main_bin_sel{};
    FF avm_main_first{};
    FF avm_main_ia{};
    FF avm_main_ib{};
    FF avm_main_ic{};
    FF avm_main_id{};
    FF avm_main_id_zero{};
    FF avm_main_ind_op_a{};
    FF avm_main_ind_op_b{};
    FF avm_main_ind_op_c{};
    FF avm_main_ind_op_d{};
    FF avm_main_internal_return_ptr{};
    FF avm_main_internal_return_ptr_shift{};
    FF avm_main_inv{};
    FF avm_main_mem_idx_a{};
    FF avm_main_mem_idx_b{};
    FF avm_main_mem_op_a{};
    FF avm_main_mem_op_b{};
    FF avm_main_mem_op_c{};
    FF avm_main_mem_op_d{};
    FF avm_main_op_err{};
    FF avm_main_pc{};
    FF avm_main_pc_shift{};
    FF avm_main_r_in_tag{};
    FF avm_main_rwa{};
    FF avm_main_rwb{};
    FF avm_main_rwc{};
    FF avm_main_rwd{};
    FF avm_main_sel_cmov{};
    FF avm_main_sel_halt{};
    FF avm_main_sel_internal_call{};
    FF avm_main_sel_internal_return{};
    FF avm_main_sel_jump{};
    FF avm_main_sel_mov{};
    FF avm_main_sel_mov_a{};
    FF avm_main_sel_mov_b{};
    FF avm_main_sel_op_add{};
    FF avm_main_sel_op_and{};
    FF avm_main_sel_op_cast{};
    FF avm_main_sel_op_div{};
    FF avm_main_sel_op_eq{};
    FF avm_main_sel_op_lt{};
    FF avm_main_sel_op_lte{};
    FF avm_main_sel_op_mul{};
    FF avm_main_sel_op_not{};
    FF avm_main_sel_op_or{};
    FF avm_main_sel_op_sub{};
    FF avm_main_sel_op_xor{};
    FF avm_main_tag_err{};
    FF avm_main_w_in_tag{};
};

inline std::string get_relation_label_avm_main(int index)
{
    switch (index) {
    case 33:
        return "OUTPUT_U8";

    case 34:
        return "SUBOP_DIVISION_FF";

    case 35:
        return "SUBOP_DIVISION_ZERO_ERR1";

    case 36:
        return "SUBOP_DIVISION_ZERO_ERR2";

    case 37:
        return "SUBOP_ERROR_RELEVANT_OP";

    case 39:
        return "RETURN_POINTER_INCREMENT";

    case 45:
        return "RETURN_POINTER_DECREMENT";

    case 50:
        return "PC_INCREMENT";

    case 51:
        return "INTERNAL_RETURN_POINTER_CONSISTENCY";

    case 52:
        return "CMOV_CONDITION_RES_1";

    case 53:
        return "CMOV_CONDITION_RES_2";

    case 56:
        return "MOV_SAME_VALUE_A";

    case 57:
        return "MOV_SAME_VALUE_B";

    case 58:
        return "MOV_MAIN_SAME_TAG";

    case 62:
        return "BIN_SEL_1";

    case 63:
        return "BIN_SEL_2";
    }
    return std::to_string(index);
}

template <typename FF_> class avm_mainImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 64> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 5, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 5, 3, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2,
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

            auto tmp = (avm_main_sel_op_add * (-avm_main_sel_op_add + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);

            auto tmp = (avm_main_sel_op_sub * (-avm_main_sel_op_sub + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);

            auto tmp = (avm_main_sel_op_mul * (-avm_main_sel_op_mul + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);

            auto tmp = (avm_main_sel_op_div * (-avm_main_sel_op_div + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);

            auto tmp = (avm_main_sel_op_not * (-avm_main_sel_op_not + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);

            auto tmp = (avm_main_sel_op_eq * (-avm_main_sel_op_eq + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);

            auto tmp = (avm_main_sel_op_and * (-avm_main_sel_op_and + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);

            auto tmp = (avm_main_sel_op_or * (-avm_main_sel_op_or + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);

            auto tmp = (avm_main_sel_op_xor * (-avm_main_sel_op_xor + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);

            auto tmp = (avm_main_sel_op_cast * (-avm_main_sel_op_cast + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);

            auto tmp = (avm_main_sel_op_lt * (-avm_main_sel_op_lt + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            Avm_DECLARE_VIEWS(11);

            auto tmp = (avm_main_sel_op_lte * (-avm_main_sel_op_lte + FF(1)));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            Avm_DECLARE_VIEWS(12);

            auto tmp = (avm_main_sel_internal_call * (-avm_main_sel_internal_call + FF(1)));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            Avm_DECLARE_VIEWS(13);

            auto tmp = (avm_main_sel_internal_return * (-avm_main_sel_internal_return + FF(1)));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            Avm_DECLARE_VIEWS(14);

            auto tmp = (avm_main_sel_jump * (-avm_main_sel_jump + FF(1)));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            Avm_DECLARE_VIEWS(15);

            auto tmp = (avm_main_sel_halt * (-avm_main_sel_halt + FF(1)));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            Avm_DECLARE_VIEWS(16);

            auto tmp = (avm_main_sel_mov * (-avm_main_sel_mov + FF(1)));
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            Avm_DECLARE_VIEWS(17);

            auto tmp = (avm_main_sel_cmov * (-avm_main_sel_cmov + FF(1)));
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            Avm_DECLARE_VIEWS(18);

            auto tmp = (avm_main_op_err * (-avm_main_op_err + FF(1)));
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            Avm_DECLARE_VIEWS(19);

            auto tmp = (avm_main_tag_err * (-avm_main_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            Avm_DECLARE_VIEWS(20);

            auto tmp = (avm_main_id_zero * (-avm_main_id_zero + FF(1)));
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
        // Contribution 21
        {
            Avm_DECLARE_VIEWS(21);

            auto tmp = (avm_main_mem_op_a * (-avm_main_mem_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<21>(evals) += tmp;
        }
        // Contribution 22
        {
            Avm_DECLARE_VIEWS(22);

            auto tmp = (avm_main_mem_op_b * (-avm_main_mem_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<22>(evals) += tmp;
        }
        // Contribution 23
        {
            Avm_DECLARE_VIEWS(23);

            auto tmp = (avm_main_mem_op_c * (-avm_main_mem_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<23>(evals) += tmp;
        }
        // Contribution 24
        {
            Avm_DECLARE_VIEWS(24);

            auto tmp = (avm_main_mem_op_d * (-avm_main_mem_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<24>(evals) += tmp;
        }
        // Contribution 25
        {
            Avm_DECLARE_VIEWS(25);

            auto tmp = (avm_main_rwa * (-avm_main_rwa + FF(1)));
            tmp *= scaling_factor;
            std::get<25>(evals) += tmp;
        }
        // Contribution 26
        {
            Avm_DECLARE_VIEWS(26);

            auto tmp = (avm_main_rwb * (-avm_main_rwb + FF(1)));
            tmp *= scaling_factor;
            std::get<26>(evals) += tmp;
        }
        // Contribution 27
        {
            Avm_DECLARE_VIEWS(27);

            auto tmp = (avm_main_rwc * (-avm_main_rwc + FF(1)));
            tmp *= scaling_factor;
            std::get<27>(evals) += tmp;
        }
        // Contribution 28
        {
            Avm_DECLARE_VIEWS(28);

            auto tmp = (avm_main_rwd * (-avm_main_rwd + FF(1)));
            tmp *= scaling_factor;
            std::get<28>(evals) += tmp;
        }
        // Contribution 29
        {
            Avm_DECLARE_VIEWS(29);

            auto tmp = (avm_main_ind_op_a * (-avm_main_ind_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<29>(evals) += tmp;
        }
        // Contribution 30
        {
            Avm_DECLARE_VIEWS(30);

            auto tmp = (avm_main_ind_op_b * (-avm_main_ind_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<30>(evals) += tmp;
        }
        // Contribution 31
        {
            Avm_DECLARE_VIEWS(31);

            auto tmp = (avm_main_ind_op_c * (-avm_main_ind_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<31>(evals) += tmp;
        }
        // Contribution 32
        {
            Avm_DECLARE_VIEWS(32);

            auto tmp = (avm_main_ind_op_d * (-avm_main_ind_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<32>(evals) += tmp;
        }
        // Contribution 33
        {
            Avm_DECLARE_VIEWS(33);

            auto tmp =
                (((avm_main_sel_op_eq + avm_main_sel_op_lte) + avm_main_sel_op_lt) * (avm_main_w_in_tag - FF(1)));
            tmp *= scaling_factor;
            std::get<33>(evals) += tmp;
        }
        // Contribution 34
        {
            Avm_DECLARE_VIEWS(34);

            auto tmp =
                ((avm_main_sel_op_div * (-avm_main_op_err + FF(1))) * ((avm_main_ic * avm_main_ib) - avm_main_ia));
            tmp *= scaling_factor;
            std::get<34>(evals) += tmp;
        }
        // Contribution 35
        {
            Avm_DECLARE_VIEWS(35);

            auto tmp = (avm_main_sel_op_div * (((avm_main_ib * avm_main_inv) - FF(1)) + avm_main_op_err));
            tmp *= scaling_factor;
            std::get<35>(evals) += tmp;
        }
        // Contribution 36
        {
            Avm_DECLARE_VIEWS(36);

            auto tmp = ((avm_main_sel_op_div * avm_main_op_err) * (-avm_main_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<36>(evals) += tmp;
        }
        // Contribution 37
        {
            Avm_DECLARE_VIEWS(37);

            auto tmp = (avm_main_op_err * (avm_main_sel_op_div - FF(1)));
            tmp *= scaling_factor;
            std::get<37>(evals) += tmp;
        }
        // Contribution 38
        {
            Avm_DECLARE_VIEWS(38);

            auto tmp = (avm_main_sel_jump * (avm_main_pc_shift - avm_main_ia));
            tmp *= scaling_factor;
            std::get<38>(evals) += tmp;
        }
        // Contribution 39
        {
            Avm_DECLARE_VIEWS(39);

            auto tmp = (avm_main_sel_internal_call *
                        (avm_main_internal_return_ptr_shift - (avm_main_internal_return_ptr + FF(1))));
            tmp *= scaling_factor;
            std::get<39>(evals) += tmp;
        }
        // Contribution 40
        {
            Avm_DECLARE_VIEWS(40);

            auto tmp = (avm_main_sel_internal_call * (avm_main_internal_return_ptr - avm_main_mem_idx_b));
            tmp *= scaling_factor;
            std::get<40>(evals) += tmp;
        }
        // Contribution 41
        {
            Avm_DECLARE_VIEWS(41);

            auto tmp = (avm_main_sel_internal_call * (avm_main_pc_shift - avm_main_ia));
            tmp *= scaling_factor;
            std::get<41>(evals) += tmp;
        }
        // Contribution 42
        {
            Avm_DECLARE_VIEWS(42);

            auto tmp = (avm_main_sel_internal_call * ((avm_main_pc + FF(1)) - avm_main_ib));
            tmp *= scaling_factor;
            std::get<42>(evals) += tmp;
        }
        // Contribution 43
        {
            Avm_DECLARE_VIEWS(43);

            auto tmp = (avm_main_sel_internal_call * (avm_main_rwb - FF(1)));
            tmp *= scaling_factor;
            std::get<43>(evals) += tmp;
        }
        // Contribution 44
        {
            Avm_DECLARE_VIEWS(44);

            auto tmp = (avm_main_sel_internal_call * (avm_main_mem_op_b - FF(1)));
            tmp *= scaling_factor;
            std::get<44>(evals) += tmp;
        }
        // Contribution 45
        {
            Avm_DECLARE_VIEWS(45);

            auto tmp = (avm_main_sel_internal_return *
                        (avm_main_internal_return_ptr_shift - (avm_main_internal_return_ptr - FF(1))));
            tmp *= scaling_factor;
            std::get<45>(evals) += tmp;
        }
        // Contribution 46
        {
            Avm_DECLARE_VIEWS(46);

            auto tmp = (avm_main_sel_internal_return * ((avm_main_internal_return_ptr - FF(1)) - avm_main_mem_idx_a));
            tmp *= scaling_factor;
            std::get<46>(evals) += tmp;
        }
        // Contribution 47
        {
            Avm_DECLARE_VIEWS(47);

            auto tmp = (avm_main_sel_internal_return * (avm_main_pc_shift - avm_main_ia));
            tmp *= scaling_factor;
            std::get<47>(evals) += tmp;
        }
        // Contribution 48
        {
            Avm_DECLARE_VIEWS(48);

            auto tmp = (avm_main_sel_internal_return * avm_main_rwa);
            tmp *= scaling_factor;
            std::get<48>(evals) += tmp;
        }
        // Contribution 49
        {
            Avm_DECLARE_VIEWS(49);

            auto tmp = (avm_main_sel_internal_return * (avm_main_mem_op_a - FF(1)));
            tmp *= scaling_factor;
            std::get<49>(evals) += tmp;
        }
        // Contribution 50
        {
            Avm_DECLARE_VIEWS(50);

            auto tmp =
                ((((-avm_main_first + FF(1)) * (-avm_main_sel_halt + FF(1))) *
                  (((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_div) + avm_main_sel_op_mul) +
                        avm_main_sel_op_not) +
                       avm_main_sel_op_eq) +
                      avm_main_sel_op_and) +
                     avm_main_sel_op_or) +
                    avm_main_sel_op_xor) +
                   avm_main_sel_op_cast)) *
                 (avm_main_pc_shift - (avm_main_pc + FF(1))));
            tmp *= scaling_factor;
            std::get<50>(evals) += tmp;
        }
        // Contribution 51
        {
            Avm_DECLARE_VIEWS(51);

            auto tmp = ((-(((avm_main_first + avm_main_sel_internal_call) + avm_main_sel_internal_return) +
                           avm_main_sel_halt) +
                         FF(1)) *
                        (avm_main_internal_return_ptr_shift - avm_main_internal_return_ptr));
            tmp *= scaling_factor;
            std::get<51>(evals) += tmp;
        }
        // Contribution 52
        {
            Avm_DECLARE_VIEWS(52);

            auto tmp = (avm_main_sel_cmov * (((avm_main_id * avm_main_inv) - FF(1)) + avm_main_id_zero));
            tmp *= scaling_factor;
            std::get<52>(evals) += tmp;
        }
        // Contribution 53
        {
            Avm_DECLARE_VIEWS(53);

            auto tmp = ((avm_main_sel_cmov * avm_main_id_zero) * (-avm_main_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<53>(evals) += tmp;
        }
        // Contribution 54
        {
            Avm_DECLARE_VIEWS(54);

            auto tmp = (avm_main_sel_mov_a - (avm_main_sel_mov + (avm_main_sel_cmov * (-avm_main_id_zero + FF(1)))));
            tmp *= scaling_factor;
            std::get<54>(evals) += tmp;
        }
        // Contribution 55
        {
            Avm_DECLARE_VIEWS(55);

            auto tmp = (avm_main_sel_mov_b - (avm_main_sel_cmov * avm_main_id_zero));
            tmp *= scaling_factor;
            std::get<55>(evals) += tmp;
        }
        // Contribution 56
        {
            Avm_DECLARE_VIEWS(56);

            auto tmp = (avm_main_sel_mov_a * (avm_main_ia - avm_main_ic));
            tmp *= scaling_factor;
            std::get<56>(evals) += tmp;
        }
        // Contribution 57
        {
            Avm_DECLARE_VIEWS(57);

            auto tmp = (avm_main_sel_mov_b * (avm_main_ib - avm_main_ic));
            tmp *= scaling_factor;
            std::get<57>(evals) += tmp;
        }
        // Contribution 58
        {
            Avm_DECLARE_VIEWS(58);

            auto tmp = ((avm_main_sel_mov + avm_main_sel_cmov) * (avm_main_r_in_tag - avm_main_w_in_tag));
            tmp *= scaling_factor;
            std::get<58>(evals) += tmp;
        }
        // Contribution 59
        {
            Avm_DECLARE_VIEWS(59);

            auto tmp =
                (avm_main_alu_sel -
                 ((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) + avm_main_sel_op_not) +
                      avm_main_sel_op_eq) +
                     avm_main_sel_op_lt) +
                    avm_main_sel_op_lte) +
                   avm_main_sel_op_cast) *
                  (-avm_main_tag_err + FF(1))));
            tmp *= scaling_factor;
            std::get<59>(evals) += tmp;
        }
        // Contribution 60
        {
            Avm_DECLARE_VIEWS(60);

            auto tmp = (((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) + avm_main_sel_op_not) +
                           avm_main_sel_op_eq) +
                          avm_main_sel_op_lt) +
                         avm_main_sel_op_lte) *
                        (avm_main_alu_in_tag - avm_main_r_in_tag));
            tmp *= scaling_factor;
            std::get<60>(evals) += tmp;
        }
        // Contribution 61
        {
            Avm_DECLARE_VIEWS(61);

            auto tmp = (avm_main_sel_op_cast * (avm_main_alu_in_tag - avm_main_w_in_tag));
            tmp *= scaling_factor;
            std::get<61>(evals) += tmp;
        }
        // Contribution 62
        {
            Avm_DECLARE_VIEWS(62);

            auto tmp = (avm_main_bin_op_id - (avm_main_sel_op_or + (avm_main_sel_op_xor * FF(2))));
            tmp *= scaling_factor;
            std::get<62>(evals) += tmp;
        }
        // Contribution 63
        {
            Avm_DECLARE_VIEWS(63);

            auto tmp = (avm_main_bin_sel - ((avm_main_sel_op_and + avm_main_sel_op_or) + avm_main_sel_op_xor));
            tmp *= scaling_factor;
            std::get<63>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_main = Relation<avm_mainImpl<FF>>;

} // namespace bb::Avm_vm