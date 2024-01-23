#pragma once

#include "barretenberg/vm/avm_trace/AvmMini_trace.hpp"

#define EXPECT_THROW_WITH_MESSAGE(code, expectedMessage)                                                               \
    try {                                                                                                              \
        code;                                                                                                          \
        FAIL() << "An exception was expected";                                                                         \
    } catch (const std::exception& e) {                                                                                \
        std::string message = e.what();                                                                                \
        EXPECT_TRUE(message.find(expectedMessage) != std::string::npos);                                               \
    }
namespace tests_avm {
void validate_trace_proof(std::vector<Row>&& trace);
void mutate_ic_in_trace(std::vector<Row>& trace,
                        std::function<bool(Row)>&& selectRow,
                        FF const& newValue,
                        bool alu = false);

} // namespace tests_avm