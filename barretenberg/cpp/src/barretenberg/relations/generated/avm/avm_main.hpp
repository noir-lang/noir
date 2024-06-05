
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"

namespace bb::Avm_vm {

template <typename FF> struct Avm_mainRow {
    FF avm_kernel_emit_l2_to_l1_msg_write_offset{};
    FF avm_kernel_emit_note_hash_write_offset{};
    FF avm_kernel_emit_nullifier_write_offset{};
    FF avm_kernel_emit_unencrypted_log_write_offset{};
    FF avm_kernel_kernel_in_offset{};
    FF avm_kernel_kernel_out_offset{};
    FF avm_kernel_l1_to_l2_msg_exists_write_offset{};
    FF avm_kernel_note_hash_exist_write_offset{};
    FF avm_kernel_nullifier_exists_write_offset{};
    FF avm_kernel_nullifier_non_exists_write_offset{};
    FF avm_kernel_side_effect_counter{};
    FF avm_kernel_side_effect_counter_shift{};
    FF avm_kernel_sload_write_offset{};
    FF avm_kernel_sstore_write_offset{};
    FF avm_main_alu_in_tag{};
    FF avm_main_alu_sel{};
    FF avm_main_bin_op_id{};
    FF avm_main_bin_sel{};
    FF avm_main_call_ptr{};
    FF avm_main_da_gas_op{};
    FF avm_main_da_gas_remaining{};
    FF avm_main_da_gas_remaining_shift{};
    FF avm_main_first{};
    FF avm_main_gas_cost_active{};
    FF avm_main_ia{};
    FF avm_main_ib{};
    FF avm_main_ic{};
    FF avm_main_id{};
    FF avm_main_id_zero{};
    FF avm_main_ind_op_a{};
    FF avm_main_ind_op_b{};
    FF avm_main_ind_op_c{};
    FF avm_main_ind_op_d{};
    FF avm_main_internal_return_ptr{};
    FF avm_main_internal_return_ptr_shift{};
    FF avm_main_inv{};
    FF avm_main_l2_gas_op{};
    FF avm_main_l2_gas_remaining{};
    FF avm_main_l2_gas_remaining_shift{};
    FF avm_main_mem_idx_a{};
    FF avm_main_mem_idx_b{};
    FF avm_main_mem_op_a{};
    FF avm_main_mem_op_activate_gas{};
    FF avm_main_mem_op_b{};
    FF avm_main_mem_op_c{};
    FF avm_main_mem_op_d{};
    FF avm_main_op_err{};
    FF avm_main_pc{};
    FF avm_main_pc_shift{};
    FF avm_main_q_kernel_lookup{};
    FF avm_main_q_kernel_output_lookup{};
    FF avm_main_r_in_tag{};
    FF avm_main_rwa{};
    FF avm_main_rwb{};
    FF avm_main_rwc{};
    FF avm_main_rwd{};
    FF avm_main_sel_cmov{};
    FF avm_main_sel_external_call{};
    FF avm_main_sel_halt{};
    FF avm_main_sel_internal_call{};
    FF avm_main_sel_internal_return{};
    FF avm_main_sel_jump{};
    FF avm_main_sel_jumpi{};
    FF avm_main_sel_mov{};
    FF avm_main_sel_mov_a{};
    FF avm_main_sel_mov_b{};
    FF avm_main_sel_op_add{};
    FF avm_main_sel_op_address{};
    FF avm_main_sel_op_and{};
    FF avm_main_sel_op_block_number{};
    FF avm_main_sel_op_cast{};
    FF avm_main_sel_op_chain_id{};
    FF avm_main_sel_op_coinbase{};
    FF avm_main_sel_op_dagasleft{};
    FF avm_main_sel_op_div{};
    FF avm_main_sel_op_emit_l2_to_l1_msg{};
    FF avm_main_sel_op_emit_note_hash{};
    FF avm_main_sel_op_emit_nullifier{};
    FF avm_main_sel_op_emit_unencrypted_log{};
    FF avm_main_sel_op_eq{};
    FF avm_main_sel_op_fdiv{};
    FF avm_main_sel_op_fee_per_da_gas{};
    FF avm_main_sel_op_fee_per_l2_gas{};
    FF avm_main_sel_op_get_contract_instance{};
    FF avm_main_sel_op_keccak{};
    FF avm_main_sel_op_l1_to_l2_msg_exists{};
    FF avm_main_sel_op_l2gasleft{};
    FF avm_main_sel_op_lt{};
    FF avm_main_sel_op_lte{};
    FF avm_main_sel_op_mul{};
    FF avm_main_sel_op_not{};
    FF avm_main_sel_op_note_hash_exists{};
    FF avm_main_sel_op_nullifier_exists{};
    FF avm_main_sel_op_or{};
    FF avm_main_sel_op_pedersen{};
    FF avm_main_sel_op_poseidon2{};
    FF avm_main_sel_op_radix_le{};
    FF avm_main_sel_op_sender{};
    FF avm_main_sel_op_sha256{};
    FF avm_main_sel_op_shl{};
    FF avm_main_sel_op_shr{};
    FF avm_main_sel_op_sload{};
    FF avm_main_sel_op_sstore{};
    FF avm_main_sel_op_storage_address{};
    FF avm_main_sel_op_sub{};
    FF avm_main_sel_op_timestamp{};
    FF avm_main_sel_op_transaction_fee{};
    FF avm_main_sel_op_version{};
    FF avm_main_sel_op_xor{};
    FF avm_main_space_id{};
    FF avm_main_tag_err{};
    FF avm_main_w_in_tag{};
};

inline std::string get_relation_label_avm_main(int index)
{
    switch (index) {
    case 0:
        return "L2_GAS_REMAINING_DECREMENT";

    case 1:
        return "DA_GAS_REMAINING_DECREMENT";

    case 2:
        return "L2_GAS_INACTIVE";

    case 3:
        return "DA_GAS_INACTIVE";

    case 70:
        return "OUTPUT_U8";

    case 71:
        return "SUBOP_FDIV";

    case 72:
        return "SUBOP_FDIV_ZERO_ERR1";

    case 73:
        return "SUBOP_FDIV_ZERO_ERR2";

    case 74:
        return "SUBOP_FDIV_R_IN_TAG_FF";

    case 75:
        return "SUBOP_FDIV_W_IN_TAG_FF";

    case 76:
        return "SUBOP_ERROR_RELEVANT_OP";

    case 77:
        return "KERNEL_INPUT_ACTIVE_CHECK";

    case 78:
        return "KERNEL_OUTPUT_ACTIVE_CHECK";

    case 79:
        return "PC_JUMP";

    case 80:
        return "PC_JUMPI";

    case 81:
        return "RETURN_POINTER_INCREMENT";

    case 87:
        return "RETURN_POINTER_DECREMENT";

    case 93:
        return "PC_INCREMENT";

    case 94:
        return "INTERNAL_RETURN_POINTER_CONSISTENCY";

    case 95:
        return "SPACE_ID_INTERNAL";

    case 96:
        return "SPACE_ID_STANDARD_OPCODES";

    case 97:
        return "CMOV_CONDITION_RES_1";

    case 98:
        return "CMOV_CONDITION_RES_2";

    case 101:
        return "MOV_SAME_VALUE_A";

    case 102:
        return "MOV_SAME_VALUE_B";

    case 103:
        return "MOV_MAIN_SAME_TAG";

    case 107:
        return "L2GASLEFT";

    case 108:
        return "DAGASLEFT";

    case 109:
        return "SENDER_KERNEL";

    case 110:
        return "ADDRESS_KERNEL";

    case 111:
        return "STORAGE_ADDRESS_KERNEL";

    case 112:
        return "FEE_DA_GAS_KERNEL";

    case 113:
        return "FEE_L2_GAS_KERNEL";

    case 114:
        return "FEE_TRANSACTION_FEE_KERNEL";

    case 115:
        return "CHAIN_ID_KERNEL";

    case 116:
        return "VERSION_KERNEL";

    case 117:
        return "BLOCK_NUMBER_KERNEL";

    case 118:
        return "COINBASE_KERNEL";

    case 119:
        return "TIMESTAMP_KERNEL";

    case 120:
        return "NOTE_HASH_KERNEL_OUTPUT";

    case 122:
        return "EMIT_NOTE_HASH_KERNEL_OUTPUT";

    case 124:
        return "NULLIFIER_EXISTS_KERNEL_OUTPUT";

    case 127:
        return "EMIT_NULLIFIER_KERNEL_OUTPUT";

    case 129:
        return "L1_TO_L2_MSG_EXISTS_KERNEL_OUTPUT";

    case 131:
        return "EMIT_UNENCRYPTED_LOG_KERNEL_OUTPUT";

    case 133:
        return "EMIT_L2_TO_L1_MSGS_KERNEL_OUTPUT";

    case 135:
        return "SLOAD_KERNEL_OUTPUT";

    case 137:
        return "SSTORE_KERNEL_OUTPUT";

    case 140:
        return "BIN_SEL_1";

    case 141:
        return "BIN_SEL_2";
    }
    return std::to_string(index);
}

template <typename FF_> class avm_mainImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 142> SUBRELATION_PARTIAL_LENGTHS{
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 5,
        4, 4, 3, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 5, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 4, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2,
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

            auto tmp = (avm_main_gas_cost_active *
                        ((avm_main_l2_gas_remaining_shift - avm_main_l2_gas_remaining) + avm_main_l2_gas_op));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            Avm_DECLARE_VIEWS(1);

            auto tmp = (avm_main_gas_cost_active *
                        ((avm_main_da_gas_remaining_shift - avm_main_da_gas_remaining) + avm_main_da_gas_op));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
        // Contribution 2
        {
            Avm_DECLARE_VIEWS(2);

            auto tmp = ((-avm_main_gas_cost_active + FF(1)) * avm_main_l2_gas_op);
            tmp *= scaling_factor;
            std::get<2>(evals) += tmp;
        }
        // Contribution 3
        {
            Avm_DECLARE_VIEWS(3);

            auto tmp = ((-avm_main_gas_cost_active + FF(1)) * avm_main_da_gas_op);
            tmp *= scaling_factor;
            std::get<3>(evals) += tmp;
        }
        // Contribution 4
        {
            Avm_DECLARE_VIEWS(4);

            auto tmp = (avm_main_sel_op_sender * (-avm_main_sel_op_sender + FF(1)));
            tmp *= scaling_factor;
            std::get<4>(evals) += tmp;
        }
        // Contribution 5
        {
            Avm_DECLARE_VIEWS(5);

            auto tmp = (avm_main_sel_op_address * (-avm_main_sel_op_address + FF(1)));
            tmp *= scaling_factor;
            std::get<5>(evals) += tmp;
        }
        // Contribution 6
        {
            Avm_DECLARE_VIEWS(6);

            auto tmp = (avm_main_sel_op_storage_address * (-avm_main_sel_op_storage_address + FF(1)));
            tmp *= scaling_factor;
            std::get<6>(evals) += tmp;
        }
        // Contribution 7
        {
            Avm_DECLARE_VIEWS(7);

            auto tmp = (avm_main_sel_op_chain_id * (-avm_main_sel_op_chain_id + FF(1)));
            tmp *= scaling_factor;
            std::get<7>(evals) += tmp;
        }
        // Contribution 8
        {
            Avm_DECLARE_VIEWS(8);

            auto tmp = (avm_main_sel_op_version * (-avm_main_sel_op_version + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += tmp;
        }
        // Contribution 9
        {
            Avm_DECLARE_VIEWS(9);

            auto tmp = (avm_main_sel_op_block_number * (-avm_main_sel_op_block_number + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += tmp;
        }
        // Contribution 10
        {
            Avm_DECLARE_VIEWS(10);

            auto tmp = (avm_main_sel_op_coinbase * (-avm_main_sel_op_coinbase + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += tmp;
        }
        // Contribution 11
        {
            Avm_DECLARE_VIEWS(11);

            auto tmp = (avm_main_sel_op_timestamp * (-avm_main_sel_op_timestamp + FF(1)));
            tmp *= scaling_factor;
            std::get<11>(evals) += tmp;
        }
        // Contribution 12
        {
            Avm_DECLARE_VIEWS(12);

            auto tmp = (avm_main_sel_op_fee_per_l2_gas * (-avm_main_sel_op_fee_per_l2_gas + FF(1)));
            tmp *= scaling_factor;
            std::get<12>(evals) += tmp;
        }
        // Contribution 13
        {
            Avm_DECLARE_VIEWS(13);

            auto tmp = (avm_main_sel_op_fee_per_da_gas * (-avm_main_sel_op_fee_per_da_gas + FF(1)));
            tmp *= scaling_factor;
            std::get<13>(evals) += tmp;
        }
        // Contribution 14
        {
            Avm_DECLARE_VIEWS(14);

            auto tmp = (avm_main_sel_op_transaction_fee * (-avm_main_sel_op_transaction_fee + FF(1)));
            tmp *= scaling_factor;
            std::get<14>(evals) += tmp;
        }
        // Contribution 15
        {
            Avm_DECLARE_VIEWS(15);

            auto tmp = (avm_main_sel_op_l2gasleft * (-avm_main_sel_op_l2gasleft + FF(1)));
            tmp *= scaling_factor;
            std::get<15>(evals) += tmp;
        }
        // Contribution 16
        {
            Avm_DECLARE_VIEWS(16);

            auto tmp = (avm_main_sel_op_dagasleft * (-avm_main_sel_op_dagasleft + FF(1)));
            tmp *= scaling_factor;
            std::get<16>(evals) += tmp;
        }
        // Contribution 17
        {
            Avm_DECLARE_VIEWS(17);

            auto tmp = (avm_main_sel_op_note_hash_exists * (-avm_main_sel_op_note_hash_exists + FF(1)));
            tmp *= scaling_factor;
            std::get<17>(evals) += tmp;
        }
        // Contribution 18
        {
            Avm_DECLARE_VIEWS(18);

            auto tmp = (avm_main_sel_op_emit_note_hash * (-avm_main_sel_op_emit_note_hash + FF(1)));
            tmp *= scaling_factor;
            std::get<18>(evals) += tmp;
        }
        // Contribution 19
        {
            Avm_DECLARE_VIEWS(19);

            auto tmp = (avm_main_sel_op_nullifier_exists * (-avm_main_sel_op_nullifier_exists + FF(1)));
            tmp *= scaling_factor;
            std::get<19>(evals) += tmp;
        }
        // Contribution 20
        {
            Avm_DECLARE_VIEWS(20);

            auto tmp = (avm_main_sel_op_emit_nullifier * (-avm_main_sel_op_emit_nullifier + FF(1)));
            tmp *= scaling_factor;
            std::get<20>(evals) += tmp;
        }
        // Contribution 21
        {
            Avm_DECLARE_VIEWS(21);

            auto tmp = (avm_main_sel_op_l1_to_l2_msg_exists * (-avm_main_sel_op_l1_to_l2_msg_exists + FF(1)));
            tmp *= scaling_factor;
            std::get<21>(evals) += tmp;
        }
        // Contribution 22
        {
            Avm_DECLARE_VIEWS(22);

            auto tmp = (avm_main_sel_op_emit_unencrypted_log * (-avm_main_sel_op_emit_unencrypted_log + FF(1)));
            tmp *= scaling_factor;
            std::get<22>(evals) += tmp;
        }
        // Contribution 23
        {
            Avm_DECLARE_VIEWS(23);

            auto tmp = (avm_main_sel_op_emit_l2_to_l1_msg * (-avm_main_sel_op_emit_l2_to_l1_msg + FF(1)));
            tmp *= scaling_factor;
            std::get<23>(evals) += tmp;
        }
        // Contribution 24
        {
            Avm_DECLARE_VIEWS(24);

            auto tmp = (avm_main_sel_op_get_contract_instance * (-avm_main_sel_op_get_contract_instance + FF(1)));
            tmp *= scaling_factor;
            std::get<24>(evals) += tmp;
        }
        // Contribution 25
        {
            Avm_DECLARE_VIEWS(25);

            auto tmp = (avm_main_sel_op_sload * (-avm_main_sel_op_sload + FF(1)));
            tmp *= scaling_factor;
            std::get<25>(evals) += tmp;
        }
        // Contribution 26
        {
            Avm_DECLARE_VIEWS(26);

            auto tmp = (avm_main_sel_op_sstore * (-avm_main_sel_op_sstore + FF(1)));
            tmp *= scaling_factor;
            std::get<26>(evals) += tmp;
        }
        // Contribution 27
        {
            Avm_DECLARE_VIEWS(27);

            auto tmp = (avm_main_sel_op_radix_le * (-avm_main_sel_op_radix_le + FF(1)));
            tmp *= scaling_factor;
            std::get<27>(evals) += tmp;
        }
        // Contribution 28
        {
            Avm_DECLARE_VIEWS(28);

            auto tmp = (avm_main_sel_op_sha256 * (-avm_main_sel_op_sha256 + FF(1)));
            tmp *= scaling_factor;
            std::get<28>(evals) += tmp;
        }
        // Contribution 29
        {
            Avm_DECLARE_VIEWS(29);

            auto tmp = (avm_main_sel_op_poseidon2 * (-avm_main_sel_op_poseidon2 + FF(1)));
            tmp *= scaling_factor;
            std::get<29>(evals) += tmp;
        }
        // Contribution 30
        {
            Avm_DECLARE_VIEWS(30);

            auto tmp = (avm_main_sel_op_keccak * (-avm_main_sel_op_keccak + FF(1)));
            tmp *= scaling_factor;
            std::get<30>(evals) += tmp;
        }
        // Contribution 31
        {
            Avm_DECLARE_VIEWS(31);

            auto tmp = (avm_main_sel_op_pedersen * (-avm_main_sel_op_pedersen + FF(1)));
            tmp *= scaling_factor;
            std::get<31>(evals) += tmp;
        }
        // Contribution 32
        {
            Avm_DECLARE_VIEWS(32);

            auto tmp = (avm_main_sel_op_add * (-avm_main_sel_op_add + FF(1)));
            tmp *= scaling_factor;
            std::get<32>(evals) += tmp;
        }
        // Contribution 33
        {
            Avm_DECLARE_VIEWS(33);

            auto tmp = (avm_main_sel_op_sub * (-avm_main_sel_op_sub + FF(1)));
            tmp *= scaling_factor;
            std::get<33>(evals) += tmp;
        }
        // Contribution 34
        {
            Avm_DECLARE_VIEWS(34);

            auto tmp = (avm_main_sel_op_mul * (-avm_main_sel_op_mul + FF(1)));
            tmp *= scaling_factor;
            std::get<34>(evals) += tmp;
        }
        // Contribution 35
        {
            Avm_DECLARE_VIEWS(35);

            auto tmp = (avm_main_sel_op_div * (-avm_main_sel_op_div + FF(1)));
            tmp *= scaling_factor;
            std::get<35>(evals) += tmp;
        }
        // Contribution 36
        {
            Avm_DECLARE_VIEWS(36);

            auto tmp = (avm_main_sel_op_fdiv * (-avm_main_sel_op_fdiv + FF(1)));
            tmp *= scaling_factor;
            std::get<36>(evals) += tmp;
        }
        // Contribution 37
        {
            Avm_DECLARE_VIEWS(37);

            auto tmp = (avm_main_sel_op_not * (-avm_main_sel_op_not + FF(1)));
            tmp *= scaling_factor;
            std::get<37>(evals) += tmp;
        }
        // Contribution 38
        {
            Avm_DECLARE_VIEWS(38);

            auto tmp = (avm_main_sel_op_eq * (-avm_main_sel_op_eq + FF(1)));
            tmp *= scaling_factor;
            std::get<38>(evals) += tmp;
        }
        // Contribution 39
        {
            Avm_DECLARE_VIEWS(39);

            auto tmp = (avm_main_sel_op_and * (-avm_main_sel_op_and + FF(1)));
            tmp *= scaling_factor;
            std::get<39>(evals) += tmp;
        }
        // Contribution 40
        {
            Avm_DECLARE_VIEWS(40);

            auto tmp = (avm_main_sel_op_or * (-avm_main_sel_op_or + FF(1)));
            tmp *= scaling_factor;
            std::get<40>(evals) += tmp;
        }
        // Contribution 41
        {
            Avm_DECLARE_VIEWS(41);

            auto tmp = (avm_main_sel_op_xor * (-avm_main_sel_op_xor + FF(1)));
            tmp *= scaling_factor;
            std::get<41>(evals) += tmp;
        }
        // Contribution 42
        {
            Avm_DECLARE_VIEWS(42);

            auto tmp = (avm_main_sel_op_cast * (-avm_main_sel_op_cast + FF(1)));
            tmp *= scaling_factor;
            std::get<42>(evals) += tmp;
        }
        // Contribution 43
        {
            Avm_DECLARE_VIEWS(43);

            auto tmp = (avm_main_sel_op_lt * (-avm_main_sel_op_lt + FF(1)));
            tmp *= scaling_factor;
            std::get<43>(evals) += tmp;
        }
        // Contribution 44
        {
            Avm_DECLARE_VIEWS(44);

            auto tmp = (avm_main_sel_op_lte * (-avm_main_sel_op_lte + FF(1)));
            tmp *= scaling_factor;
            std::get<44>(evals) += tmp;
        }
        // Contribution 45
        {
            Avm_DECLARE_VIEWS(45);

            auto tmp = (avm_main_sel_op_shl * (-avm_main_sel_op_shl + FF(1)));
            tmp *= scaling_factor;
            std::get<45>(evals) += tmp;
        }
        // Contribution 46
        {
            Avm_DECLARE_VIEWS(46);

            auto tmp = (avm_main_sel_op_shr * (-avm_main_sel_op_shr + FF(1)));
            tmp *= scaling_factor;
            std::get<46>(evals) += tmp;
        }
        // Contribution 47
        {
            Avm_DECLARE_VIEWS(47);

            auto tmp = (avm_main_sel_internal_call * (-avm_main_sel_internal_call + FF(1)));
            tmp *= scaling_factor;
            std::get<47>(evals) += tmp;
        }
        // Contribution 48
        {
            Avm_DECLARE_VIEWS(48);

            auto tmp = (avm_main_sel_internal_return * (-avm_main_sel_internal_return + FF(1)));
            tmp *= scaling_factor;
            std::get<48>(evals) += tmp;
        }
        // Contribution 49
        {
            Avm_DECLARE_VIEWS(49);

            auto tmp = (avm_main_sel_jump * (-avm_main_sel_jump + FF(1)));
            tmp *= scaling_factor;
            std::get<49>(evals) += tmp;
        }
        // Contribution 50
        {
            Avm_DECLARE_VIEWS(50);

            auto tmp = (avm_main_sel_jumpi * (-avm_main_sel_jumpi + FF(1)));
            tmp *= scaling_factor;
            std::get<50>(evals) += tmp;
        }
        // Contribution 51
        {
            Avm_DECLARE_VIEWS(51);

            auto tmp = (avm_main_sel_halt * (-avm_main_sel_halt + FF(1)));
            tmp *= scaling_factor;
            std::get<51>(evals) += tmp;
        }
        // Contribution 52
        {
            Avm_DECLARE_VIEWS(52);

            auto tmp = (avm_main_sel_external_call * (-avm_main_sel_external_call + FF(1)));
            tmp *= scaling_factor;
            std::get<52>(evals) += tmp;
        }
        // Contribution 53
        {
            Avm_DECLARE_VIEWS(53);

            auto tmp = (avm_main_sel_mov * (-avm_main_sel_mov + FF(1)));
            tmp *= scaling_factor;
            std::get<53>(evals) += tmp;
        }
        // Contribution 54
        {
            Avm_DECLARE_VIEWS(54);

            auto tmp = (avm_main_sel_cmov * (-avm_main_sel_cmov + FF(1)));
            tmp *= scaling_factor;
            std::get<54>(evals) += tmp;
        }
        // Contribution 55
        {
            Avm_DECLARE_VIEWS(55);

            auto tmp = (avm_main_op_err * (-avm_main_op_err + FF(1)));
            tmp *= scaling_factor;
            std::get<55>(evals) += tmp;
        }
        // Contribution 56
        {
            Avm_DECLARE_VIEWS(56);

            auto tmp = (avm_main_tag_err * (-avm_main_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<56>(evals) += tmp;
        }
        // Contribution 57
        {
            Avm_DECLARE_VIEWS(57);

            auto tmp = (avm_main_id_zero * (-avm_main_id_zero + FF(1)));
            tmp *= scaling_factor;
            std::get<57>(evals) += tmp;
        }
        // Contribution 58
        {
            Avm_DECLARE_VIEWS(58);

            auto tmp = (avm_main_mem_op_a * (-avm_main_mem_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<58>(evals) += tmp;
        }
        // Contribution 59
        {
            Avm_DECLARE_VIEWS(59);

            auto tmp = (avm_main_mem_op_b * (-avm_main_mem_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<59>(evals) += tmp;
        }
        // Contribution 60
        {
            Avm_DECLARE_VIEWS(60);

            auto tmp = (avm_main_mem_op_c * (-avm_main_mem_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<60>(evals) += tmp;
        }
        // Contribution 61
        {
            Avm_DECLARE_VIEWS(61);

            auto tmp = (avm_main_mem_op_d * (-avm_main_mem_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<61>(evals) += tmp;
        }
        // Contribution 62
        {
            Avm_DECLARE_VIEWS(62);

            auto tmp = (avm_main_rwa * (-avm_main_rwa + FF(1)));
            tmp *= scaling_factor;
            std::get<62>(evals) += tmp;
        }
        // Contribution 63
        {
            Avm_DECLARE_VIEWS(63);

            auto tmp = (avm_main_rwb * (-avm_main_rwb + FF(1)));
            tmp *= scaling_factor;
            std::get<63>(evals) += tmp;
        }
        // Contribution 64
        {
            Avm_DECLARE_VIEWS(64);

            auto tmp = (avm_main_rwc * (-avm_main_rwc + FF(1)));
            tmp *= scaling_factor;
            std::get<64>(evals) += tmp;
        }
        // Contribution 65
        {
            Avm_DECLARE_VIEWS(65);

            auto tmp = (avm_main_rwd * (-avm_main_rwd + FF(1)));
            tmp *= scaling_factor;
            std::get<65>(evals) += tmp;
        }
        // Contribution 66
        {
            Avm_DECLARE_VIEWS(66);

            auto tmp = (avm_main_ind_op_a * (-avm_main_ind_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<66>(evals) += tmp;
        }
        // Contribution 67
        {
            Avm_DECLARE_VIEWS(67);

            auto tmp = (avm_main_ind_op_b * (-avm_main_ind_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<67>(evals) += tmp;
        }
        // Contribution 68
        {
            Avm_DECLARE_VIEWS(68);

            auto tmp = (avm_main_ind_op_c * (-avm_main_ind_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<68>(evals) += tmp;
        }
        // Contribution 69
        {
            Avm_DECLARE_VIEWS(69);

            auto tmp = (avm_main_ind_op_d * (-avm_main_ind_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<69>(evals) += tmp;
        }
        // Contribution 70
        {
            Avm_DECLARE_VIEWS(70);

            auto tmp =
                (((avm_main_sel_op_eq + avm_main_sel_op_lte) + avm_main_sel_op_lt) * (avm_main_w_in_tag - FF(1)));
            tmp *= scaling_factor;
            std::get<70>(evals) += tmp;
        }
        // Contribution 71
        {
            Avm_DECLARE_VIEWS(71);

            auto tmp =
                ((avm_main_sel_op_fdiv * (-avm_main_op_err + FF(1))) * ((avm_main_ic * avm_main_ib) - avm_main_ia));
            tmp *= scaling_factor;
            std::get<71>(evals) += tmp;
        }
        // Contribution 72
        {
            Avm_DECLARE_VIEWS(72);

            auto tmp = ((avm_main_sel_op_fdiv + avm_main_sel_op_div) *
                        (((avm_main_ib * avm_main_inv) - FF(1)) + avm_main_op_err));
            tmp *= scaling_factor;
            std::get<72>(evals) += tmp;
        }
        // Contribution 73
        {
            Avm_DECLARE_VIEWS(73);

            auto tmp = (((avm_main_sel_op_fdiv + avm_main_sel_op_div) * avm_main_op_err) * (-avm_main_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<73>(evals) += tmp;
        }
        // Contribution 74
        {
            Avm_DECLARE_VIEWS(74);

            auto tmp = (avm_main_sel_op_fdiv * (avm_main_r_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<74>(evals) += tmp;
        }
        // Contribution 75
        {
            Avm_DECLARE_VIEWS(75);

            auto tmp = (avm_main_sel_op_fdiv * (avm_main_w_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<75>(evals) += tmp;
        }
        // Contribution 76
        {
            Avm_DECLARE_VIEWS(76);

            auto tmp = (avm_main_op_err * ((avm_main_sel_op_fdiv + avm_main_sel_op_div) - FF(1)));
            tmp *= scaling_factor;
            std::get<76>(evals) += tmp;
        }
        // Contribution 77
        {
            Avm_DECLARE_VIEWS(77);

            auto tmp = (((((((((((avm_main_sel_op_sender + avm_main_sel_op_address) + avm_main_sel_op_storage_address) +
                                avm_main_sel_op_chain_id) +
                               avm_main_sel_op_version) +
                              avm_main_sel_op_block_number) +
                             avm_main_sel_op_coinbase) +
                            avm_main_sel_op_timestamp) +
                           avm_main_sel_op_fee_per_l2_gas) +
                          avm_main_sel_op_fee_per_da_gas) +
                         avm_main_sel_op_transaction_fee) *
                        (-avm_main_q_kernel_lookup + FF(1)));
            tmp *= scaling_factor;
            std::get<77>(evals) += tmp;
        }
        // Contribution 78
        {
            Avm_DECLARE_VIEWS(78);

            auto tmp = (((((((((avm_main_sel_op_note_hash_exists + avm_main_sel_op_emit_note_hash) +
                               avm_main_sel_op_nullifier_exists) +
                              avm_main_sel_op_emit_nullifier) +
                             avm_main_sel_op_l1_to_l2_msg_exists) +
                            avm_main_sel_op_emit_unencrypted_log) +
                           avm_main_sel_op_emit_l2_to_l1_msg) +
                          avm_main_sel_op_sload) +
                         avm_main_sel_op_sstore) *
                        (-avm_main_q_kernel_output_lookup + FF(1)));
            tmp *= scaling_factor;
            std::get<78>(evals) += tmp;
        }
        // Contribution 79
        {
            Avm_DECLARE_VIEWS(79);

            auto tmp = (avm_main_sel_jump * (avm_main_pc_shift - avm_main_ia));
            tmp *= scaling_factor;
            std::get<79>(evals) += tmp;
        }
        // Contribution 80
        {
            Avm_DECLARE_VIEWS(80);

            auto tmp = (avm_main_sel_jumpi * (((-avm_main_id_zero + FF(1)) * (avm_main_pc_shift - avm_main_ia)) +
                                              (avm_main_id_zero * ((avm_main_pc_shift - avm_main_pc) - FF(1)))));
            tmp *= scaling_factor;
            std::get<80>(evals) += tmp;
        }
        // Contribution 81
        {
            Avm_DECLARE_VIEWS(81);

            auto tmp = (avm_main_sel_internal_call *
                        (avm_main_internal_return_ptr_shift - (avm_main_internal_return_ptr + FF(1))));
            tmp *= scaling_factor;
            std::get<81>(evals) += tmp;
        }
        // Contribution 82
        {
            Avm_DECLARE_VIEWS(82);

            auto tmp = (avm_main_sel_internal_call * (avm_main_internal_return_ptr - avm_main_mem_idx_b));
            tmp *= scaling_factor;
            std::get<82>(evals) += tmp;
        }
        // Contribution 83
        {
            Avm_DECLARE_VIEWS(83);

            auto tmp = (avm_main_sel_internal_call * (avm_main_pc_shift - avm_main_ia));
            tmp *= scaling_factor;
            std::get<83>(evals) += tmp;
        }
        // Contribution 84
        {
            Avm_DECLARE_VIEWS(84);

            auto tmp = (avm_main_sel_internal_call * ((avm_main_pc + FF(1)) - avm_main_ib));
            tmp *= scaling_factor;
            std::get<84>(evals) += tmp;
        }
        // Contribution 85
        {
            Avm_DECLARE_VIEWS(85);

            auto tmp = (avm_main_sel_internal_call * (avm_main_rwb - FF(1)));
            tmp *= scaling_factor;
            std::get<85>(evals) += tmp;
        }
        // Contribution 86
        {
            Avm_DECLARE_VIEWS(86);

            auto tmp = (avm_main_sel_internal_call * (avm_main_mem_op_b - FF(1)));
            tmp *= scaling_factor;
            std::get<86>(evals) += tmp;
        }
        // Contribution 87
        {
            Avm_DECLARE_VIEWS(87);

            auto tmp = (avm_main_sel_internal_return *
                        (avm_main_internal_return_ptr_shift - (avm_main_internal_return_ptr - FF(1))));
            tmp *= scaling_factor;
            std::get<87>(evals) += tmp;
        }
        // Contribution 88
        {
            Avm_DECLARE_VIEWS(88);

            auto tmp = (avm_main_sel_internal_return * ((avm_main_internal_return_ptr - FF(1)) - avm_main_mem_idx_a));
            tmp *= scaling_factor;
            std::get<88>(evals) += tmp;
        }
        // Contribution 89
        {
            Avm_DECLARE_VIEWS(89);

            auto tmp = (avm_main_sel_internal_return * (avm_main_pc_shift - avm_main_ia));
            tmp *= scaling_factor;
            std::get<89>(evals) += tmp;
        }
        // Contribution 90
        {
            Avm_DECLARE_VIEWS(90);

            auto tmp = (avm_main_sel_internal_return * avm_main_rwa);
            tmp *= scaling_factor;
            std::get<90>(evals) += tmp;
        }
        // Contribution 91
        {
            Avm_DECLARE_VIEWS(91);

            auto tmp = (avm_main_sel_internal_return * (avm_main_mem_op_a - FF(1)));
            tmp *= scaling_factor;
            std::get<91>(evals) += tmp;
        }
        // Contribution 92
        {
            Avm_DECLARE_VIEWS(92);

            auto tmp =
                (((avm_main_gas_cost_active -
                   (((((((avm_main_sel_op_fdiv +
                          ((((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) +
                                  avm_main_sel_op_div) +
                                 avm_main_sel_op_not) +
                                avm_main_sel_op_eq) +
                               avm_main_sel_op_lt) +
                              avm_main_sel_op_lte) +
                             avm_main_sel_op_shr) +
                            avm_main_sel_op_shl) +
                           avm_main_sel_op_cast)) +
                         ((avm_main_sel_op_and + avm_main_sel_op_or) + avm_main_sel_op_xor)) +
                        (avm_main_sel_cmov + avm_main_sel_mov)) +
                       ((((avm_main_sel_op_radix_le + avm_main_sel_op_sha256) + avm_main_sel_op_poseidon2) +
                         avm_main_sel_op_keccak) +
                        avm_main_sel_op_pedersen)) +
                      ((((((((((avm_main_sel_op_sender + avm_main_sel_op_address) + avm_main_sel_op_storage_address) +
                              avm_main_sel_op_chain_id) +
                             avm_main_sel_op_version) +
                            avm_main_sel_op_block_number) +
                           avm_main_sel_op_coinbase) +
                          avm_main_sel_op_timestamp) +
                         avm_main_sel_op_fee_per_l2_gas) +
                        avm_main_sel_op_fee_per_da_gas) +
                       avm_main_sel_op_transaction_fee)) +
                     ((((((((avm_main_sel_op_note_hash_exists + avm_main_sel_op_emit_note_hash) +
                            avm_main_sel_op_nullifier_exists) +
                           avm_main_sel_op_emit_nullifier) +
                          avm_main_sel_op_l1_to_l2_msg_exists) +
                         avm_main_sel_op_emit_unencrypted_log) +
                        avm_main_sel_op_emit_l2_to_l1_msg) +
                       avm_main_sel_op_sload) +
                      avm_main_sel_op_sstore)) +
                    (avm_main_sel_op_dagasleft + avm_main_sel_op_l2gasleft))) -
                  (((avm_main_sel_jump + avm_main_sel_jumpi) + avm_main_sel_internal_call) +
                   avm_main_sel_internal_return)) -
                 avm_main_mem_op_activate_gas);
            tmp *= scaling_factor;
            std::get<92>(evals) += tmp;
        }
        // Contribution 93
        {
            Avm_DECLARE_VIEWS(93);

            auto tmp =
                ((((-avm_main_first + FF(1)) * (-avm_main_sel_halt + FF(1))) *
                  (((((((avm_main_sel_op_fdiv +
                         ((((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) +
                                 avm_main_sel_op_div) +
                                avm_main_sel_op_not) +
                               avm_main_sel_op_eq) +
                              avm_main_sel_op_lt) +
                             avm_main_sel_op_lte) +
                            avm_main_sel_op_shr) +
                           avm_main_sel_op_shl) +
                          avm_main_sel_op_cast)) +
                        ((avm_main_sel_op_and + avm_main_sel_op_or) + avm_main_sel_op_xor)) +
                       (avm_main_sel_cmov + avm_main_sel_mov)) +
                      ((((avm_main_sel_op_radix_le + avm_main_sel_op_sha256) + avm_main_sel_op_poseidon2) +
                        avm_main_sel_op_keccak) +
                       avm_main_sel_op_pedersen)) +
                     ((((((((((avm_main_sel_op_sender + avm_main_sel_op_address) + avm_main_sel_op_storage_address) +
                             avm_main_sel_op_chain_id) +
                            avm_main_sel_op_version) +
                           avm_main_sel_op_block_number) +
                          avm_main_sel_op_coinbase) +
                         avm_main_sel_op_timestamp) +
                        avm_main_sel_op_fee_per_l2_gas) +
                       avm_main_sel_op_fee_per_da_gas) +
                      avm_main_sel_op_transaction_fee)) +
                    ((((((((avm_main_sel_op_note_hash_exists + avm_main_sel_op_emit_note_hash) +
                           avm_main_sel_op_nullifier_exists) +
                          avm_main_sel_op_emit_nullifier) +
                         avm_main_sel_op_l1_to_l2_msg_exists) +
                        avm_main_sel_op_emit_unencrypted_log) +
                       avm_main_sel_op_emit_l2_to_l1_msg) +
                      avm_main_sel_op_sload) +
                     avm_main_sel_op_sstore)) +
                   (avm_main_sel_op_dagasleft + avm_main_sel_op_l2gasleft))) *
                 (avm_main_pc_shift - (avm_main_pc + FF(1))));
            tmp *= scaling_factor;
            std::get<93>(evals) += tmp;
        }
        // Contribution 94
        {
            Avm_DECLARE_VIEWS(94);

            auto tmp = ((-(((avm_main_first + avm_main_sel_internal_call) + avm_main_sel_internal_return) +
                           avm_main_sel_halt) +
                         FF(1)) *
                        (avm_main_internal_return_ptr_shift - avm_main_internal_return_ptr));
            tmp *= scaling_factor;
            std::get<94>(evals) += tmp;
        }
        // Contribution 95
        {
            Avm_DECLARE_VIEWS(95);

            auto tmp = ((avm_main_sel_internal_call + avm_main_sel_internal_return) * (avm_main_space_id - FF(255)));
            tmp *= scaling_factor;
            std::get<95>(evals) += tmp;
        }
        // Contribution 96
        {
            Avm_DECLARE_VIEWS(96);

            auto tmp =
                ((((((((avm_main_sel_op_fdiv +
                        ((((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) +
                                avm_main_sel_op_div) +
                               avm_main_sel_op_not) +
                              avm_main_sel_op_eq) +
                             avm_main_sel_op_lt) +
                            avm_main_sel_op_lte) +
                           avm_main_sel_op_shr) +
                          avm_main_sel_op_shl) +
                         avm_main_sel_op_cast)) +
                       ((avm_main_sel_op_and + avm_main_sel_op_or) + avm_main_sel_op_xor)) +
                      (avm_main_sel_cmov + avm_main_sel_mov)) +
                     ((((avm_main_sel_op_radix_le + avm_main_sel_op_sha256) + avm_main_sel_op_poseidon2) +
                       avm_main_sel_op_keccak) +
                      avm_main_sel_op_pedersen)) +
                    ((((((((((avm_main_sel_op_sender + avm_main_sel_op_address) + avm_main_sel_op_storage_address) +
                            avm_main_sel_op_chain_id) +
                           avm_main_sel_op_version) +
                          avm_main_sel_op_block_number) +
                         avm_main_sel_op_coinbase) +
                        avm_main_sel_op_timestamp) +
                       avm_main_sel_op_fee_per_l2_gas) +
                      avm_main_sel_op_fee_per_da_gas) +
                     avm_main_sel_op_transaction_fee)) +
                   ((((((((avm_main_sel_op_note_hash_exists + avm_main_sel_op_emit_note_hash) +
                          avm_main_sel_op_nullifier_exists) +
                         avm_main_sel_op_emit_nullifier) +
                        avm_main_sel_op_l1_to_l2_msg_exists) +
                       avm_main_sel_op_emit_unencrypted_log) +
                      avm_main_sel_op_emit_l2_to_l1_msg) +
                     avm_main_sel_op_sload) +
                    avm_main_sel_op_sstore)) +
                  (avm_main_sel_op_dagasleft + avm_main_sel_op_l2gasleft)) *
                 (avm_main_call_ptr - avm_main_space_id));
            tmp *= scaling_factor;
            std::get<96>(evals) += tmp;
        }
        // Contribution 97
        {
            Avm_DECLARE_VIEWS(97);

            auto tmp = ((avm_main_sel_cmov + avm_main_sel_jumpi) *
                        (((avm_main_id * avm_main_inv) - FF(1)) + avm_main_id_zero));
            tmp *= scaling_factor;
            std::get<97>(evals) += tmp;
        }
        // Contribution 98
        {
            Avm_DECLARE_VIEWS(98);

            auto tmp = (((avm_main_sel_cmov + avm_main_sel_jumpi) * avm_main_id_zero) * (-avm_main_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<98>(evals) += tmp;
        }
        // Contribution 99
        {
            Avm_DECLARE_VIEWS(99);

            auto tmp = (avm_main_sel_mov_a - (avm_main_sel_mov + (avm_main_sel_cmov * (-avm_main_id_zero + FF(1)))));
            tmp *= scaling_factor;
            std::get<99>(evals) += tmp;
        }
        // Contribution 100
        {
            Avm_DECLARE_VIEWS(100);

            auto tmp = (avm_main_sel_mov_b - (avm_main_sel_cmov * avm_main_id_zero));
            tmp *= scaling_factor;
            std::get<100>(evals) += tmp;
        }
        // Contribution 101
        {
            Avm_DECLARE_VIEWS(101);

            auto tmp = (avm_main_sel_mov_a * (avm_main_ia - avm_main_ic));
            tmp *= scaling_factor;
            std::get<101>(evals) += tmp;
        }
        // Contribution 102
        {
            Avm_DECLARE_VIEWS(102);

            auto tmp = (avm_main_sel_mov_b * (avm_main_ib - avm_main_ic));
            tmp *= scaling_factor;
            std::get<102>(evals) += tmp;
        }
        // Contribution 103
        {
            Avm_DECLARE_VIEWS(103);

            auto tmp = ((avm_main_sel_mov + avm_main_sel_cmov) * (avm_main_r_in_tag - avm_main_w_in_tag));
            tmp *= scaling_factor;
            std::get<103>(evals) += tmp;
        }
        // Contribution 104
        {
            Avm_DECLARE_VIEWS(104);

            auto tmp =
                (avm_main_alu_sel -
                 ((((((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) + avm_main_sel_op_div) +
                          avm_main_sel_op_not) +
                         avm_main_sel_op_eq) +
                        avm_main_sel_op_lt) +
                       avm_main_sel_op_lte) +
                      avm_main_sel_op_shr) +
                     avm_main_sel_op_shl) +
                    avm_main_sel_op_cast) *
                   (-avm_main_tag_err + FF(1))) *
                  (-avm_main_op_err + FF(1))));
            tmp *= scaling_factor;
            std::get<104>(evals) += tmp;
        }
        // Contribution 105
        {
            Avm_DECLARE_VIEWS(105);

            auto tmp =
                ((((((((((avm_main_sel_op_add + avm_main_sel_op_sub) + avm_main_sel_op_mul) + avm_main_sel_op_div) +
                       avm_main_sel_op_not) +
                      avm_main_sel_op_eq) +
                     avm_main_sel_op_lt) +
                    avm_main_sel_op_lte) +
                   avm_main_sel_op_shr) +
                  avm_main_sel_op_shl) *
                 (avm_main_alu_in_tag - avm_main_r_in_tag));
            tmp *= scaling_factor;
            std::get<105>(evals) += tmp;
        }
        // Contribution 106
        {
            Avm_DECLARE_VIEWS(106);

            auto tmp = (avm_main_sel_op_cast * (avm_main_alu_in_tag - avm_main_w_in_tag));
            tmp *= scaling_factor;
            std::get<106>(evals) += tmp;
        }
        // Contribution 107
        {
            Avm_DECLARE_VIEWS(107);

            auto tmp = (avm_main_sel_op_l2gasleft * (avm_main_ia - avm_main_l2_gas_remaining_shift));
            tmp *= scaling_factor;
            std::get<107>(evals) += tmp;
        }
        // Contribution 108
        {
            Avm_DECLARE_VIEWS(108);

            auto tmp = (avm_main_sel_op_dagasleft * (avm_main_ia - avm_main_da_gas_remaining_shift));
            tmp *= scaling_factor;
            std::get<108>(evals) += tmp;
        }
        // Contribution 109
        {
            Avm_DECLARE_VIEWS(109);

            auto tmp = (avm_main_sel_op_sender * (avm_kernel_kernel_in_offset - FF(0)));
            tmp *= scaling_factor;
            std::get<109>(evals) += tmp;
        }
        // Contribution 110
        {
            Avm_DECLARE_VIEWS(110);

            auto tmp = (avm_main_sel_op_address * (avm_kernel_kernel_in_offset - FF(1)));
            tmp *= scaling_factor;
            std::get<110>(evals) += tmp;
        }
        // Contribution 111
        {
            Avm_DECLARE_VIEWS(111);

            auto tmp = (avm_main_sel_op_storage_address * (avm_kernel_kernel_in_offset - FF(2)));
            tmp *= scaling_factor;
            std::get<111>(evals) += tmp;
        }
        // Contribution 112
        {
            Avm_DECLARE_VIEWS(112);

            auto tmp = (avm_main_sel_op_fee_per_da_gas * (avm_kernel_kernel_in_offset - FF(35)));
            tmp *= scaling_factor;
            std::get<112>(evals) += tmp;
        }
        // Contribution 113
        {
            Avm_DECLARE_VIEWS(113);

            auto tmp = (avm_main_sel_op_fee_per_l2_gas * (avm_kernel_kernel_in_offset - FF(36)));
            tmp *= scaling_factor;
            std::get<113>(evals) += tmp;
        }
        // Contribution 114
        {
            Avm_DECLARE_VIEWS(114);

            auto tmp = (avm_main_sel_op_transaction_fee * (avm_kernel_kernel_in_offset - FF(40)));
            tmp *= scaling_factor;
            std::get<114>(evals) += tmp;
        }
        // Contribution 115
        {
            Avm_DECLARE_VIEWS(115);

            auto tmp = (avm_main_sel_op_chain_id * (avm_kernel_kernel_in_offset - FF(29)));
            tmp *= scaling_factor;
            std::get<115>(evals) += tmp;
        }
        // Contribution 116
        {
            Avm_DECLARE_VIEWS(116);

            auto tmp = (avm_main_sel_op_version * (avm_kernel_kernel_in_offset - FF(30)));
            tmp *= scaling_factor;
            std::get<116>(evals) += tmp;
        }
        // Contribution 117
        {
            Avm_DECLARE_VIEWS(117);

            auto tmp = (avm_main_sel_op_block_number * (avm_kernel_kernel_in_offset - FF(31)));
            tmp *= scaling_factor;
            std::get<117>(evals) += tmp;
        }
        // Contribution 118
        {
            Avm_DECLARE_VIEWS(118);

            auto tmp = (avm_main_sel_op_coinbase * (avm_kernel_kernel_in_offset - FF(33)));
            tmp *= scaling_factor;
            std::get<118>(evals) += tmp;
        }
        // Contribution 119
        {
            Avm_DECLARE_VIEWS(119);

            auto tmp = (avm_main_sel_op_timestamp * (avm_kernel_kernel_in_offset - FF(32)));
            tmp *= scaling_factor;
            std::get<119>(evals) += tmp;
        }
        // Contribution 120
        {
            Avm_DECLARE_VIEWS(120);

            auto tmp = (avm_main_sel_op_note_hash_exists *
                        (avm_kernel_kernel_out_offset - (avm_kernel_note_hash_exist_write_offset + FF(0))));
            tmp *= scaling_factor;
            std::get<120>(evals) += tmp;
        }
        // Contribution 121
        {
            Avm_DECLARE_VIEWS(121);

            auto tmp = (avm_main_first * avm_kernel_note_hash_exist_write_offset);
            tmp *= scaling_factor;
            std::get<121>(evals) += tmp;
        }
        // Contribution 122
        {
            Avm_DECLARE_VIEWS(122);

            auto tmp = (avm_main_sel_op_emit_note_hash *
                        (avm_kernel_kernel_out_offset - (avm_kernel_emit_note_hash_write_offset + FF(160))));
            tmp *= scaling_factor;
            std::get<122>(evals) += tmp;
        }
        // Contribution 123
        {
            Avm_DECLARE_VIEWS(123);

            auto tmp = (avm_main_first * avm_kernel_emit_note_hash_write_offset);
            tmp *= scaling_factor;
            std::get<123>(evals) += tmp;
        }
        // Contribution 124
        {
            Avm_DECLARE_VIEWS(124);

            auto tmp = (avm_main_sel_op_nullifier_exists *
                        (avm_kernel_kernel_out_offset -
                         ((avm_main_ib * (avm_kernel_nullifier_exists_write_offset + FF(32))) +
                          ((-avm_main_ib + FF(1)) * (avm_kernel_nullifier_non_exists_write_offset + FF(64))))));
            tmp *= scaling_factor;
            std::get<124>(evals) += tmp;
        }
        // Contribution 125
        {
            Avm_DECLARE_VIEWS(125);

            auto tmp = (avm_main_first * avm_kernel_nullifier_exists_write_offset);
            tmp *= scaling_factor;
            std::get<125>(evals) += tmp;
        }
        // Contribution 126
        {
            Avm_DECLARE_VIEWS(126);

            auto tmp = (avm_main_first * avm_kernel_nullifier_non_exists_write_offset);
            tmp *= scaling_factor;
            std::get<126>(evals) += tmp;
        }
        // Contribution 127
        {
            Avm_DECLARE_VIEWS(127);

            auto tmp = (avm_main_sel_op_emit_nullifier *
                        (avm_kernel_kernel_out_offset - (avm_kernel_emit_nullifier_write_offset + FF(176))));
            tmp *= scaling_factor;
            std::get<127>(evals) += tmp;
        }
        // Contribution 128
        {
            Avm_DECLARE_VIEWS(128);

            auto tmp = (avm_main_first * avm_kernel_emit_nullifier_write_offset);
            tmp *= scaling_factor;
            std::get<128>(evals) += tmp;
        }
        // Contribution 129
        {
            Avm_DECLARE_VIEWS(129);

            auto tmp = (avm_main_sel_op_l1_to_l2_msg_exists *
                        (avm_kernel_kernel_out_offset - (avm_kernel_l1_to_l2_msg_exists_write_offset + FF(96))));
            tmp *= scaling_factor;
            std::get<129>(evals) += tmp;
        }
        // Contribution 130
        {
            Avm_DECLARE_VIEWS(130);

            auto tmp = (avm_main_first * avm_kernel_l1_to_l2_msg_exists_write_offset);
            tmp *= scaling_factor;
            std::get<130>(evals) += tmp;
        }
        // Contribution 131
        {
            Avm_DECLARE_VIEWS(131);

            auto tmp = (avm_main_sel_op_emit_unencrypted_log *
                        (avm_kernel_kernel_out_offset - (avm_kernel_emit_unencrypted_log_write_offset + FF(194))));
            tmp *= scaling_factor;
            std::get<131>(evals) += tmp;
        }
        // Contribution 132
        {
            Avm_DECLARE_VIEWS(132);

            auto tmp = (avm_main_first * avm_kernel_emit_unencrypted_log_write_offset);
            tmp *= scaling_factor;
            std::get<132>(evals) += tmp;
        }
        // Contribution 133
        {
            Avm_DECLARE_VIEWS(133);

            auto tmp = (avm_main_sel_op_emit_l2_to_l1_msg *
                        (avm_kernel_kernel_out_offset - (avm_kernel_emit_l2_to_l1_msg_write_offset + FF(192))));
            tmp *= scaling_factor;
            std::get<133>(evals) += tmp;
        }
        // Contribution 134
        {
            Avm_DECLARE_VIEWS(134);

            auto tmp = (avm_main_first * avm_kernel_emit_l2_to_l1_msg_write_offset);
            tmp *= scaling_factor;
            std::get<134>(evals) += tmp;
        }
        // Contribution 135
        {
            Avm_DECLARE_VIEWS(135);

            auto tmp =
                (avm_main_sel_op_sload * (avm_kernel_kernel_out_offset - (avm_kernel_sload_write_offset + FF(144))));
            tmp *= scaling_factor;
            std::get<135>(evals) += tmp;
        }
        // Contribution 136
        {
            Avm_DECLARE_VIEWS(136);

            auto tmp = (avm_main_first * avm_kernel_sload_write_offset);
            tmp *= scaling_factor;
            std::get<136>(evals) += tmp;
        }
        // Contribution 137
        {
            Avm_DECLARE_VIEWS(137);

            auto tmp =
                (avm_main_sel_op_sstore * (avm_kernel_kernel_out_offset - (avm_kernel_sstore_write_offset + FF(128))));
            tmp *= scaling_factor;
            std::get<137>(evals) += tmp;
        }
        // Contribution 138
        {
            Avm_DECLARE_VIEWS(138);

            auto tmp = (avm_main_first * avm_kernel_sstore_write_offset);
            tmp *= scaling_factor;
            std::get<138>(evals) += tmp;
        }
        // Contribution 139
        {
            Avm_DECLARE_VIEWS(139);

            auto tmp = (((((((((avm_main_sel_op_note_hash_exists + avm_main_sel_op_emit_note_hash) +
                               avm_main_sel_op_nullifier_exists) +
                              avm_main_sel_op_emit_nullifier) +
                             avm_main_sel_op_l1_to_l2_msg_exists) +
                            avm_main_sel_op_emit_unencrypted_log) +
                           avm_main_sel_op_emit_l2_to_l1_msg) +
                          avm_main_sel_op_sload) +
                         avm_main_sel_op_sstore) *
                        (avm_kernel_side_effect_counter_shift - (avm_kernel_side_effect_counter + FF(1))));
            tmp *= scaling_factor;
            std::get<139>(evals) += tmp;
        }
        // Contribution 140
        {
            Avm_DECLARE_VIEWS(140);

            auto tmp = (avm_main_bin_op_id - (avm_main_sel_op_or + (avm_main_sel_op_xor * FF(2))));
            tmp *= scaling_factor;
            std::get<140>(evals) += tmp;
        }
        // Contribution 141
        {
            Avm_DECLARE_VIEWS(141);

            auto tmp = (avm_main_bin_sel - ((avm_main_sel_op_and + avm_main_sel_op_or) + avm_main_sel_op_xor));
            tmp *= scaling_factor;
            std::get<141>(evals) += tmp;
        }
    }
};

template <typename FF> using avm_main = Relation<avm_mainImpl<FF>>;

} // namespace bb::Avm_vm