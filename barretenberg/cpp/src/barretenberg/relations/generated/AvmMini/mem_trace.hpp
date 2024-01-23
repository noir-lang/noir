
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::AvmMini_vm {

template <typename FF> struct Mem_traceRow {
    FF memTrace_m_val{};
    FF memTrace_m_last{};
    FF memTrace_m_in_tag{};
    FF memTrace_m_tag{};
    FF memTrace_m_rw{};
    FF memTrace_m_tag_shift{};
    FF memTrace_m_addr_shift{};
    FF memTrace_m_tag_err{};
    FF memTrace_m_one_min_inv{};
    FF memTrace_m_val_shift{};
    FF memTrace_m_lastAccess{};
    FF memTrace_m_addr{};
    FF memTrace_m_rw_shift{};
};

inline std::string get_relation_label_mem_trace(int index)
{
    switch (index) {
    case 8:
        return "MEM_IN_TAG_CONSISTENCY_1";

    case 9:
        return "MEM_IN_TAG_CONSISTENCY_2";

    case 5:
        return "MEM_READ_WRITE_VAL_CONSISTENCY";

    case 4:
        return "MEM_LAST_ACCESS_DELIMITER";

    case 6:
        return "MEM_READ_WRITE_TAG_CONSISTENCY";

    case 7:
        return "MEM_ZERO_INIT";
    }
    return std::to_string(index);
}

template <typename FF_> class mem_traceImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 10> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 4, 4, 4, 3, 3,
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {

        // Contribution 0
        {
            AvmMini_DECLARE_VIEWS(0);

            auto tmp = (memTrace_m_lastAccess * (-memTrace_m_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            AvmMini_DECLARE_VIEWS(1);

            auto tmp = (memTrace_m_last * (-memTrace_m_last + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            AvmMini_DECLARE_VIEWS(2);

            auto tmp = (memTrace_m_rw * (-memTrace_m_rw + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            AvmMini_DECLARE_VIEWS(3);

            auto tmp = (memTrace_m_tag_err * (-memTrace_m_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            AvmMini_DECLARE_VIEWS(4);

            auto tmp = ((-memTrace_m_lastAccess + FF(1)) * (memTrace_m_addr_shift - memTrace_m_addr));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            AvmMini_DECLARE_VIEWS(5);

            auto tmp = (((-memTrace_m_lastAccess + FF(1)) * (-memTrace_m_rw_shift + FF(1))) *
                        (memTrace_m_val_shift - memTrace_m_val));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            AvmMini_DECLARE_VIEWS(6);

            auto tmp = (((-memTrace_m_lastAccess + FF(1)) * (-memTrace_m_rw_shift + FF(1))) *
                        (memTrace_m_tag_shift - memTrace_m_tag));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            AvmMini_DECLARE_VIEWS(7);

            auto tmp = ((memTrace_m_lastAccess * (-memTrace_m_rw_shift + FF(1))) * memTrace_m_val_shift);
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            AvmMini_DECLARE_VIEWS(8);

            auto tmp =
                (((memTrace_m_in_tag - memTrace_m_tag) * (-memTrace_m_one_min_inv + FF(1))) - memTrace_m_tag_err);
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            AvmMini_DECLARE_VIEWS(9);

            auto tmp = ((-memTrace_m_tag_err + FF(1)) * memTrace_m_one_min_inv);
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
    }
};

template <typename FF> using mem_trace = Relation<mem_traceImpl<FF>>;

} // namespace bb::AvmMini_vm