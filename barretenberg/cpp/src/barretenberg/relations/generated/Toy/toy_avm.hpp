
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Toy_vm {

template <typename FF> struct Toy_avmRow {
    FF toy_q_xor_table{};
    FF toy_q_tuple_set{};
    FF toy_q_xor{};
};

inline std::string get_relation_label_toy_avm(int index)
{
    switch (index) {}
    return std::to_string(index);
}

template <typename FF_> class toy_avmImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 3> SUBRELATION_PARTIAL_LENGTHS{
        3,
        3,
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
            Toy_DECLARE_VIEWS(0);

            auto tmp = (toy_q_tuple_set * (-toy_q_tuple_set + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Toy_DECLARE_VIEWS(1);

            auto tmp = (toy_q_xor * (-toy_q_xor + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Toy_DECLARE_VIEWS(2);

            auto tmp = (toy_q_xor_table * (-toy_q_xor_table + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
    }
};

template <typename FF> using toy_avm = Relation<toy_avmImpl<FF>>;

} // namespace bb::Toy_vm