#pragma once

#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"
#include "gmock/gmock.h"
#include <array>

#define EXPECT_THROW_WITH_MESSAGE(code, expectedMessage)                                                               \
    EXPECT_DEATH(                                                                                                      \
        try {                                                                                                          \
            code;                                                                                                      \
            FAIL() << "An exception was expected";                                                                     \
        } catch (const std::exception& e) {                                                                            \
            std::cerr << e.what();                                                                                     \
            std::abort();                                                                                              \
        },                                                                                                             \
        expectedMessage);

#define MAIN_ROW_FIELD_EQ(field_name, expression) Field(#field_name, &Row::main_##field_name, expression)
#define MEM_ROW_FIELD_EQ(field_name, expression) Field(#field_name, &Row::mem_##field_name, expression)

namespace tests_avm {

using FF = bb::AvmFlavorSettings::FF;
using Row = bb::AvmFullRow<bb::fr>;
using ThreeOpParam = std::array<FF, 3>;
using ThreeOpParamRow = std::tuple<ThreeOpParam, bb::avm_trace::AvmMemoryTag>;
using VmPublicInputs = bb::avm_trace::VmPublicInputs;

// If the test is expecting a relation to fail, then use validate_trace_check_circuit.
// Otherwise, use validate_trace with a single argument. If the proving needs to be
// enabled all the time in a given test, use validate_trace with setting with_proof = true.
void validate_trace_check_circuit(std::vector<Row>&& trace);
void validate_trace(std::vector<Row>&& trace,
                    VmPublicInputs const& public_inputs = {},
                    std::vector<FF> const& calldata = {},
                    std::vector<FF> const& returndata = {},
                    bool with_proof = bb::avm_trace::ENABLE_PROVING,
                    bool expect_proof_failure = false);
void mutate_ic_in_trace(std::vector<Row>& trace,
                        std::function<bool(Row)>&& selectRow,
                        FF const& newValue,
                        bool alu = false);
void clear_range_check_counters(std::vector<Row>& trace, uint256_t previous_value);
void update_slice_registers(Row& row, uint256_t a);
std::vector<ThreeOpParamRow> gen_three_op_params(std::vector<std::array<FF, 3>> operands,
                                                 std::vector<bb::avm_trace::AvmMemoryTag> mem_tags);

VmPublicInputs generate_base_public_inputs();

} // namespace tests_avm
