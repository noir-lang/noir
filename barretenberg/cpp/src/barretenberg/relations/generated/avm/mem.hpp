#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct MemRow {
    FF main_sel_first{};
    FF mem_addr{};
    FF mem_clk{};
    FF mem_diff_hi{};
    FF mem_diff_lo{};
    FF mem_diff_mid{};
    FF mem_glob_addr{};
    FF mem_glob_addr_shift{};
    FF mem_last{};
    FF mem_lastAccess{};
    FF mem_one_min_inv{};
    FF mem_r_in_tag{};
    FF mem_rw{};
    FF mem_rw_shift{};
    FF mem_sel_mem{};
    FF mem_sel_mem_shift{};
    FF mem_sel_mov_ia_to_ic{};
    FF mem_sel_mov_ib_to_ic{};
    FF mem_sel_op_a{};
    FF mem_sel_op_b{};
    FF mem_sel_op_c{};
    FF mem_sel_op_cmov{};
    FF mem_sel_op_d{};
    FF mem_sel_op_slice{};
    FF mem_sel_resolve_ind_addr_a{};
    FF mem_sel_resolve_ind_addr_b{};
    FF mem_sel_resolve_ind_addr_c{};
    FF mem_sel_resolve_ind_addr_d{};
    FF mem_sel_rng_chk{};
    FF mem_skip_check_tag{};
    FF mem_space_id{};
    FF mem_tag{};
    FF mem_tag_err{};
    FF mem_tag_shift{};
    FF mem_tsp{};
    FF mem_tsp_shift{};
    FF mem_val{};
    FF mem_val_shift{};
    FF mem_w_in_tag{};
};

inline std::string get_relation_label_mem(int index)
{
    switch (index) {
    case 15:
        return "MEM_CONTIGUOUS";
    case 16:
        return "MEM_FIRST_EMPTY";
    case 17:
        return "MEM_LAST";
    case 19:
        return "TIMESTAMP";
    case 20:
        return "GLOBAL_ADDR";
    case 21:
        return "LAST_ACCESS_FIRST_ROW";
    case 22:
        return "MEM_LAST_ACCESS_DELIMITER";
    case 23:
        return "DIFF_RNG_CHK_DEC";
    case 24:
        return "MEM_READ_WRITE_VAL_CONSISTENCY";
    case 25:
        return "MEM_READ_WRITE_TAG_CONSISTENCY";
    case 26:
        return "MEM_ZERO_INIT";
    case 27:
        return "SKIP_CHECK_TAG";
    case 28:
        return "MEM_IN_TAG_CONSISTENCY_1";
    case 29:
        return "MEM_IN_TAG_CONSISTENCY_2";
    case 30:
        return "NO_TAG_ERR_WRITE_OR_SKIP";
    case 32:
        return "NO_TAG_ERR_WRITE";
    case 43:
        return "MOV_SAME_TAG";
    }
    return std::to_string(index);
}

