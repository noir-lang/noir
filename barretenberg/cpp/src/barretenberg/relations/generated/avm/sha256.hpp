#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Sha256Row {
    FF sha256_sel_sha256_compression{};
};

inline std::string get_relation_label_sha256(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class sha256Impl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS = { 3 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        // Contribution 0
        {
            Avm_DECLARE_VIEWS(0);
            auto tmp = (sha256_sel_sha256_compression * (-sha256_sel_sha256_compression + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
    }
};

template <typename FF> using sha256 = Relation<sha256Impl<FF>>;

} // namespace bb::Avm_vm