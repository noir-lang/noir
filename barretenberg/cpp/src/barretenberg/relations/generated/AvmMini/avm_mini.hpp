
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace proof_system::AvmMini_vm {

template <typename FF> struct Avm_miniRow {
    FF avmMini_mem_op_b{};
    FF avmMini_ib{};
    FF avmMini_ic{};
    FF avmMini_sel_op_sub{};
    FF avmMini_mem_op_c{};
    FF avmMini_op_err{};
    FF avmMini_ia{};
    FF avmMini_inv{};
    FF avmMini_sel_op_div{};
    FF avmMini_mem_op_a{};
    FF avmMini_rwa{};
    FF avmMini_sel_op_mul{};
    FF avmMini_rwc{};
    FF avmMini_sel_op_add{};
    FF avmMini_rwb{};
};

template <typename FF_> class avm_miniImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 18> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 5, 4, 4, 3,
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

            auto tmp = (avmMini_sel_op_add * (-avmMini_sel_op_add + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            DECLARE_VIEWS(1);

            auto tmp = (avmMini_sel_op_sub * (-avmMini_sel_op_sub + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            DECLARE_VIEWS(2);

            auto tmp = (avmMini_sel_op_mul * (-avmMini_sel_op_mul + FF(1)));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            DECLARE_VIEWS(3);

            auto tmp = (avmMini_sel_op_div * (-avmMini_sel_op_div + FF(1)));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            DECLARE_VIEWS(4);

            auto tmp = (avmMini_op_err * (-avmMini_op_err + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            DECLARE_VIEWS(5);

            auto tmp = (avmMini_mem_op_a * (-avmMini_mem_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            DECLARE_VIEWS(6);

            auto tmp = (avmMini_mem_op_b * (-avmMini_mem_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            DECLARE_VIEWS(7);

            auto tmp = (avmMini_mem_op_c * (-avmMini_mem_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            DECLARE_VIEWS(8);

            auto tmp = (avmMini_rwa * (-avmMini_rwa + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            DECLARE_VIEWS(9);

            auto tmp = (avmMini_rwb * (-avmMini_rwb + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            DECLARE_VIEWS(10);

            auto tmp = (avmMini_rwc * (-avmMini_rwc + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            DECLARE_VIEWS(11);

            auto tmp = (avmMini_sel_op_add * ((avmMini_ia + avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            DECLARE_VIEWS(12);

            auto tmp = (avmMini_sel_op_sub * ((avmMini_ia - avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            DECLARE_VIEWS(13);

            auto tmp = (avmMini_sel_op_mul * ((avmMini_ia * avmMini_ib) - avmMini_ic));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            DECLARE_VIEWS(14);

            auto tmp = ((avmMini_sel_op_div * (-avmMini_op_err + FF(1))) * ((avmMini_ic * avmMini_ib) - avmMini_ia));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            DECLARE_VIEWS(15);

            auto tmp = (avmMini_sel_op_div * (((avmMini_ib * avmMini_inv) - FF(1)) + avmMini_op_err));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            DECLARE_VIEWS(16);

            auto tmp = ((avmMini_sel_op_div * avmMini_op_err) * (-avmMini_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            DECLARE_VIEWS(17);

            auto tmp = (avmMini_op_err * (avmMini_sel_op_div - FF(1)));
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_mini = Relation<avm_miniImpl<FF>>;

} // namespace proof_system::AvmMini_vm