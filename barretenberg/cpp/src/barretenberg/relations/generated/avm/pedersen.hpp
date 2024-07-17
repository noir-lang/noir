#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct PedersenRow {
    FF pedersen_sel_pedersen{};
};

inline std::string get_relation_label_pedersen(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class pedersenImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS = { 3 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        {
            using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
            auto tmp = (new_term.pedersen_sel_pedersen * (-new_term.pedersen_sel_pedersen + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += typename Accumulator::View(tmp);
        }
    }
};

template <typename FF> using pedersen = Relation<pedersenImpl<FF>>;

} // namespace bb::Avm_vm