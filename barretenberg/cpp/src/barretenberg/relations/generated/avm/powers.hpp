#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct PowersRow {
    FF powers_power_of_2{};
};

inline std::string get_relation_label_powers(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class powersImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS = { 2 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        // Contribution 0
        {
            Avm_DECLARE_VIEWS(0);
            auto tmp = (powers_power_of_2 - powers_power_of_2);
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
    }
};

template <typename FF> using powers = Relation<powersImpl<FF>>;

} // namespace bb::Avm_vm