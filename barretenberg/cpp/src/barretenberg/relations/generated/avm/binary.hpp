
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct BinaryRow {
    FF binary_acc_ia{};
    FF binary_acc_ia_shift{};
    FF binary_acc_ib{};
    FF binary_acc_ib_shift{};
    FF binary_acc_ic{};
    FF binary_acc_ic_shift{};
    FF binary_bin_sel{};
    FF binary_ia_bytes{};
    FF binary_ib_bytes{};
    FF binary_ic_bytes{};
    FF binary_mem_tag_ctr{};
    FF binary_mem_tag_ctr_inv{};
    FF binary_mem_tag_ctr_shift{};
    FF binary_op_id{};
    FF binary_op_id_shift{};

    [[maybe_unused]] static std::vector<std::string> names();
};

inline std::string get_relation_label_binary(int index)
{
    switch (index) {
    case 1:
        return "OP_ID_REL";

    case 2:
        return "MEM_TAG_REL";

    case 3:
        return "BIN_SEL_CTR_REL";

    case 7:
        return "ACC_REL_A";

    case 8:
        return "ACC_REL_B";

    case 9:
        return "ACC_REL_C";
    }
    return std::to_string(index);
}

template <typename FF_> class binaryImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 10> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 4, 3, 3, 3, 4, 4, 4,
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

            auto tmp = (binary_bin_sel * (-binary_bin_sel + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);

            auto tmp = ((binary_op_id_shift - binary_op_id) * binary_mem_tag_ctr);
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);

            auto tmp = (((binary_mem_tag_ctr_shift - binary_mem_tag_ctr) + FF(1)) * binary_mem_tag_ctr);
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);

            auto tmp = ((binary_mem_tag_ctr *
                         (((-binary_bin_sel + FF(1)) * (-binary_mem_tag_ctr_inv + FF(1))) + binary_mem_tag_ctr_inv)) -
                        binary_bin_sel);
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);

            auto tmp = ((-binary_bin_sel + FF(1)) * binary_acc_ia);
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);

            auto tmp = ((-binary_bin_sel + FF(1)) * binary_acc_ib);
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);

            auto tmp = ((-binary_bin_sel + FF(1)) * binary_acc_ic);
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);

            auto tmp = (((binary_acc_ia - binary_ia_bytes) - (binary_acc_ia_shift * FF(256))) * binary_mem_tag_ctr);
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);

            auto tmp = (((binary_acc_ib - binary_ib_bytes) - (binary_acc_ib_shift * FF(256))) * binary_mem_tag_ctr);
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);

            auto tmp = (((binary_acc_ic - binary_ic_bytes) - (binary_acc_ic_shift * FF(256))) * binary_mem_tag_ctr);
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
    }
};

template <typename FF> using binary = Relation<binaryImpl<FF>>;

} // namespace bb::Avm_vm