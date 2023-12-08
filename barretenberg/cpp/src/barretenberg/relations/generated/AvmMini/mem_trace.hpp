
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace proof_system::AvmMini_vm {

template <typename FF> struct Mem_traceRow {
    FF memTrace_m_rw_shift{};
    FF memTrace_m_lastAccess{};
    FF memTrace_m_addr{};
    FF memTrace_m_val_shift{};
    FF memTrace_m_rw{};
    FF avmMini_first{};
    FF memTrace_m_addr_shift{};
    FF avmMini_last{};
    FF memTrace_m_val{};
};

template <typename FF_> class mem_traceImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 4> SUBRELATION_PARTIAL_LENGTHS{
        3,
        3,
        4,
        6,
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

            auto tmp = (memTrace_m_lastAccess * (-memTrace_m_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            DECLARE_VIEWS(1);

            auto tmp = (memTrace_m_rw * (-memTrace_m_rw + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            DECLARE_VIEWS(2);

            auto tmp = (((-avmMini_first + FF(1)) * (-memTrace_m_lastAccess + FF(1))) *
                        (memTrace_m_addr_shift - memTrace_m_addr));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            DECLARE_VIEWS(3);

            auto tmp = (((((-avmMini_first + FF(1)) * (-avmMini_last + FF(1))) * (-memTrace_m_lastAccess + FF(1))) *
                         (-memTrace_m_rw_shift + FF(1))) *
                        (memTrace_m_val_shift - memTrace_m_val));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
    }
};

template <typename FF> using mem_trace = Relation<mem_traceImpl<FF>>;

} // namespace proof_system::AvmMini_vm