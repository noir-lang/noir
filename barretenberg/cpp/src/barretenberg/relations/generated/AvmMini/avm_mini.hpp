
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace proof_system::AvmMini_vm {

template <typename FF> struct Avm_miniRow {
    FF avmMini_rwc{};
    FF avmMini_rwa{};
    FF avmMini_mem_op_b{};
    FF avmMini_ib{};
    FF avmMini_rwb{};
    FF avmMini_subop{};
    FF avmMini_mem_op_c{};
    FF avmMini_ia{};
    FF avmMini_ic{};
    FF avmMini_mem_op_a{};
};

template <typename FF_> class avm_miniImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 8> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 3, 3,
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {

        // Contribution 0
        {
            DECLARE_VIEWS(0);

            auto tmp = (avmMini_subop * (-avmMini_subop + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            DECLARE_VIEWS(1);

            auto tmp = (avmMini_mem_op_a * (-avmMini_mem_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            DECLARE_VIEWS(2);

            auto tmp = (avmMini_mem_op_b * (-avmMini_mem_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            DECLARE_VIEWS(3);

            auto tmp = (avmMini_mem_op_c * (-avmMini_mem_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            DECLARE_VIEWS(4);

            auto tmp = (avmMini_rwa * (-avmMini_rwa + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            DECLARE_VIEWS(5);

            auto tmp = (avmMini_rwb * (-avmMini_rwb + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            DECLARE_VIEWS(6);

            auto tmp = (avmMini_rwc * (-avmMini_rwc + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            DECLARE_VIEWS(7);

            auto tmp = (avmMini_subop * ((avmMini_ia + avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_mini = Relation<avm_miniImpl<FF>>;

} // namespace proof_system::AvmMini_vm