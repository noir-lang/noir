#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

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
    FF main_sel_last{};
    FF main_sel_op_emit_l2_to_l1_msg{};
    FF main_sel_op_emit_note_hash{};
    FF main_sel_op_emit_nullifier{};
    FF main_sel_op_emit_unencrypted_log{};
    FF main_sel_op_l1_to_l2_msg_exists{};
    FF main_sel_op_note_hash_exists{};
    FF main_sel_op_nullifier_exists{};
    FF main_sel_op_sload{};
    FF main_sel_op_sstore{};
};

template <typename FF_> class kernelImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 10> SUBRELATION_PARTIAL_LENGTHS = { 3, 3, 4, 4, 3, 3, 3, 3, 3, 3 };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        {
            using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_note_hash_exist_write_offset_shift -
                         (new_term.kernel_note_hash_exist_write_offset + new_term.main_sel_op_note_hash_exists)));
            tmp *= scaling_factor;
            std::get<0>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<1, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_emit_note_hash_write_offset_shift -
                         (new_term.kernel_emit_note_hash_write_offset + new_term.main_sel_op_emit_note_hash)));
            tmp *= scaling_factor;
            std::get<1>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<2, ContainerOverSubrelations>;
            auto tmp =
                ((-new_term.main_sel_last + FF(1)) * (new_term.kernel_nullifier_exists_write_offset_shift -
                                                      (new_term.kernel_nullifier_exists_write_offset +
                                                       (new_term.main_sel_op_nullifier_exists * new_term.main_ib))));
            tmp *= scaling_factor;
            std::get<2>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<3, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_nullifier_non_exists_write_offset_shift -
                         (new_term.kernel_nullifier_non_exists_write_offset +
                          (new_term.main_sel_op_nullifier_exists * (-new_term.main_ib + FF(1))))));
            tmp *= scaling_factor;
            std::get<3>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<4, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_emit_nullifier_write_offset_shift -
                         (new_term.kernel_emit_nullifier_write_offset + new_term.main_sel_op_emit_nullifier)));
            tmp *= scaling_factor;
            std::get<4>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<5, ContainerOverSubrelations>;
            auto tmp =
                ((-new_term.main_sel_last + FF(1)) *
                 (new_term.kernel_l1_to_l2_msg_exists_write_offset_shift -
                  (new_term.kernel_l1_to_l2_msg_exists_write_offset + new_term.main_sel_op_l1_to_l2_msg_exists)));
            tmp *= scaling_factor;
            std::get<5>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<6, ContainerOverSubrelations>;
            auto tmp =
                ((-new_term.main_sel_last + FF(1)) *
                 (new_term.kernel_emit_unencrypted_log_write_offset_shift -
                  (new_term.kernel_emit_unencrypted_log_write_offset + new_term.main_sel_op_emit_unencrypted_log)));
            tmp *= scaling_factor;
            std::get<6>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<7, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_emit_l2_to_l1_msg_write_offset_shift -
                         (new_term.kernel_emit_l2_to_l1_msg_write_offset + new_term.main_sel_op_emit_l2_to_l1_msg)));
            tmp *= scaling_factor;
            std::get<7>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<8, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_sload_write_offset_shift -
                         (new_term.kernel_sload_write_offset + new_term.main_sel_op_sload)));
            tmp *= scaling_factor;
            std::get<8>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<9, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_last + FF(1)) *
                        (new_term.kernel_sstore_write_offset_shift -
                         (new_term.kernel_sstore_write_offset + new_term.main_sel_op_sstore)));
            tmp *= scaling_factor;
            std::get<9>(evals) += typename Accumulator::View(tmp);
        }
    }
};

template <typename FF> class kernel : public Relation<kernelImpl<FF>> {
  public:
    static constexpr const char* NAME = "kernel";

    static std::string get_subrelation_label(size_t index)
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
};

} // namespace bb::Avm_vm