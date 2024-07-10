#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct MemSliceRow {
    FF slice_addr{};
    FF slice_addr_shift{};
    FF slice_clk{};
    FF slice_clk_shift{};
    FF slice_cnt{};
    FF slice_cnt_shift{};
    FF slice_col_offset{};
    FF slice_col_offset_shift{};
    FF slice_one_min_inv{};
    FF slice_sel_cd_cpy{};
    FF slice_sel_cd_cpy_shift{};
    FF slice_sel_mem_active{};
    FF slice_sel_mem_active_shift{};
    FF slice_sel_return{};
    FF slice_sel_return_shift{};
    FF slice_sel_start_shift{};
    FF slice_space_id{};
    FF slice_space_id_shift{};
};

inline std::string get_relation_label_mem_slice(int index)
{
    switch (index) {
    case 1:
        return "SLICE_CNT_ZERO_TEST1";
    case 2:
        return "SLICE_CNT_ZERO_TEST2";
    case 3:
        return "SLICE_CNT_DECREMENT";
    case 4:
        return "ADDR_INCREMENT";
    case 5:
        return "COL_OFFSET_INCREMENT";
    case 6:
        return "SAME_CLK";
    case 7:
        return "SAME_SPACE_ID";
    case 8:
        return "SAME_SEL_RETURN";
    case 9:
        return "SAME_SEL_CD_CPY";
    case 10:
        return "SEL_MEM_INACTIVE";
    }
    return std::to_string(index);
}

template <typename FF_> class mem_sliceImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 11> SUBRELATION_PARTIAL_LENGTHS = { 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        // Contribution 0
        {
            Avm_DECLARE_VIEWS(0);
            auto tmp = (slice_sel_mem_active - (slice_sel_cd_cpy + slice_sel_return));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);
            auto tmp = ((slice_cnt * (-slice_one_min_inv + FF(1))) - slice_sel_mem_active);
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);
            auto tmp = ((-slice_sel_mem_active + FF(1)) * slice_one_min_inv);
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);
            auto tmp = (slice_sel_mem_active * ((slice_cnt - FF(1)) - slice_cnt_shift));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);
            auto tmp = (slice_sel_mem_active * ((slice_addr + FF(1)) - slice_addr_shift));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);
            auto tmp = (slice_sel_mem_active * ((slice_col_offset + FF(1)) - slice_col_offset_shift));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);
            auto tmp = (slice_sel_mem_active * (slice_clk - slice_clk_shift));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);
            auto tmp = (slice_sel_mem_active * (slice_space_id - slice_space_id_shift));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);
            auto tmp =
                ((slice_sel_mem_active * slice_sel_mem_active_shift) * (slice_sel_return - slice_sel_return_shift));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);
            auto tmp =
                ((slice_sel_mem_active * slice_sel_mem_active_shift) * (slice_sel_cd_cpy - slice_sel_cd_cpy_shift));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);
            auto tmp =
                (((-slice_sel_mem_active + FF(1)) * slice_sel_mem_active_shift) * (-slice_sel_start_shift + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
    }
};

template <typename FF> using mem_slice = Relation<mem_sliceImpl<FF>>;

} // namespace bb::Avm_vm