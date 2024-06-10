
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Avm_sha256Row {
    FF avm_sha256_sha256_compression_sel{};

    [[maybe_unused]] static std::vector<std::string> names();
};

inline std::string get_relation_label_avm_sha256(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class avm_sha256Impl {
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

            auto tmp = (avm_sha256_sha256_compression_sel * (-avm_sha256_sha256_compression_sel + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_sha256 = Relation<avm_sha256Impl<FF>>;

} // namespace bb::Avm_vm