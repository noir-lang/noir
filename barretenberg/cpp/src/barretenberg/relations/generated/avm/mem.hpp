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
        {
            using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_lastAccess * (-new_term.mem_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<1, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_last * (-new_term.mem_last + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<2, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_rw * (-new_term.mem_rw + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<3, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_tag_err * (-new_term.mem_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<4, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_a * (-new_term.mem_sel_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<5, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_b * (-new_term.mem_sel_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<6, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_c * (-new_term.mem_sel_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<7, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_d * (-new_term.mem_sel_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<8, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_slice * (-new_term.mem_sel_op_slice + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<9, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_a * (-new_term.mem_sel_resolve_ind_addr_a + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<10, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_b * (-new_term.mem_sel_resolve_ind_addr_b + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<11, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_c * (-new_term.mem_sel_resolve_ind_addr_c + FF(1)));
            tmp *= scaling_factor;
            std::get<11>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<12, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_d * (-new_term.mem_sel_resolve_ind_addr_d + FF(1)));
            tmp *= scaling_factor;
            std::get<12>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<13, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_mem -
                        ((((((((new_term.mem_sel_op_a + new_term.mem_sel_op_b) + new_term.mem_sel_op_c) +
                              new_term.mem_sel_op_d) +
                             new_term.mem_sel_resolve_ind_addr_a) +
                            new_term.mem_sel_resolve_ind_addr_b) +
                           new_term.mem_sel_resolve_ind_addr_c) +
                          new_term.mem_sel_resolve_ind_addr_d) +
                         new_term.mem_sel_op_slice));
            tmp *= scaling_factor;
            std::get<13>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<14, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_mem * (new_term.mem_sel_mem - FF(1)));
            tmp *= scaling_factor;
            std::get<14>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<15, ContainerOverSubrelations>;
            auto tmp =
                (((-new_term.main_sel_first + FF(1)) * new_term.mem_sel_mem_shift) * (-new_term.mem_sel_mem + FF(1)));
            tmp *= scaling_factor;
            std::get<15>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<16, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.mem_sel_mem);
            tmp *= scaling_factor;
            std::get<16>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<17, ContainerOverSubrelations>;
            auto tmp = (((-new_term.mem_last + FF(1)) * new_term.mem_sel_mem) * (-new_term.mem_sel_mem_shift + FF(1)));
            tmp *= scaling_factor;
            std::get<17>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<18, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_rng_chk - (new_term.mem_sel_mem * (-new_term.mem_last + FF(1))));
            tmp *= scaling_factor;
            std::get<18>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<19, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_tsp -
                        ((new_term.mem_clk * FF(12)) +
                         (new_term.mem_sel_mem *
                          ((((new_term.mem_sel_resolve_ind_addr_b + new_term.mem_sel_op_b) +
                             ((new_term.mem_sel_resolve_ind_addr_c + new_term.mem_sel_op_c) * FF(2))) +
                            ((new_term.mem_sel_resolve_ind_addr_d + new_term.mem_sel_op_d) * FF(3))) +
                           (((-(((new_term.mem_sel_resolve_ind_addr_a + new_term.mem_sel_resolve_ind_addr_b) +
                                 new_term.mem_sel_resolve_ind_addr_c) +
                                new_term.mem_sel_resolve_ind_addr_d) +
                              FF(1)) +
                             new_term.mem_rw) *
                            FF(4))))));
            tmp *= scaling_factor;
            std::get<19>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<20, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_glob_addr - ((new_term.mem_space_id * FF(4294967296UL)) + new_term.mem_addr));
            tmp *= scaling_factor;
            std::get<20>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<21, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * (-new_term.mem_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<21>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<22, ContainerOverSubrelations>;
            auto tmp = ((-new_term.mem_lastAccess + FF(1)) * (new_term.mem_glob_addr_shift - new_term.mem_glob_addr));
            tmp *= scaling_factor;
            std::get<22>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<23, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_rng_chk *
                        (((((new_term.mem_lastAccess * (new_term.mem_glob_addr_shift - new_term.mem_glob_addr)) +
                            ((-new_term.mem_lastAccess + FF(1)) * (new_term.mem_tsp_shift - new_term.mem_tsp))) -
                           (new_term.mem_diff_hi * FF(4294967296UL))) -
                          (new_term.mem_diff_mid * FF(65536))) -
                         new_term.mem_diff_lo));
            tmp *= scaling_factor;
            std::get<23>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<24, ContainerOverSubrelations>;
            auto tmp = (((-new_term.mem_lastAccess + FF(1)) * (-new_term.mem_rw_shift + FF(1))) *
                        (new_term.mem_val_shift - new_term.mem_val));
            tmp *= scaling_factor;
            std::get<24>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<25, ContainerOverSubrelations>;
            auto tmp = (((-new_term.mem_lastAccess + FF(1)) * (-new_term.mem_rw_shift + FF(1))) *
                        (new_term.mem_tag_shift - new_term.mem_tag));
            tmp *= scaling_factor;
            std::get<25>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<26, ContainerOverSubrelations>;
            auto tmp = ((new_term.mem_lastAccess * (-new_term.mem_rw_shift + FF(1))) * new_term.mem_val_shift);
            tmp *= scaling_factor;
            std::get<26>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<27, ContainerOverSubrelations>;
            auto tmp =
                (new_term.mem_skip_check_tag -
                 ((new_term.mem_sel_op_cmov *
                   ((new_term.mem_sel_op_d + (new_term.mem_sel_op_a * (-new_term.mem_sel_mov_ia_to_ic + FF(1)))) +
                    (new_term.mem_sel_op_b * (-new_term.mem_sel_mov_ib_to_ic + FF(1))))) +
                  new_term.mem_sel_op_slice));
            tmp *= scaling_factor;
            std::get<27>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<28, ContainerOverSubrelations>;
            auto tmp = (((new_term.mem_tag * (-new_term.mem_skip_check_tag + FF(1))) * (-new_term.mem_rw + FF(1))) *
                        (((new_term.mem_r_in_tag - new_term.mem_tag) * (-new_term.mem_one_min_inv + FF(1))) -
                         new_term.mem_tag_err));
            tmp *= scaling_factor;
            std::get<28>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<29, ContainerOverSubrelations>;
            auto tmp = ((new_term.mem_tag * (-new_term.mem_tag_err + FF(1))) * new_term.mem_one_min_inv);
            tmp *= scaling_factor;
            std::get<29>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<30, ContainerOverSubrelations>;
            auto tmp = ((new_term.mem_skip_check_tag + new_term.mem_rw) * new_term.mem_tag_err);
            tmp *= scaling_factor;
            std::get<30>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<31, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_rw * (new_term.mem_w_in_tag - new_term.mem_tag));
            tmp *= scaling_factor;
            std::get<31>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<32, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_rw * new_term.mem_tag_err);
            tmp *= scaling_factor;
            std::get<32>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<33, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_a * (new_term.mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<33>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<34, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_b * (new_term.mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<34>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<35, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_c * (new_term.mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<35>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<36, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_d * (new_term.mem_r_in_tag - FF(3)));
            tmp *= scaling_factor;
            std::get<36>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<37, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_a * new_term.mem_rw);
            tmp *= scaling_factor;
            std::get<37>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<38, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_b * new_term.mem_rw);
            tmp *= scaling_factor;
            std::get<38>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<39, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_c * new_term.mem_rw);
            tmp *= scaling_factor;
            std::get<39>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<40, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_resolve_ind_addr_d * new_term.mem_rw);
            tmp *= scaling_factor;
            std::get<40>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<41, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_slice * (new_term.mem_w_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<41>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<42, ContainerOverSubrelations>;
            auto tmp = (new_term.mem_sel_op_slice * (new_term.mem_r_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<42>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<43, ContainerOverSubrelations>;
            auto tmp = ((new_term.mem_sel_mov_ia_to_ic + new_term.mem_sel_mov_ib_to_ic) * new_term.mem_tag_err);
            tmp *= scaling_factor;
            std::get<43>(evals) += typename Accumulator::View(tmp);
        }
    }
};

template <typename FF> class mem : public Relation<memImpl<FF>> {
  public:
    static constexpr const char* NAME = "mem";

    static std::string get_subrelation_label(size_t index)
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
};

} // namespace bb::Avm_vm