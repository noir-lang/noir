
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct PedersenRow {
    FF pedersen_sel_pedersen{};

    [[maybe_unused]] static std::vector<std::string> names();
};

inline std::string get_relation_label_pedersen(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class pedersenImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS{
        3,
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

            auto tmp = ((pedersen_sel_pedersen * (-pedersen_sel_pedersen + FF(1))) - FF(0));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
    }
};

template <typename FF> using pedersen = Relation<pedersenImpl<FF>>;

} // namespace bb::Avm_vm