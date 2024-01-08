
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace proof_system::Toy_vm {

template <typename FF> struct Toy_avmRow {
    FF toy_x{};
    FF toy_x_shift{};
};

inline std::string get_relation_label_toy_avm(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class toy_avmImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 1> SUBRELATION_PARTIAL_LENGTHS{
        2,
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {

        // Contribution 0
        {
            Toy_DECLARE_VIEWS(0);

            auto tmp = (toy_x_shift - toy_x);
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
    }
};

template <typename FF> using toy_avm = Relation<toy_avmImpl<FF>>;

} // namespace proof_system::Toy_vm