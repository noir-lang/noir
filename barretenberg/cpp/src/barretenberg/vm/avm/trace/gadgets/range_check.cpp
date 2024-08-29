
#include "barretenberg/vm/avm/trace/gadgets/range_check.hpp"
namespace bb::avm_trace {

// This function just enqueues a range check event, we handle processing them later in finalize.
bool AvmRangeCheckBuilder::assert_range(uint128_t value, uint8_t num_bits, EventEmitter e, uint64_t clk)
{
    // We don't support range checks on values that are field-sized
    // ASSERT(num_bits <= 128);
    range_check_events.push_back({ clk, uint128_t(value), num_bits, e });
    return true;
}
// Turns range check events into real entries
std::vector<AvmRangeCheckBuilder::RangeCheckEntry> AvmRangeCheckBuilder::finalize()
{
    std::vector<RangeCheckEntry> entries;
    // Process each range check event into entries
    for (auto& event : range_check_events) {
        auto entry = RangeCheckEntry{};
        // Set all the easy stuff
        entry.clk = event.clk;
        entry.value = event.value;
        entry.num_bits = event.num_bits;
        auto value_u256 = uint256_t::from_uint128(event.value);

        // Now some harder stuff, split the value into 16-bit chunks
        for (size_t i = 0; i < 8; i++) {
            // The most significant 16-bits have to be placed in the dynamic slice register
            if (event.num_bits <= 16) {
                entry.dynamic_slice_register = uint16_t(value_u256);
                u16_range_chk_counters[7][entry.dynamic_slice_register]++;
                // Set the bit range flag at this bit range
                entry.bit_range_flag |= 1 << i;
                entry.dyn_bits = event.num_bits;
                break;
            }
            // We have more chunks of 16-bits to operate on, so set the ith fixed register
            entry.fixed_slice_registers[i] = uint16_t(value_u256);
            u16_range_chk_counters[i][uint16_t(value_u256)]++;
            event.num_bits -= 16;
            value_u256 >>= 16;
        }

        // Update the other counters
        powers_of_2_counts[uint8_t(entry.dyn_bits)]++;
        auto dyn_diff = uint16_t((1 << entry.dyn_bits) - entry.dynamic_slice_register - 1);
        entry.dyn_diff = dyn_diff;
        dyn_diff_counts[dyn_diff]++;

        switch (event.emitter) {
        case EventEmitter::ALU:
            entry.is_alu_sel = true;
            break;
        case EventEmitter::MEMORY:
            entry.is_mem_sel = true;
            break;
        case EventEmitter::GAS_L2:
            entry.is_gas_l2_sel = true;
            break;
        case EventEmitter::GAS_DA:
            entry.is_gas_da_sel = true;
            break;
        }
        entries.push_back(entry);
    }
    return entries;
}
} // namespace bb::avm_trace
