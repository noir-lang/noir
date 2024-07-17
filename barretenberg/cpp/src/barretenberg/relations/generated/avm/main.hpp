#pragma once

#include "barretenberg/relations/generated/avm/declare_views.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb::Avm_vm {

template <typename FF> struct MainRow {
    FF kernel_emit_l2_to_l1_msg_write_offset{};
    FF kernel_emit_note_hash_write_offset{};
    FF kernel_emit_nullifier_write_offset{};
    FF kernel_emit_unencrypted_log_write_offset{};
    FF kernel_kernel_in_offset{};
    FF kernel_kernel_out_offset{};
    FF kernel_l1_to_l2_msg_exists_write_offset{};
    FF kernel_note_hash_exist_write_offset{};
    FF kernel_nullifier_exists_write_offset{};
    FF kernel_nullifier_non_exists_write_offset{};
    FF kernel_side_effect_counter{};
    FF kernel_side_effect_counter_shift{};
    FF kernel_sload_write_offset{};
    FF kernel_sstore_write_offset{};
    FF main_abs_da_rem_gas_hi{};
    FF main_abs_da_rem_gas_lo{};
    FF main_abs_l2_rem_gas_hi{};
    FF main_abs_l2_rem_gas_lo{};
    FF main_alu_in_tag{};
    FF main_bin_op_id{};
    FF main_call_ptr{};
    FF main_da_gas_op_cost{};
    FF main_da_gas_remaining{};
    FF main_da_gas_remaining_shift{};
    FF main_da_out_of_gas{};
    FF main_ia{};
    FF main_ib{};
    FF main_ic{};
    FF main_id{};
    FF main_id_zero{};
    FF main_internal_return_ptr{};
    FF main_internal_return_ptr_shift{};
    FF main_inv{};
    FF main_l2_gas_op_cost{};
    FF main_l2_gas_remaining{};
    FF main_l2_gas_remaining_shift{};
    FF main_l2_out_of_gas{};
    FF main_mem_addr_a{};
    FF main_mem_addr_b{};
    FF main_op_err{};
    FF main_pc{};
    FF main_pc_shift{};
    FF main_r_in_tag{};
    FF main_rwa{};
    FF main_rwb{};
    FF main_rwc{};
    FF main_rwd{};
    FF main_sel_alu{};
    FF main_sel_bin{};
    FF main_sel_first{};
    FF main_sel_gas_accounting_active{};
    FF main_sel_mem_op_a{};
    FF main_sel_mem_op_activate_gas{};
    FF main_sel_mem_op_b{};
    FF main_sel_mem_op_c{};
    FF main_sel_mem_op_d{};
    FF main_sel_mov_ia_to_ic{};
    FF main_sel_mov_ib_to_ic{};
    FF main_sel_op_add{};
    FF main_sel_op_address{};
    FF main_sel_op_and{};
    FF main_sel_op_block_number{};
    FF main_sel_op_calldata_copy{};
    FF main_sel_op_cast{};
    FF main_sel_op_chain_id{};
    FF main_sel_op_cmov{};
    FF main_sel_op_coinbase{};
    FF main_sel_op_dagasleft{};
    FF main_sel_op_div{};
    FF main_sel_op_emit_l2_to_l1_msg{};
    FF main_sel_op_emit_note_hash{};
    FF main_sel_op_emit_nullifier{};
    FF main_sel_op_emit_unencrypted_log{};
    FF main_sel_op_eq{};
    FF main_sel_op_external_call{};
    FF main_sel_op_external_return{};
    FF main_sel_op_fdiv{};
    FF main_sel_op_fee_per_da_gas{};
    FF main_sel_op_fee_per_l2_gas{};
    FF main_sel_op_function_selector{};
    FF main_sel_op_get_contract_instance{};
    FF main_sel_op_halt{};
    FF main_sel_op_internal_call{};
    FF main_sel_op_internal_return{};
    FF main_sel_op_jump{};
    FF main_sel_op_jumpi{};
    FF main_sel_op_keccak{};
    FF main_sel_op_l1_to_l2_msg_exists{};
    FF main_sel_op_l2gasleft{};
    FF main_sel_op_lt{};
    FF main_sel_op_lte{};
    FF main_sel_op_mov{};
    FF main_sel_op_mul{};
    FF main_sel_op_not{};
    FF main_sel_op_note_hash_exists{};
    FF main_sel_op_nullifier_exists{};
    FF main_sel_op_or{};
    FF main_sel_op_pedersen{};
    FF main_sel_op_poseidon2{};
    FF main_sel_op_radix_le{};
    FF main_sel_op_sender{};
    FF main_sel_op_sha256{};
    FF main_sel_op_shl{};
    FF main_sel_op_shr{};
    FF main_sel_op_sload{};
    FF main_sel_op_sstore{};
    FF main_sel_op_storage_address{};
    FF main_sel_op_sub{};
    FF main_sel_op_timestamp{};
    FF main_sel_op_transaction_fee{};
    FF main_sel_op_version{};
    FF main_sel_op_xor{};
    FF main_sel_q_kernel_lookup{};
    FF main_sel_q_kernel_output_lookup{};
    FF main_sel_resolve_ind_addr_a{};
    FF main_sel_resolve_ind_addr_b{};
    FF main_sel_resolve_ind_addr_c{};
    FF main_sel_resolve_ind_addr_d{};
    FF main_sel_slice_gadget{};
    FF main_space_id{};
    FF main_tag_err{};
    FF main_w_in_tag{};
};