template <typename FF_> class memImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 44> SUBRELATION_PARTIAL_LENGTHS = { 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 3,
                                                                            4, 3, 4, 3, 4, 3, 3, 3, 4, 4, 4, 4, 4, 6, 4,
                                                                            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        // Contribution 0
        {
            Avm_DECLARE_VIEWS(0);
            auto tmp = (mem_lastAccess * (-mem_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);
            auto tmp = (mem_last * (-mem_last + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);
            auto tmp = (mem_rw * (-mem_rw + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);
            auto tmp = (mem_tag_err * (-mem_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);
            auto tmp = (mem_sel_op_a * (-mem_sel_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);
            auto tmp = (mem_sel_op_b * (-mem_sel_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);
            auto tmp = (mem_sel_op_c * (-mem_sel_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);
            auto tmp = (mem_sel_op_d * (-mem_sel_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);
            auto tmp = (mem_sel_op_slice * (-mem_sel_op_slice + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);
            auto tmp = (mem_sel_resolve_ind_addr_a * (-mem_sel_resolve_ind_addr_a + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);
            auto tmp = (mem_sel_resolve_ind_addr_b * (-mem_sel_resolve_ind_addr_b + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            Avm_DECLARE_VIEWS(11);
            auto tmp = (mem_sel_resolve_ind_addr_c * (-mem_sel_resolve_ind_addr_c + FF(1)));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            Avm_DECLARE_VIEWS(12);
            auto tmp = (mem_sel_resolve_ind_addr_d * (-mem_sel_resolve_ind_addr_d + FF(1)));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            Avm_DECLARE_VIEWS(13);
            auto tmp =
                (mem_sel_mem -
                 ((((((((mem_sel_op_a + mem_sel_op_b) + mem_sel_op_c) + mem_sel_op_d) + mem_sel_resolve_ind_addr_a) +
                     mem_sel_resolve_ind_addr_b) +
                    mem_sel_resolve_ind_addr_c) +
                   mem_sel_resolve_ind_addr_d) +
                  mem_sel_op_slice));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            Avm_DECLARE_VIEWS(14);
            auto tmp = (mem_sel_mem * (mem_sel_mem - FF(1)));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            Avm_DECLARE_VIEWS(15);
            auto tmp = (((-main_sel_first + FF(1)) * mem_sel_mem_shift) * (-mem_sel_mem + FF(1)));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            Avm_DECLARE_VIEWS(16);
            auto tmp = (main_sel_first * mem_sel_mem);
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            Avm_DECLARE_VIEWS(17);
            auto tmp = (((-mem_last + FF(1)) * mem_sel_mem) * (-mem_sel_mem_shift + FF(1)));
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            Avm_DECLARE_VIEWS(18);
            auto tmp = (mem_sel_rng_chk - (mem_sel_mem * (-mem_last + FF(1))));
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            Avm_DECLARE_VIEWS(19);
            auto tmp =
                (mem_tsp -
                 ((mem_clk * FF(12)) +
                  (mem_sel_mem *
                   ((((mem_sel_resolve_ind_addr_b + mem_sel_op_b) +
                      ((mem_sel_resolve_ind_addr_c + mem_sel_op_c) * FF(2))) +
                     ((mem_sel_resolve_ind_addr_d + mem_sel_op_d) * FF(3))) +
                    (((-(((mem_sel_resolve_ind_addr_a + mem_sel_resolve_ind_addr_b) + mem_sel_resolve_ind_addr_c) +
                         mem_sel_resolve_ind_addr_d) +
                       FF(1)) +
                      mem_rw) *
                     FF(4))))));
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            Avm_DECLARE_VIEWS(20);
            auto tmp = (mem_glob_addr - ((mem_space_id * FF(4294967296UL)) + mem_addr));
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
        // Contribution 21
        {
            Avm_DECLARE_VIEWS(21);
            auto tmp = (main_sel_first * (-mem_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<21>(evals) += tmp;
        }
        // Contribution 22
        {
            Avm_DECLARE_VIEWS(22);
            auto tmp = ((-mem_lastAccess + FF(1)) * (mem_glob_addr_shift - mem_glob_addr));
            tmp *= scaling_factor;
            std::get<22>(evals) += tmp;
        }
        // Contribution 23
        {
            Avm_DECLARE_VIEWS(23);
            auto tmp = (mem_sel_rng_chk * (((((mem_lastAccess * (mem_glob_addr_shift - mem_glob_addr)) +
                                              ((-mem_lastAccess + FF(1)) * (mem_tsp_shift - mem_tsp))) -
                                             (mem_diff_hi * FF(4294967296UL))) -
                                            (mem_diff_mid * FF(65536))) -
                                           mem_diff_lo));
            tmp *= scaling_factor;
            std::get<23>(evals) += tmp;
        }
        // Contribution 24
        {
            Avm_DECLARE_VIEWS(24);
            auto tmp = (((-mem_lastAccess + FF(1)) * (-mem_rw_shift + FF(1))) * (mem_val_shift - mem_val));
            tmp *= scaling_factor;
            std::get<24>(evals) += tmp;
        }
        // Contribution 25
        {
            Avm_DECLARE_VIEWS(25);
            auto tmp = (((-mem_lastAccess + FF(1)) * (-mem_rw_shift + FF(1))) * (mem_tag_shift - mem_tag));
            tmp *= scaling_factor;
            std::get<25>(evals) += tmp;
        }
        // Contribution 26
        {
            Avm_DECLARE_VIEWS(26);
            auto tmp = ((mem_lastAccess * (-mem_rw_shift + FF(1))) * mem_val_shift);
            tmp *= scaling_factor;
            std::get<26>(evals) += tmp;
        }
        // Contribution 27
        {
            Avm_DECLARE_VIEWS(27);
            auto tmp = (mem_skip_check_tag -
                        ((mem_sel_op_cmov * ((mem_sel_op_d + (mem_sel_op_a * (-mem_sel_mov_ia_to_ic + FF(1)))) +
                                             (mem_sel_op_b * (-mem_sel_mov_ib_to_ic + FF(1))))) +
                         mem_sel_op_slice));
            tmp *= scaling_factor;
            std::get<27>(evals) += tmp;
        }
        // Contribution 28
        {
            Avm_DECLARE_VIEWS(28);
            auto tmp = (((mem_tag * (-mem_skip_check_tag + FF(1))) * (-mem_rw + FF(1))) *
                        (((mem_r_in_tag - mem_tag) * (-mem_one_min_inv + FF(1))) - mem_tag_err));
            tmp *= scaling_factor;
            std::get<28>(evals) += tmp;
        }
        // Contribution 29
        {
            Avm_DECLARE_VIEWS(29);
            auto tmp = ((mem_tag * (-mem_tag_err + FF(1))) * mem_one_min_inv);
            tmp *= scaling_factor;
            std::get<29>(evals) += tmp;
        }
        // Contribution 30
        {
            Avm_DECLARE_VIEWS(30);
            auto tmp = ((mem_skip_check_tag + mem_rw) * mem_tag_err);
            tmp *= scaling_factor;
            std::get<30>(evals) += tmp;
        }
        // Contribution 31
        {
            Avm_DECLARE_VIEWS(31);
            auto tmp = (mem_rw * (mem_w_in_tag - mem_tag));
            tmp *= scaling_factor;
            std::get<31>(evals) += tmp;
        }
        // Contribution 32
        {
            Avm_DECLARE_VIEWS(32);
            auto tmp = (mem_rw * mem_tag_err);
            tmp *= scaling_factor;
            std::get<32>(evals) += tmp;
        }
        // Contribution 33
        {
            Avm_DECLARE_VIEWS(33);
            auto tmp = (mem_sel_resolve_ind_addr_a * (mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<33>(evals) += tmp;
        }
        // Contribution 34
        {
            Avm_DECLARE_VIEWS(34);
            auto tmp = (mem_sel_resolve_ind_addr_b * (mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<34>(evals) += tmp;
        }
        // Contribution 35
        {
            Avm_DECLARE_VIEWS(35);
            auto tmp = (mem_sel_resolve_ind_addr_c * (mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<35>(evals) += tmp;
        }
        // Contribution 36
        {
            Avm_DECLARE_VIEWS(36);
            auto tmp = (mem_sel_resolve_ind_addr_d * (mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<36>(evals) += tmp;
        }
        // Contribution 37
        {
            Avm_DECLARE_VIEWS(37);
            auto tmp = (mem_sel_resolve_ind_addr_a * mem_rw);
            tmp *= scaling_factor;
            std::get<37>(evals) += tmp;
        }
        // Contribution 38
        {
            Avm_DECLARE_VIEWS(38);
            auto tmp = (mem_sel_resolve_ind_addr_b * mem_rw);
            tmp *= scaling_factor;
            std::get<38>(evals) += tmp;
        }
        // Contribution 39
        {
            Avm_DECLARE_VIEWS(39);
            auto tmp = (mem_sel_resolve_ind_addr_c * mem_rw);
            tmp *= scaling_factor;
            std::get<39>(evals) += tmp;
        }
        // Contribution 40
        {
            Avm_DECLARE_VIEWS(40);
            auto tmp = (mem_sel_resolve_ind_addr_d * mem_rw);
            tmp *= scaling_factor;
            std::get<40>(evals) += tmp;
        }
        // Contribution 41
        {
            Avm_DECLARE_VIEWS(41);
            auto tmp = (mem_sel_op_slice * (mem_w_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<41>(evals) += tmp;
        }
        // Contribution 42
        {
            Avm_DECLARE_VIEWS(42);
            auto tmp = (mem_sel_op_slice * (mem_r_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<42>(evals) += tmp;
        }
        // Contribution 43
        {
            Avm_DECLARE_VIEWS(43);
            auto tmp = ((mem_sel_mov_ia_to_ic + mem_sel_mov_ib_to_ic) * mem_tag_err);
            tmp *= scaling_factor;
            std::get<43>(evals) += tmp;
        }
    }
};

template <typename FF> using mem = Relation<memImpl<FF>>;

} // namespace bb::Avm_vm