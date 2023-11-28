
#pragma once
#include "../relation_parameters.hpp"
#include "../relation_types.hpp"

namespace proof_system::AvmMini_vm {

template <typename FF> struct Row {
    FF avmMini_clk{};
    FF avmMini_positive{};
    FF avmMini_first{};
    FF avmMini_subop{};
    FF avmMini_ia{};
    FF avmMini_ib{};
    FF avmMini_ic{};
    FF avmMini_mem_op_a{};
    FF avmMini_mem_op_b{};
    FF avmMini_mem_op_c{};
    FF avmMini_rwa{};
    FF avmMini_rwb{};
    FF avmMini_rwc{};
    FF avmMini_mem_idx_a{};
    FF avmMini_mem_idx_b{};
    FF avmMini_mem_idx_c{};
    FF avmMini_last{};
    FF avmMini_m_clk{};
    FF avmMini_m_sub_clk{};
    FF avmMini_m_addr{};
    FF avmMini_m_val{};
    FF avmMini_m_lastAccess{};
    FF avmMini_m_rw{};
    FF avmMini_m_val_shift{};
    FF avmMini_m_addr_shift{};
    FF avmMini_m_rw_shift{};
};

#define DECLARE_VIEWS(index)                                                                                           \
    using View = typename std::tuple_element<index, ContainerOverSubrelations>::type;                                  \
    [[maybe_unused]] auto avmMini_clk = View(new_term.avmMini_clk);                                                    \
    [[maybe_unused]] auto avmMini_positive = View(new_term.avmMini_positive);                                          \
    [[maybe_unused]] auto avmMini_first = View(new_term.avmMini_first);                                                \
    [[maybe_unused]] auto avmMini_subop = View(new_term.avmMini_subop);                                                \
    [[maybe_unused]] auto avmMini_ia = View(new_term.avmMini_ia);                                                      \
    [[maybe_unused]] auto avmMini_ib = View(new_term.avmMini_ib);                                                      \
    [[maybe_unused]] auto avmMini_ic = View(new_term.avmMini_ic);                                                      \
    [[maybe_unused]] auto avmMini_mem_op_a = View(new_term.avmMini_mem_op_a);                                          \
    [[maybe_unused]] auto avmMini_mem_op_b = View(new_term.avmMini_mem_op_b);                                          \
    [[maybe_unused]] auto avmMini_mem_op_c = View(new_term.avmMini_mem_op_c);                                          \
    [[maybe_unused]] auto avmMini_rwa = View(new_term.avmMini_rwa);                                                    \
    [[maybe_unused]] auto avmMini_rwb = View(new_term.avmMini_rwb);                                                    \
    [[maybe_unused]] auto avmMini_rwc = View(new_term.avmMini_rwc);                                                    \
    [[maybe_unused]] auto avmMini_mem_idx_a = View(new_term.avmMini_mem_idx_a);                                        \
    [[maybe_unused]] auto avmMini_mem_idx_b = View(new_term.avmMini_mem_idx_b);                                        \
    [[maybe_unused]] auto avmMini_mem_idx_c = View(new_term.avmMini_mem_idx_c);                                        \
    [[maybe_unused]] auto avmMini_last = View(new_term.avmMini_last);                                                  \
    [[maybe_unused]] auto avmMini_m_clk = View(new_term.avmMini_m_clk);                                                \
    [[maybe_unused]] auto avmMini_m_sub_clk = View(new_term.avmMini_m_sub_clk);                                        \
    [[maybe_unused]] auto avmMini_m_addr = View(new_term.avmMini_m_addr);                                              \
    [[maybe_unused]] auto avmMini_m_val = View(new_term.avmMini_m_val);                                                \
    [[maybe_unused]] auto avmMini_m_lastAccess = View(new_term.avmMini_m_lastAccess);                                  \
    [[maybe_unused]] auto avmMini_m_rw = View(new_term.avmMini_m_rw);                                                  \
    [[maybe_unused]] auto avmMini_m_val_shift = View(new_term.avmMini_m_val_shift);                                    \
    [[maybe_unused]] auto avmMini_m_addr_shift = View(new_term.avmMini_m_addr_shift);                                  \
    [[maybe_unused]] auto avmMini_m_rw_shift = View(new_term.avmMini_m_rw_shift);

template <typename FF_> class AvmMiniImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 12> SUBRELATION_PARTIAL_LENGTHS{
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
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
        // Contribution 8
        {
            DECLARE_VIEWS(8);

            auto tmp = (avmMini_m_lastAccess * (-avmMini_m_lastAccess + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            DECLARE_VIEWS(9);

            auto tmp = (avmMini_m_rw * (-avmMini_m_rw + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            DECLARE_VIEWS(10);

            auto tmp = (((-avmMini_first + FF(1)) * (-avmMini_m_lastAccess + FF(1))) *
                        (avmMini_m_addr_shift - avmMini_m_addr));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            DECLARE_VIEWS(11);

            auto tmp = (((((-avmMini_first + FF(1)) * (-avmMini_last + FF(1))) * (-avmMini_m_lastAccess + FF(1))) *
                         (-avmMini_m_rw_shift + FF(1))) *
                        (avmMini_m_val_shift - avmMini_m_val));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
    }
};

template <typename FF> using AvmMini = Relation<AvmMiniImpl<FF>>;

} // namespace proof_system::AvmMini_vm