template <typename FF_> class mainImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 151> SUBRELATION_PARTIAL_LENGTHS = {
        3, 3, 3, 3, 3, 3, 5, 5, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 5, 4, 4, 3, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 2, 5, 3, 3, 3, 4, 4, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {
        {
            using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
            auto tmp = (new_term.main_l2_out_of_gas * (-new_term.main_l2_out_of_gas + FF(1)));
            tmp *= scaling_factor;
            std::get<0>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<1, ContainerOverSubrelations>;
            auto tmp = (new_term.main_da_out_of_gas * (-new_term.main_da_out_of_gas + FF(1)));
            tmp *= scaling_factor;
            std::get<1>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<2, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_gas_accounting_active *
                        ((new_term.main_l2_gas_remaining_shift - new_term.main_l2_gas_remaining) +
                         new_term.main_l2_gas_op_cost));
            tmp *= scaling_factor;
            std::get<2>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<3, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_gas_accounting_active *
                        ((new_term.main_da_gas_remaining_shift - new_term.main_da_gas_remaining) +
                         new_term.main_da_gas_op_cost));
            tmp *= scaling_factor;
            std::get<3>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<4, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_gas_accounting_active + FF(1)) * new_term.main_l2_gas_op_cost);
            tmp *= scaling_factor;
            std::get<4>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<5, ContainerOverSubrelations>;
            auto tmp = ((-new_term.main_sel_gas_accounting_active + FF(1)) * new_term.main_da_gas_op_cost);
            tmp *= scaling_factor;
            std::get<5>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<6, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_gas_accounting_active *
                        ((((-(new_term.main_l2_out_of_gas * FF(2)) + FF(1)) * new_term.main_l2_gas_remaining_shift) -
                          (new_term.main_abs_l2_rem_gas_hi * FF(65536))) -
                         new_term.main_abs_l2_rem_gas_lo));
            tmp *= scaling_factor;
            std::get<6>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<7, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_gas_accounting_active *
                        ((((-(new_term.main_da_out_of_gas * FF(2)) + FF(1)) * new_term.main_da_gas_remaining_shift) -
                          (new_term.main_abs_da_rem_gas_hi * FF(65536))) -
                         new_term.main_abs_da_rem_gas_lo));
            tmp *= scaling_factor;
            std::get<7>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<8, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_address * (-new_term.main_sel_op_address + FF(1)));
            tmp *= scaling_factor;
            std::get<8>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<9, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_storage_address * (-new_term.main_sel_op_storage_address + FF(1)));
            tmp *= scaling_factor;
            std::get<9>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<10, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sender * (-new_term.main_sel_op_sender + FF(1)));
            tmp *= scaling_factor;
            std::get<10>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<11, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_function_selector * (-new_term.main_sel_op_function_selector + FF(1)));
            tmp *= scaling_factor;
            std::get<11>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<12, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_transaction_fee * (-new_term.main_sel_op_transaction_fee + FF(1)));
            tmp *= scaling_factor;
            std::get<12>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<13, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_chain_id * (-new_term.main_sel_op_chain_id + FF(1)));
            tmp *= scaling_factor;
            std::get<13>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<14, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_version * (-new_term.main_sel_op_version + FF(1)));
            tmp *= scaling_factor;
            std::get<14>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<15, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_block_number * (-new_term.main_sel_op_block_number + FF(1)));
            tmp *= scaling_factor;
            std::get<15>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<16, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_coinbase * (-new_term.main_sel_op_coinbase + FF(1)));
            tmp *= scaling_factor;
            std::get<16>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<17, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_timestamp * (-new_term.main_sel_op_timestamp + FF(1)));
            tmp *= scaling_factor;
            std::get<17>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<18, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fee_per_l2_gas * (-new_term.main_sel_op_fee_per_l2_gas + FF(1)));
            tmp *= scaling_factor;
            std::get<18>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<19, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fee_per_da_gas * (-new_term.main_sel_op_fee_per_da_gas + FF(1)));
            tmp *= scaling_factor;
            std::get<19>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<20, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_l2gasleft * (-new_term.main_sel_op_l2gasleft + FF(1)));
            tmp *= scaling_factor;
            std::get<20>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<21, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_dagasleft * (-new_term.main_sel_op_dagasleft + FF(1)));
            tmp *= scaling_factor;
            std::get<21>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<22, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_note_hash_exists * (-new_term.main_sel_op_note_hash_exists + FF(1)));
            tmp *= scaling_factor;
            std::get<22>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<23, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_emit_note_hash * (-new_term.main_sel_op_emit_note_hash + FF(1)));
            tmp *= scaling_factor;
            std::get<23>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<24, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_nullifier_exists * (-new_term.main_sel_op_nullifier_exists + FF(1)));
            tmp *= scaling_factor;
            std::get<24>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<25, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_emit_nullifier * (-new_term.main_sel_op_emit_nullifier + FF(1)));
            tmp *= scaling_factor;
            std::get<25>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<26, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_l1_to_l2_msg_exists * (-new_term.main_sel_op_l1_to_l2_msg_exists + FF(1)));
            tmp *= scaling_factor;
            std::get<26>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<27, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_emit_unencrypted_log * (-new_term.main_sel_op_emit_unencrypted_log + FF(1)));
            tmp *= scaling_factor;
            std::get<27>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<28, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_emit_l2_to_l1_msg * (-new_term.main_sel_op_emit_l2_to_l1_msg + FF(1)));
            tmp *= scaling_factor;
            std::get<28>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<29, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_get_contract_instance * (-new_term.main_sel_op_get_contract_instance + FF(1)));
            tmp *= scaling_factor;
            std::get<29>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<30, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sload * (-new_term.main_sel_op_sload + FF(1)));
            tmp *= scaling_factor;
            std::get<30>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<31, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sstore * (-new_term.main_sel_op_sstore + FF(1)));
            tmp *= scaling_factor;
            std::get<31>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<32, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_radix_le * (-new_term.main_sel_op_radix_le + FF(1)));
            tmp *= scaling_factor;
            std::get<32>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<33, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sha256 * (-new_term.main_sel_op_sha256 + FF(1)));
            tmp *= scaling_factor;
            std::get<33>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<34, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_poseidon2 * (-new_term.main_sel_op_poseidon2 + FF(1)));
            tmp *= scaling_factor;
            std::get<34>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<35, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_keccak * (-new_term.main_sel_op_keccak + FF(1)));
            tmp *= scaling_factor;
            std::get<35>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<36, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_pedersen * (-new_term.main_sel_op_pedersen + FF(1)));
            tmp *= scaling_factor;
            std::get<36>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<37, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_add * (-new_term.main_sel_op_add + FF(1)));
            tmp *= scaling_factor;
            std::get<37>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<38, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sub * (-new_term.main_sel_op_sub + FF(1)));
            tmp *= scaling_factor;
            std::get<38>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<39, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_mul * (-new_term.main_sel_op_mul + FF(1)));
            tmp *= scaling_factor;
            std::get<39>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<40, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_div * (-new_term.main_sel_op_div + FF(1)));
            tmp *= scaling_factor;
            std::get<40>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<41, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fdiv * (-new_term.main_sel_op_fdiv + FF(1)));
            tmp *= scaling_factor;
            std::get<41>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<42, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_not * (-new_term.main_sel_op_not + FF(1)));
            tmp *= scaling_factor;
            std::get<42>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<43, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_eq * (-new_term.main_sel_op_eq + FF(1)));
            tmp *= scaling_factor;
            std::get<43>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<44, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_and * (-new_term.main_sel_op_and + FF(1)));
            tmp *= scaling_factor;
            std::get<44>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<45, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_or * (-new_term.main_sel_op_or + FF(1)));
            tmp *= scaling_factor;
            std::get<45>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<46, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_xor * (-new_term.main_sel_op_xor + FF(1)));
            tmp *= scaling_factor;
            std::get<46>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<47, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_cast * (-new_term.main_sel_op_cast + FF(1)));
            tmp *= scaling_factor;
            std::get<47>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<48, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_lt * (-new_term.main_sel_op_lt + FF(1)));
            tmp *= scaling_factor;
            std::get<48>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<49, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_lte * (-new_term.main_sel_op_lte + FF(1)));
            tmp *= scaling_factor;
            std::get<49>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<50, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_shl * (-new_term.main_sel_op_shl + FF(1)));
            tmp *= scaling_factor;
            std::get<50>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<51, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_shr * (-new_term.main_sel_op_shr + FF(1)));
            tmp *= scaling_factor;
            std::get<51>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<52, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_call * (-new_term.main_sel_op_internal_call + FF(1)));
            tmp *= scaling_factor;
            std::get<52>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<53, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_return * (-new_term.main_sel_op_internal_return + FF(1)));
            tmp *= scaling_factor;
            std::get<53>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<54, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_jump * (-new_term.main_sel_op_jump + FF(1)));
            tmp *= scaling_factor;
            std::get<54>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<55, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_jumpi * (-new_term.main_sel_op_jumpi + FF(1)));
            tmp *= scaling_factor;
            std::get<55>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<56, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_halt * (-new_term.main_sel_op_halt + FF(1)));
            tmp *= scaling_factor;
            std::get<56>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<57, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_external_call * (-new_term.main_sel_op_external_call + FF(1)));
            tmp *= scaling_factor;
            std::get<57>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<58, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_calldata_copy * (-new_term.main_sel_op_calldata_copy + FF(1)));
            tmp *= scaling_factor;
            std::get<58>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<59, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_external_return * (-new_term.main_sel_op_external_return + FF(1)));
            tmp *= scaling_factor;
            std::get<59>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<60, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_mov * (-new_term.main_sel_op_mov + FF(1)));
            tmp *= scaling_factor;
            std::get<60>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<61, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_cmov * (-new_term.main_sel_op_cmov + FF(1)));
            tmp *= scaling_factor;
            std::get<61>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<62, ContainerOverSubrelations>;
            auto tmp = (new_term.main_op_err * (-new_term.main_op_err + FF(1)));
            tmp *= scaling_factor;
            std::get<62>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<63, ContainerOverSubrelations>;
            auto tmp = (new_term.main_tag_err * (-new_term.main_tag_err + FF(1)));
            tmp *= scaling_factor;
            std::get<63>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<64, ContainerOverSubrelations>;
            auto tmp = (new_term.main_id_zero * (-new_term.main_id_zero + FF(1)));
            tmp *= scaling_factor;
            std::get<64>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<65, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mem_op_a * (-new_term.main_sel_mem_op_a + FF(1)));
            tmp *= scaling_factor;
            std::get<65>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<66, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mem_op_b * (-new_term.main_sel_mem_op_b + FF(1)));
            tmp *= scaling_factor;
            std::get<66>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<67, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mem_op_c * (-new_term.main_sel_mem_op_c + FF(1)));
            tmp *= scaling_factor;
            std::get<67>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<68, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mem_op_d * (-new_term.main_sel_mem_op_d + FF(1)));
            tmp *= scaling_factor;
            std::get<68>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<69, ContainerOverSubrelations>;
            auto tmp = (new_term.main_rwa * (-new_term.main_rwa + FF(1)));
            tmp *= scaling_factor;
            std::get<69>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<70, ContainerOverSubrelations>;
            auto tmp = (new_term.main_rwb * (-new_term.main_rwb + FF(1)));
            tmp *= scaling_factor;
            std::get<70>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<71, ContainerOverSubrelations>;
            auto tmp = (new_term.main_rwc * (-new_term.main_rwc + FF(1)));
            tmp *= scaling_factor;
            std::get<71>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<72, ContainerOverSubrelations>;
            auto tmp = (new_term.main_rwd * (-new_term.main_rwd + FF(1)));
            tmp *= scaling_factor;
            std::get<72>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<73, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_resolve_ind_addr_a * (-new_term.main_sel_resolve_ind_addr_a + FF(1)));
            tmp *= scaling_factor;
            std::get<73>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<74, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_resolve_ind_addr_b * (-new_term.main_sel_resolve_ind_addr_b + FF(1)));
            tmp *= scaling_factor;
            std::get<74>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<75, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_resolve_ind_addr_c * (-new_term.main_sel_resolve_ind_addr_c + FF(1)));
            tmp *= scaling_factor;
            std::get<75>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<76, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_resolve_ind_addr_d * (-new_term.main_sel_resolve_ind_addr_d + FF(1)));
            tmp *= scaling_factor;
            std::get<76>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<77, ContainerOverSubrelations>;
            auto tmp = (((new_term.main_sel_op_eq + new_term.main_sel_op_lte) + new_term.main_sel_op_lt) *
                        (new_term.main_w_in_tag - FF(1)));
            tmp *= scaling_factor;
            std::get<77>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<78, ContainerOverSubrelations>;
            auto tmp = ((new_term.main_sel_op_fdiv * (-new_term.main_op_err + FF(1))) *
                        ((new_term.main_ic * new_term.main_ib) - new_term.main_ia));
            tmp *= scaling_factor;
            std::get<78>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<79, ContainerOverSubrelations>;
            auto tmp = ((new_term.main_sel_op_fdiv + new_term.main_sel_op_div) *
                        (((new_term.main_ib * new_term.main_inv) - FF(1)) + new_term.main_op_err));
            tmp *= scaling_factor;
            std::get<79>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<80, ContainerOverSubrelations>;
            auto tmp = (((new_term.main_sel_op_fdiv + new_term.main_sel_op_div) * new_term.main_op_err) *
                        (-new_term.main_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<80>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<81, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fdiv * (new_term.main_r_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<81>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<82, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fdiv * (new_term.main_w_in_tag - FF(6)));
            tmp *= scaling_factor;
            std::get<82>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<83, ContainerOverSubrelations>;
            auto tmp = (new_term.main_op_err * ((new_term.main_sel_op_fdiv + new_term.main_sel_op_div) - FF(1)));
            tmp *= scaling_factor;
            std::get<83>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<84, ContainerOverSubrelations>;
            auto tmp = ((((((((((((new_term.main_sel_op_address + new_term.main_sel_op_storage_address) +
                                  new_term.main_sel_op_sender) +
                                 new_term.main_sel_op_function_selector) +
                                new_term.main_sel_op_transaction_fee) +
                               new_term.main_sel_op_chain_id) +
                              new_term.main_sel_op_version) +
                             new_term.main_sel_op_block_number) +
                            new_term.main_sel_op_coinbase) +
                           new_term.main_sel_op_timestamp) +
                          new_term.main_sel_op_fee_per_l2_gas) +
                         new_term.main_sel_op_fee_per_da_gas) *
                        (-new_term.main_sel_q_kernel_lookup + FF(1)));
            tmp *= scaling_factor;
            std::get<84>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<85, ContainerOverSubrelations>;
            auto tmp = (((((((new_term.main_sel_op_note_hash_exists + new_term.main_sel_op_emit_note_hash) +
                             new_term.main_sel_op_nullifier_exists) +
                            new_term.main_sel_op_emit_nullifier) +
                           new_term.main_sel_op_l1_to_l2_msg_exists) +
                          new_term.main_sel_op_emit_unencrypted_log) +
                         new_term.main_sel_op_emit_l2_to_l1_msg) *
                        (-new_term.main_sel_q_kernel_output_lookup + FF(1)));
            tmp *= scaling_factor;
            std::get<85>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<86, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_jump * (new_term.main_pc_shift - new_term.main_ia));
            tmp *= scaling_factor;
            std::get<86>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<87, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_jumpi *
                        (((-new_term.main_id_zero + FF(1)) * (new_term.main_pc_shift - new_term.main_ia)) +
                         (new_term.main_id_zero * ((new_term.main_pc_shift - new_term.main_pc) - FF(1)))));
            tmp *= scaling_factor;
            std::get<87>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<88, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_call *
                        (new_term.main_internal_return_ptr_shift - (new_term.main_internal_return_ptr + FF(1))));
            tmp *= scaling_factor;
            std::get<88>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<89, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_internal_call * (new_term.main_internal_return_ptr - new_term.main_mem_addr_b));
            tmp *= scaling_factor;
            std::get<89>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<90, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_call * (new_term.main_pc_shift - new_term.main_ia));
            tmp *= scaling_factor;
            std::get<90>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<91, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_call * ((new_term.main_pc + FF(1)) - new_term.main_ib));
            tmp *= scaling_factor;
            std::get<91>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<92, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_call * (new_term.main_rwb - FF(1)));
            tmp *= scaling_factor;
            std::get<92>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<93, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_call * (new_term.main_sel_mem_op_b - FF(1)));
            tmp *= scaling_factor;
            std::get<93>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<94, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_return *
                        (new_term.main_internal_return_ptr_shift - (new_term.main_internal_return_ptr - FF(1))));
            tmp *= scaling_factor;
            std::get<94>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<95, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_return *
                        ((new_term.main_internal_return_ptr - FF(1)) - new_term.main_mem_addr_a));
            tmp *= scaling_factor;
            std::get<95>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<96, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_return * (new_term.main_pc_shift - new_term.main_ia));
            tmp *= scaling_factor;
            std::get<96>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<97, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_return * new_term.main_rwa);
            tmp *= scaling_factor;
            std::get<97>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<98, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_internal_return * (new_term.main_sel_mem_op_a - FF(1)));
            tmp *= scaling_factor;
            std::get<98>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<99, ContainerOverSubrelations>;
            auto tmp =
                (((((new_term.main_sel_gas_accounting_active -
                     ((((((((new_term.main_sel_op_fdiv +
                             ((((((((((new_term.main_sel_op_add + new_term.main_sel_op_sub) +
                                      new_term.main_sel_op_mul) +
                                     new_term.main_sel_op_div) +
                                    new_term.main_sel_op_not) +
                                   new_term.main_sel_op_eq) +
                                  new_term.main_sel_op_lt) +
                                 new_term.main_sel_op_lte) +
                                new_term.main_sel_op_shr) +
                               new_term.main_sel_op_shl) +
                              new_term.main_sel_op_cast)) +
                            ((new_term.main_sel_op_and + new_term.main_sel_op_or) + new_term.main_sel_op_xor)) +
                           (new_term.main_sel_op_cmov + new_term.main_sel_op_mov)) +
                          ((((new_term.main_sel_op_radix_le + new_term.main_sel_op_sha256) +
                             new_term.main_sel_op_poseidon2) +
                            new_term.main_sel_op_keccak) +
                           new_term.main_sel_op_pedersen)) +
                         (((((((((((new_term.main_sel_op_address + new_term.main_sel_op_storage_address) +
                                   new_term.main_sel_op_sender) +
                                  new_term.main_sel_op_function_selector) +
                                 new_term.main_sel_op_transaction_fee) +
                                new_term.main_sel_op_chain_id) +
                               new_term.main_sel_op_version) +
                              new_term.main_sel_op_block_number) +
                             new_term.main_sel_op_coinbase) +
                            new_term.main_sel_op_timestamp) +
                           new_term.main_sel_op_fee_per_l2_gas) +
                          new_term.main_sel_op_fee_per_da_gas)) +
                        ((((((new_term.main_sel_op_note_hash_exists + new_term.main_sel_op_emit_note_hash) +
                             new_term.main_sel_op_nullifier_exists) +
                            new_term.main_sel_op_emit_nullifier) +
                           new_term.main_sel_op_l1_to_l2_msg_exists) +
                          new_term.main_sel_op_emit_unencrypted_log) +
                         new_term.main_sel_op_emit_l2_to_l1_msg)) +
                       (new_term.main_sel_op_dagasleft + new_term.main_sel_op_l2gasleft)) +
                      (new_term.main_sel_op_calldata_copy + new_term.main_sel_op_external_return))) -
                    (((new_term.main_sel_op_jump + new_term.main_sel_op_jumpi) + new_term.main_sel_op_internal_call) +
                     new_term.main_sel_op_internal_return)) -
                   new_term.main_sel_op_sload) -
                  new_term.main_sel_op_sstore) -
                 new_term.main_sel_mem_op_activate_gas);
            tmp *= scaling_factor;
            std::get<99>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<100, ContainerOverSubrelations>;
            auto tmp =
                ((((-new_term.main_sel_first + FF(1)) * (-new_term.main_sel_op_halt + FF(1))) *
                  ((((((((new_term.main_sel_op_fdiv +
                          ((((((((((new_term.main_sel_op_add + new_term.main_sel_op_sub) + new_term.main_sel_op_mul) +
                                  new_term.main_sel_op_div) +
                                 new_term.main_sel_op_not) +
                                new_term.main_sel_op_eq) +
                               new_term.main_sel_op_lt) +
                              new_term.main_sel_op_lte) +
                             new_term.main_sel_op_shr) +
                            new_term.main_sel_op_shl) +
                           new_term.main_sel_op_cast)) +
                         ((new_term.main_sel_op_and + new_term.main_sel_op_or) + new_term.main_sel_op_xor)) +
                        (new_term.main_sel_op_cmov + new_term.main_sel_op_mov)) +
                       ((((new_term.main_sel_op_radix_le + new_term.main_sel_op_sha256) +
                          new_term.main_sel_op_poseidon2) +
                         new_term.main_sel_op_keccak) +
                        new_term.main_sel_op_pedersen)) +
                      (((((((((((new_term.main_sel_op_address + new_term.main_sel_op_storage_address) +
                                new_term.main_sel_op_sender) +
                               new_term.main_sel_op_function_selector) +
                              new_term.main_sel_op_transaction_fee) +
                             new_term.main_sel_op_chain_id) +
                            new_term.main_sel_op_version) +
                           new_term.main_sel_op_block_number) +
                          new_term.main_sel_op_coinbase) +
                         new_term.main_sel_op_timestamp) +
                        new_term.main_sel_op_fee_per_l2_gas) +
                       new_term.main_sel_op_fee_per_da_gas)) +
                     ((((((new_term.main_sel_op_note_hash_exists + new_term.main_sel_op_emit_note_hash) +
                          new_term.main_sel_op_nullifier_exists) +
                         new_term.main_sel_op_emit_nullifier) +
                        new_term.main_sel_op_l1_to_l2_msg_exists) +
                       new_term.main_sel_op_emit_unencrypted_log) +
                      new_term.main_sel_op_emit_l2_to_l1_msg)) +
                    (new_term.main_sel_op_dagasleft + new_term.main_sel_op_l2gasleft)) +
                   (new_term.main_sel_op_calldata_copy + new_term.main_sel_op_external_return))) *
                 (new_term.main_pc_shift - (new_term.main_pc + FF(1))));
            tmp *= scaling_factor;
            std::get<100>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<101, ContainerOverSubrelations>;
            auto tmp = ((-(((new_term.main_sel_first + new_term.main_sel_op_internal_call) +
                            new_term.main_sel_op_internal_return) +
                           new_term.main_sel_op_halt) +
                         FF(1)) *
                        (new_term.main_internal_return_ptr_shift - new_term.main_internal_return_ptr));
            tmp *= scaling_factor;
            std::get<101>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<102, ContainerOverSubrelations>;
            auto tmp = ((new_term.main_sel_op_internal_call + new_term.main_sel_op_internal_return) *
                        (new_term.main_space_id - FF(255)));
            tmp *= scaling_factor;
            std::get<102>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<103, ContainerOverSubrelations>;
            auto tmp =
                (((((((((new_term.main_sel_op_fdiv +
                         ((((((((((new_term.main_sel_op_add + new_term.main_sel_op_sub) + new_term.main_sel_op_mul) +
                                 new_term.main_sel_op_div) +
                                new_term.main_sel_op_not) +
                               new_term.main_sel_op_eq) +
                              new_term.main_sel_op_lt) +
                             new_term.main_sel_op_lte) +
                            new_term.main_sel_op_shr) +
                           new_term.main_sel_op_shl) +
                          new_term.main_sel_op_cast)) +
                        ((new_term.main_sel_op_and + new_term.main_sel_op_or) + new_term.main_sel_op_xor)) +
                       (new_term.main_sel_op_cmov + new_term.main_sel_op_mov)) +
                      ((((new_term.main_sel_op_radix_le + new_term.main_sel_op_sha256) +
                         new_term.main_sel_op_poseidon2) +
                        new_term.main_sel_op_keccak) +
                       new_term.main_sel_op_pedersen)) +
                     (((((((((((new_term.main_sel_op_address + new_term.main_sel_op_storage_address) +
                               new_term.main_sel_op_sender) +
                              new_term.main_sel_op_function_selector) +
                             new_term.main_sel_op_transaction_fee) +
                            new_term.main_sel_op_chain_id) +
                           new_term.main_sel_op_version) +
                          new_term.main_sel_op_block_number) +
                         new_term.main_sel_op_coinbase) +
                        new_term.main_sel_op_timestamp) +
                       new_term.main_sel_op_fee_per_l2_gas) +
                      new_term.main_sel_op_fee_per_da_gas)) +
                    ((((((new_term.main_sel_op_note_hash_exists + new_term.main_sel_op_emit_note_hash) +
                         new_term.main_sel_op_nullifier_exists) +
                        new_term.main_sel_op_emit_nullifier) +
                       new_term.main_sel_op_l1_to_l2_msg_exists) +
                      new_term.main_sel_op_emit_unencrypted_log) +
                     new_term.main_sel_op_emit_l2_to_l1_msg)) +
                   (new_term.main_sel_op_dagasleft + new_term.main_sel_op_l2gasleft)) +
                  (new_term.main_sel_op_calldata_copy + new_term.main_sel_op_external_return)) *
                 (new_term.main_call_ptr - new_term.main_space_id));
            tmp *= scaling_factor;
            std::get<103>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<104, ContainerOverSubrelations>;
            auto tmp = ((new_term.main_sel_op_cmov + new_term.main_sel_op_jumpi) *
                        (((new_term.main_id * new_term.main_inv) - FF(1)) + new_term.main_id_zero));
            tmp *= scaling_factor;
            std::get<104>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<105, ContainerOverSubrelations>;
            auto tmp = (((new_term.main_sel_op_cmov + new_term.main_sel_op_jumpi) * new_term.main_id_zero) *
                        (-new_term.main_inv + FF(1)));
            tmp *= scaling_factor;
            std::get<105>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<106, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mov_ia_to_ic -
                        (new_term.main_sel_op_mov + (new_term.main_sel_op_cmov * (-new_term.main_id_zero + FF(1)))));
            tmp *= scaling_factor;
            std::get<106>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<107, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mov_ib_to_ic - (new_term.main_sel_op_cmov * new_term.main_id_zero));
            tmp *= scaling_factor;
            std::get<107>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<108, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mov_ia_to_ic * (new_term.main_ia - new_term.main_ic));
            tmp *= scaling_factor;
            std::get<108>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<109, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_mov_ib_to_ic * (new_term.main_ib - new_term.main_ic));
            tmp *= scaling_factor;
            std::get<109>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<110, ContainerOverSubrelations>;
            auto tmp = ((new_term.main_sel_op_mov + new_term.main_sel_op_cmov) *
                        (new_term.main_r_in_tag - new_term.main_w_in_tag));
            tmp *= scaling_factor;
            std::get<110>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<111, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_alu -
                        ((((((((((((new_term.main_sel_op_add + new_term.main_sel_op_sub) + new_term.main_sel_op_mul) +
                                  new_term.main_sel_op_div) +
                                 new_term.main_sel_op_not) +
                                new_term.main_sel_op_eq) +
                               new_term.main_sel_op_lt) +
                              new_term.main_sel_op_lte) +
                             new_term.main_sel_op_shr) +
                            new_term.main_sel_op_shl) +
                           new_term.main_sel_op_cast) *
                          (-new_term.main_tag_err + FF(1))) *
                         (-new_term.main_op_err + FF(1))));
            tmp *= scaling_factor;
            std::get<111>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<112, ContainerOverSubrelations>;
            auto tmp = ((((((((((new_term.main_sel_op_add + new_term.main_sel_op_sub) + new_term.main_sel_op_mul) +
                               new_term.main_sel_op_div) +
                              new_term.main_sel_op_not) +
                             new_term.main_sel_op_eq) +
                            new_term.main_sel_op_lt) +
                           new_term.main_sel_op_lte) +
                          new_term.main_sel_op_shr) +
                         new_term.main_sel_op_shl) *
                        (new_term.main_alu_in_tag - new_term.main_r_in_tag));
            tmp *= scaling_factor;
            std::get<112>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<113, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_cast * (new_term.main_alu_in_tag - new_term.main_w_in_tag));
            tmp *= scaling_factor;
            std::get<113>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<114, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_l2gasleft * (new_term.main_ia - new_term.main_l2_gas_remaining_shift));
            tmp *= scaling_factor;
            std::get<114>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<115, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_dagasleft * (new_term.main_ia - new_term.main_da_gas_remaining_shift));
            tmp *= scaling_factor;
            std::get<115>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<116, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_address * (new_term.kernel_kernel_in_offset - FF(1)));
            tmp *= scaling_factor;
            std::get<116>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<117, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_storage_address * (new_term.kernel_kernel_in_offset - FF(1)));
            tmp *= scaling_factor;
            std::get<117>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<118, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sender * new_term.kernel_kernel_in_offset);
            tmp *= scaling_factor;
            std::get<118>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<119, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_function_selector * (new_term.kernel_kernel_in_offset - FF(2)));
            tmp *= scaling_factor;
            std::get<119>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<120, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_transaction_fee * (new_term.kernel_kernel_in_offset - FF(39)));
            tmp *= scaling_factor;
            std::get<120>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<121, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_chain_id * (new_term.kernel_kernel_in_offset - FF(28)));
            tmp *= scaling_factor;
            std::get<121>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<122, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_version * (new_term.kernel_kernel_in_offset - FF(29)));
            tmp *= scaling_factor;
            std::get<122>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<123, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_block_number * (new_term.kernel_kernel_in_offset - FF(30)));
            tmp *= scaling_factor;
            std::get<123>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<124, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_timestamp * (new_term.kernel_kernel_in_offset - FF(31)));
            tmp *= scaling_factor;
            std::get<124>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<125, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_coinbase * (new_term.kernel_kernel_in_offset - FF(32)));
            tmp *= scaling_factor;
            std::get<125>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<126, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fee_per_da_gas * (new_term.kernel_kernel_in_offset - FF(34)));
            tmp *= scaling_factor;
            std::get<126>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<127, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_fee_per_l2_gas * (new_term.kernel_kernel_in_offset - FF(35)));
            tmp *= scaling_factor;
            std::get<127>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<128, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_note_hash_exists *
                        (new_term.kernel_kernel_out_offset - (new_term.kernel_note_hash_exist_write_offset + FF(0))));
            tmp *= scaling_factor;
            std::get<128>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<129, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_note_hash_exist_write_offset);
            tmp *= scaling_factor;
            std::get<129>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<130, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_emit_note_hash *
                        (new_term.kernel_kernel_out_offset - (new_term.kernel_emit_note_hash_write_offset + FF(128))));
            tmp *= scaling_factor;
            std::get<130>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<131, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_emit_note_hash_write_offset);
            tmp *= scaling_factor;
            std::get<131>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<132, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_nullifier_exists *
                 (new_term.kernel_kernel_out_offset -
                  ((new_term.main_ib * (new_term.kernel_nullifier_exists_write_offset + FF(16))) +
                   ((-new_term.main_ib + FF(1)) * (new_term.kernel_nullifier_non_exists_write_offset + FF(32))))));
            tmp *= scaling_factor;
            std::get<132>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<133, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_nullifier_exists_write_offset);
            tmp *= scaling_factor;
            std::get<133>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<134, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_nullifier_non_exists_write_offset);
            tmp *= scaling_factor;
            std::get<134>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<135, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_emit_nullifier *
                        (new_term.kernel_kernel_out_offset - (new_term.kernel_emit_nullifier_write_offset + FF(144))));
            tmp *= scaling_factor;
            std::get<135>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<136, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_emit_nullifier_write_offset);
            tmp *= scaling_factor;
            std::get<136>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<137, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_l1_to_l2_msg_exists *
                 (new_term.kernel_kernel_out_offset - (new_term.kernel_l1_to_l2_msg_exists_write_offset + FF(48))));
            tmp *= scaling_factor;
            std::get<137>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<138, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_l1_to_l2_msg_exists_write_offset);
            tmp *= scaling_factor;
            std::get<138>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<139, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_emit_unencrypted_log *
                 (new_term.kernel_kernel_out_offset - (new_term.kernel_emit_unencrypted_log_write_offset + FF(162))));
            tmp *= scaling_factor;
            std::get<139>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<140, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_emit_unencrypted_log_write_offset);
            tmp *= scaling_factor;
            std::get<140>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<141, ContainerOverSubrelations>;
            auto tmp =
                (new_term.main_sel_op_emit_l2_to_l1_msg *
                 (new_term.kernel_kernel_out_offset - (new_term.kernel_emit_l2_to_l1_msg_write_offset + FF(160))));
            tmp *= scaling_factor;
            std::get<141>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<142, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_emit_l2_to_l1_msg_write_offset);
            tmp *= scaling_factor;
            std::get<142>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<143, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sload *
                        (new_term.kernel_kernel_out_offset - (new_term.kernel_sload_write_offset + FF(96))));
            tmp *= scaling_factor;
            std::get<143>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<144, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_sload_write_offset);
            tmp *= scaling_factor;
            std::get<144>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<145, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_op_sstore *
                        (new_term.kernel_kernel_out_offset - (new_term.kernel_sstore_write_offset + FF(64))));
            tmp *= scaling_factor;
            std::get<145>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<146, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_first * new_term.kernel_sstore_write_offset);
            tmp *= scaling_factor;
            std::get<146>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<147, ContainerOverSubrelations>;
            auto tmp = (((((((new_term.main_sel_op_note_hash_exists + new_term.main_sel_op_emit_note_hash) +
                             new_term.main_sel_op_nullifier_exists) +
                            new_term.main_sel_op_emit_nullifier) +
                           new_term.main_sel_op_l1_to_l2_msg_exists) +
                          new_term.main_sel_op_emit_unencrypted_log) +
                         new_term.main_sel_op_emit_l2_to_l1_msg) *
                        (new_term.kernel_side_effect_counter_shift - (new_term.kernel_side_effect_counter + FF(1))));
            tmp *= scaling_factor;
            std::get<147>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<148, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_slice_gadget -
                        ((new_term.main_sel_op_calldata_copy + new_term.main_sel_op_external_return) *
                         (-new_term.main_tag_err + FF(1))));
            tmp *= scaling_factor;
            std::get<148>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<149, ContainerOverSubrelations>;
            auto tmp = (new_term.main_bin_op_id - (new_term.main_sel_op_or + (new_term.main_sel_op_xor * FF(2))));
            tmp *= scaling_factor;
            std::get<149>(evals) += typename Accumulator::View(tmp);
        }
        {
            using Accumulator = typename std::tuple_element_t<150, ContainerOverSubrelations>;
            auto tmp = (new_term.main_sel_bin -
                        ((new_term.main_sel_op_and + new_term.main_sel_op_or) + new_term.main_sel_op_xor));
            tmp *= scaling_factor;
            std::get<150>(evals) += typename Accumulator::View(tmp);
        }
    }
};

