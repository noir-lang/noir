#include "barretenberg/vm/avm/trace/fixed_bytes.hpp"

namespace bb::avm_trace {

// Singleton.
const FixedBytesTable& FixedBytesTable::get()
{
    static FixedBytesTable table;
    return table;
}

void FixedBytesTable::finalize(std::vector<AvmFullRow<FF>>& main_trace) const
{
    if (main_trace.size() < 3 * (1 << 16)) {
        main_trace.resize(3 * (1 << 16));
    }
    // Generate Lookup Table of all combinations of 2, 8-bit numbers and op_id.
    for (uint32_t op_id = 0; op_id < 3; op_id++) {
        for (uint32_t input_a = 0; input_a <= UINT8_MAX; input_a++) {
            for (uint32_t input_b = 0; input_b <= UINT8_MAX; input_b++) {
                auto a = static_cast<uint8_t>(input_a);
                auto b = static_cast<uint8_t>(input_b);

                // Derive a unique row index given op_id, a, and b.
                auto main_trace_index = (op_id << 16) + (input_a << 8) + b;

                main_trace.at(main_trace_index).byte_lookup_sel_bin = FF(1);
                main_trace.at(main_trace_index).byte_lookup_table_op_id = op_id;
                main_trace.at(main_trace_index).byte_lookup_table_input_a = a;
                main_trace.at(main_trace_index).byte_lookup_table_input_b = b;
            }
        }
    }

    finalize_byte_length(main_trace);
}

void FixedBytesTable::finalize_for_testing(std::vector<AvmFullRow<FF>>& main_trace,
                                           const std::unordered_map<uint32_t, uint32_t>& byte_operation_counter) const
{
    // Generate ByteLength Lookup table of instruction tags to the number of bytes
    // {U8: 1, U16: 2, U32: 4, U64: 8, U128: 16}
    for (auto const& [clk, count] : byte_operation_counter) {
        // from the clk we can derive the a and b inputs
        auto b = static_cast<uint8_t>(clk);
        auto a = static_cast<uint8_t>(clk >> 8);
        auto op_id = static_cast<uint8_t>(clk >> 16);
        uint8_t bit_op = 0;
        if (op_id == 0) {
            bit_op = a & b;
        } else if (op_id == 1) {
            bit_op = a | b;
        } else {
            bit_op = a ^ b;
        }
        if (clk > (main_trace.size() - 1)) {
            main_trace.push_back(AvmFullRow<FF>{
                .byte_lookup_sel_bin = FF(1),
                .byte_lookup_table_input_a = a,
                .byte_lookup_table_input_b = b,
                .byte_lookup_table_op_id = op_id,
                .byte_lookup_table_output = bit_op,
                .main_clk = FF(clk),
                .lookup_byte_operations_counts = count,
            });
        } else {
            main_trace.at(clk).lookup_byte_operations_counts = count;
            main_trace.at(clk).byte_lookup_sel_bin = FF(1);
            main_trace.at(clk).byte_lookup_table_op_id = op_id;
            main_trace.at(clk).byte_lookup_table_input_a = a;
            main_trace.at(clk).byte_lookup_table_input_b = b;
            main_trace.at(clk).byte_lookup_table_output = bit_op;
        }
        // Add the counter value stored throughout the execution
    }

    finalize_byte_length(main_trace);
}

void FixedBytesTable::finalize_byte_length(std::vector<AvmFullRow<FF>>& main_trace)
{
    // Generate ByteLength Lookup table of instruction tags to the number of bytes
    // {U8: 1, U16: 2, U32: 4, U64: 8, U128: 16}
    for (uint8_t avm_in_tag = 0; avm_in_tag < 5; avm_in_tag++) {
        // The +1 here is because the instruction tags we care about (i.e excl U0 and FF) has the range 1,5]
        main_trace.at(avm_in_tag).byte_lookup_sel_bin = FF(1);
        main_trace.at(avm_in_tag).byte_lookup_table_in_tags = avm_in_tag + 1;
        main_trace.at(avm_in_tag).byte_lookup_table_byte_lengths = static_cast<uint8_t>(1 << avm_in_tag);
    }
}

} // namespace bb::avm_trace