#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Keccakf1600Row {
    FF keccakf1600_sel_keccakf1600{};
};

inline std::string get_relation_label_keccakf1600(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class keccakf1600Impl {
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
            auto tmp = (new_term.keccakf1600_sel_keccakf1600 * (-new_term.keccakf1600_sel_keccakf1600 + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += typename Accumulator::View(tmp);
        }
    }
};

template <typename FF> using keccakf1600 = Relation<keccakf1600Impl<FF>>;

} // namespace bb::Avm_vm