template <typename FF> class main : public Relation<mainImpl<FF>> {
  public:
    static constexpr const char* NAME = "main";

    static std::string get_subrelation_label(size_t index)
    {
        switch (index) {
        case 2:
            return "L2_GAS_REMAINING_DECREMENT";
        case 3:
            return "DA_GAS_REMAINING_DECREMENT";
        case 4:
            return "L2_GAS_INACTIVE";
        case 5:
            return "DA_GAS_INACTIVE";
        case 77:
            return "OUTPUT_U8";
        case 78:
            return "SUBOP_FDIV";
        case 79:
            return "SUBOP_FDIV_ZERO_ERR1";
        case 80:
            return "SUBOP_FDIV_ZERO_ERR2";
        case 81:
            return "SUBOP_FDIV_R_IN_TAG_FF";
        case 82:
            return "SUBOP_FDIV_W_IN_TAG_FF";
        case 83:
            return "SUBOP_ERROR_RELEVANT_OP";
        case 84:
            return "KERNEL_INPUT_ACTIVE_CHECK";
        case 85:
            return "KERNEL_OUTPUT_ACTIVE_CHECK";
        case 86:
            return "PC_JUMP";
        case 87:
            return "PC_JUMPI";
        case 88:
            return "RETURN_POINTER_INCREMENT";
        case 94:
            return "RETURN_POINTER_DECREMENT";
        case 100:
            return "PC_INCREMENT";
        case 101:
            return "INTERNAL_RETURN_POINTER_CONSISTENCY";
        case 102:
            return "SPACE_ID_INTERNAL";
        case 103:
            return "SPACE_ID_STANDARD_OPCODES";
        case 104:
            return "CMOV_CONDITION_RES_1";
        case 105:
            return "CMOV_CONDITION_RES_2";
        case 108:
            return "MOV_SAME_VALUE_A";
        case 109:
            return "MOV_SAME_VALUE_B";
        case 110:
            return "MOV_MAIN_SAME_TAG";
        case 114:
            return "L2GASLEFT";
        case 115:
            return "DAGASLEFT";
        case 116:
            return "ADDRESS_KERNEL";
        case 117:
            return "STORAGE_ADDRESS_KERNEL";
        case 118:
            return "SENDER_KERNEL";
        case 119:
            return "FUNCTION_SELECTOR_KERNEL";
        case 120:
            return "FEE_TRANSACTION_FEE_KERNEL";
        case 121:
            return "CHAIN_ID_KERNEL";
        case 122:
            return "VERSION_KERNEL";
        case 123:
            return "BLOCK_NUMBER_KERNEL";
        case 124:
            return "TIMESTAMP_KERNEL";
        case 125:
            return "COINBASE_KERNEL";
        case 126:
            return "FEE_DA_GAS_KERNEL";
        case 127:
            return "FEE_L2_GAS_KERNEL";
        case 128:
            return "NOTE_HASH_KERNEL_OUTPUT";
        case 130:
            return "EMIT_NOTE_HASH_KERNEL_OUTPUT";
        case 132:
            return "NULLIFIER_EXISTS_KERNEL_OUTPUT";
        case 135:
            return "EMIT_NULLIFIER_KERNEL_OUTPUT";
        case 137:
            return "L1_TO_L2_MSG_EXISTS_KERNEL_OUTPUT";
        case 139:
            return "EMIT_UNENCRYPTED_LOG_KERNEL_OUTPUT";
        case 141:
            return "EMIT_L2_TO_L1_MSGS_KERNEL_OUTPUT";
        case 143:
            return "SLOAD_KERNEL_OUTPUT";
        case 145:
            return "SSTORE_KERNEL_OUTPUT";
        case 149:
            return "BIN_SEL_1";
        case 150:
            return "BIN_SEL_2";
        }
        return std::to_string(index);
    }
};

} // namespace bb::Avm_vm