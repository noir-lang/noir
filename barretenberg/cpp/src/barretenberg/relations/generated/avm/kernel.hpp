
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct KernelRow {
    FF kernel_emit_l2_to_l1_msg_write_offset{};
    FF kernel_emit_l2_to_l1_msg_write_offset_shift{};
    FF kernel_emit_note_hash_write_offset{};
    FF kernel_emit_note_hash_write_offset_shift{};
    FF kernel_emit_nullifier_write_offset{};
    FF kernel_emit_nullifier_write_offset_shift{};
    FF kernel_emit_unencrypted_log_write_offset{};
    FF kernel_emit_unencrypted_log_write_offset_shift{};
    FF kernel_l1_to_l2_msg_exists_write_offset{};
    FF kernel_l1_to_l2_msg_exists_write_offset_shift{};
    FF kernel_note_hash_exist_write_offset{};
    FF kernel_note_hash_exist_write_offset_shift{};
    FF kernel_nullifier_exists_write_offset{};
    FF kernel_nullifier_exists_write_offset_shift{};
    FF kernel_nullifier_non_exists_write_offset{};
    FF kernel_nullifier_non_exists_write_offset_shift{};
    FF kernel_sload_write_offset{};
    FF kernel_sload_write_offset_shift{};
    FF kernel_sstore_write_offset{};
    FF kernel_sstore_write_offset_shift{};
    FF main_ib{};
    FF main_last{};
    FF main_sel_op_emit_l2_to_l1_msg{};
    FF main_sel_op_emit_note_hash{};
    FF main_sel_op_emit_nullifier{};
    FF main_sel_op_emit_unencrypted_log{};
    FF main_sel_op_l1_to_l2_msg_exists{};
    FF main_sel_op_note_hash_exists{};
    FF main_sel_op_nullifier_exists{};
    FF main_sel_op_sload{};
    FF main_sel_op_sstore{};

    [[maybe_unused]] static std::vector<std::string> names();
};

inline std::string get_relation_label_kernel(int index)
{
    switch (index) {
    case 0:
        return "NOTE_HASH_EXISTS_INC_CONSISTENCY_CHECK";

    case 1:
        return "EMIT_NOTE_HASH_INC_CONSISTENCY_CHECK";

    case 2:
        return "NULLIFIER_EXISTS_INC_CONSISTENCY_CHECK";

    case 3:
        return "NULLIFIER_NON_EXISTS_INC_CONSISTENCY_CHECK";

    case 4:
        return "EMIT_NULLIFIER_INC_CONSISTENCY_CHECK";

    case 5:
        return "L1_TO_L2_MSG_EXISTS_INC_CONSISTENCY_CHECK";

    case 6:
        return "EMIT_UNENCRYPTED_LOG_INC_CONSISTENCY_CHECK";

    case 7:
        return "EMIT_L2_TO_L1_MSG_INC_CONSISTENCY_CHECK";

    case 8:
        return "SLOAD_INC_CONSISTENCY_CHECK";

    case 9:
        return "SSTORE_INC_CONSISTENCY_CHECK";
    }
    return std::to_string(index);
}

template <typename FF_> class kernelImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 10> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 4, 4, 3, 3, 3, 3, 3, 3,
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

            auto tmp = ((-main_last + FF(1)) * (kernel_note_hash_exist_write_offset_shift -
                                                (kernel_note_hash_exist_write_offset + main_sel_op_note_hash_exists)));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);

            auto tmp = ((-main_last + FF(1)) * (kernel_emit_note_hash_write_offset_shift -
                                                (kernel_emit_note_hash_write_offset + main_sel_op_emit_note_hash)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);

            auto tmp = ((-main_last + FF(1)) *
                        (kernel_nullifier_exists_write_offset_shift -
                         (kernel_nullifier_exists_write_offset + (main_sel_op_nullifier_exists * main_ib))));
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);

            auto tmp =
                ((-main_last + FF(1)) *
                 (kernel_nullifier_non_exists_write_offset_shift -
                  (kernel_nullifier_non_exists_write_offset + (main_sel_op_nullifier_exists * (-main_ib + FF(1))))));
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);

            auto tmp = ((-main_last + FF(1)) * (kernel_emit_nullifier_write_offset_shift -
                                                (kernel_emit_nullifier_write_offset + main_sel_op_emit_nullifier)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);

            auto tmp =
                ((-main_last + FF(1)) * (kernel_l1_to_l2_msg_exists_write_offset_shift -
                                         (kernel_l1_to_l2_msg_exists_write_offset + main_sel_op_l1_to_l2_msg_exists)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);

            auto tmp = ((-main_last + FF(1)) *
                        (kernel_emit_unencrypted_log_write_offset_shift -
                         (kernel_emit_unencrypted_log_write_offset + main_sel_op_emit_unencrypted_log)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);

            auto tmp =
                ((-main_last + FF(1)) * (kernel_emit_l2_to_l1_msg_write_offset_shift -
                                         (kernel_emit_l2_to_l1_msg_write_offset + main_sel_op_emit_l2_to_l1_msg)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);

            auto tmp = ((-main_last + FF(1)) *
                        (kernel_sload_write_offset_shift - (kernel_sload_write_offset + main_sel_op_sload)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);

            auto tmp = ((-main_last + FF(1)) *
                        (kernel_sstore_write_offset_shift - (kernel_sstore_write_offset + main_sel_op_sstore)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
    }
};

template <typename FF> using kernel = Relation<kernelImpl<FF>>;

} // namespace bb::Avm_vm