#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct GasRow {
    FF gas_da_gas_fixed_table{};
    FF gas_l2_gas_fixed_table{};
    FF gas_sel_gas_cost{};
};

inline std::string get_relation_label_gas(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class gasImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 3> SUBRELATION_PARTIAL_LENGTHS = { 2, 2, 2 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        // Contribution 0
        {
            Avm_DECLARE_VIEWS(0);
            auto tmp = (gas_sel_gas_cost - gas_sel_gas_cost);
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);
            auto tmp = (gas_l2_gas_fixed_table - gas_l2_gas_fixed_table);
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);
            auto tmp = (gas_da_gas_fixed_table - gas_da_gas_fixed_table);
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
    }
};

template <typename FF> using gas = Relation<gasImpl<FF>>;

} // namespace bb::Avm_vm