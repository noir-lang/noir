#pragma once

#include "barretenberg/vm/avm/generated/relations/range_check.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include <cstdint>

enum class EventEmitter {
    ALU,
    MEMORY,
    GAS_L2,
    GAS_DA,
};

namespace bb::avm_trace {
class AvmRangeCheckBuilder {
  public:
    struct RangeCheckEvent {
        uint64_t clk;
        uint128_t value;
        uint8_t num_bits;
        EventEmitter emitter;
    };

    struct RangeCheckEntry {
        uint64_t clk;
        uint128_t value;
        uint8_t num_bits;
        // 8 total 16-bit registers, the last one is dynamic
        std::array<uint16_t, 7> fixed_slice_registers;
        uint16_t dynamic_slice_register;
        // The number of bits that need to be dynamically checked
        uint16_t dyn_bits;
        // The difference between max value of the dynamic bit range and what is stored in the dyn register
        uint16_t dyn_diff;
        // Bit string representing which of the is_lte_x flags are on
        // From LSB to MSB:
        // [is_lte_u16, is_lte_u32, is_lte_u48, is_lte_u64, is_lte_u80, is_lte_u96, is_lte_u112, is_lte_u128]
        uint8_t bit_range_flag;
        bool is_mem_sel;
        bool is_alu_sel;
        bool is_gas_l2_sel;
        bool is_gas_da_sel;
    };

    std::array<std::unordered_map<uint16_t, uint32_t>, 8> u16_range_chk_counters;
    std::unordered_map<uint8_t, uint32_t> powers_of_2_counts;
    std::unordered_map<uint16_t, uint32_t> dyn_diff_counts;

    // This function just enqueues a range check event, we handle processing them later in finalize.
    bool assert_range(uint128_t value, uint8_t num_bits, EventEmitter e, uint64_t clk);

    // Turns range check events into real entries
    std::vector<RangeCheckEntry> finalize();

    template <typename DestRow> void merge_into(DestRow& row, RangeCheckEntry const& entry)
    {
        row.range_check_clk = entry.clk;
        row.range_check_sel_rng_chk = FF::one();
        row.range_check_value = FF(uint256_t::from_uint128(entry.value));
        row.range_check_rng_chk_bits = entry.num_bits;
        row.range_check_dyn_rng_chk_bits = entry.dyn_bits;
        row.range_check_dyn_rng_chk_pow_2 = 1 << entry.dyn_bits;
        row.range_check_dyn_diff = entry.dyn_diff;

        // The position of the set bit in the bit_range_flag tells us which flag to set
        row.range_check_is_lte_u16 = entry.bit_range_flag & 1;
        row.range_check_is_lte_u32 = entry.bit_range_flag >> 1 & 1;
        row.range_check_is_lte_u48 = entry.bit_range_flag >> 2 & 1;
        row.range_check_is_lte_u64 = entry.bit_range_flag >> 3 & 1;
        row.range_check_is_lte_u80 = entry.bit_range_flag >> 4 & 1;
        row.range_check_is_lte_u96 = entry.bit_range_flag >> 5 & 1;
        row.range_check_is_lte_u112 = entry.bit_range_flag >> 6 & 1;
        row.range_check_is_lte_u128 = entry.bit_range_flag >> 7 & 1;
        // The value of the bit_range_flag tells us registers are part of the range check
        row.range_check_sel_lookup_0 = entry.bit_range_flag >= 2;
        row.range_check_sel_lookup_1 = entry.bit_range_flag >= 4;
        row.range_check_sel_lookup_2 = entry.bit_range_flag >= 8;
        row.range_check_sel_lookup_3 = entry.bit_range_flag >= 16;
        row.range_check_sel_lookup_4 = entry.bit_range_flag >= 32;
        row.range_check_sel_lookup_5 = entry.bit_range_flag >= 64;
        row.range_check_sel_lookup_6 = entry.bit_range_flag >= 128;
        row.range_check_u16_r0 = entry.fixed_slice_registers[0];
        row.range_check_u16_r1 = entry.fixed_slice_registers[1];
        row.range_check_u16_r2 = entry.fixed_slice_registers[2];
        row.range_check_u16_r3 = entry.fixed_slice_registers[3];
        row.range_check_u16_r4 = entry.fixed_slice_registers[4];
        row.range_check_u16_r5 = entry.fixed_slice_registers[5];
        row.range_check_u16_r6 = entry.fixed_slice_registers[6];
        row.range_check_u16_r7 = entry.dynamic_slice_register;

        row.range_check_mem_rng_chk = entry.is_mem_sel;
        row.range_check_gas_l2_rng_chk = entry.is_gas_l2_sel;
        row.range_check_gas_da_rng_chk = entry.is_gas_da_sel;
    }

  private:
    std::vector<RangeCheckEvent> range_check_events;
};
} // namespace bb::avm_trace
