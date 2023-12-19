#pragma once

#include "barretenberg/proof_system/circuit_builder/AvmMini_trace.hpp"

#define EXPECT_THROW_WITH_MESSAGE(code, expectedMessage)                                                               \
    try {                                                                                                              \
        code;                                                                                                          \
        FAIL() << "An exception was expected";                                                                         \
    } catch (const std::exception& e) {                                                                                \
        std::string message = e.what();                                                                                \
        EXPECT_TRUE(message.find(expectedMessage) != std::string::npos);                                               \
    }

namespace tests_avm {

void validateTraceProof(std::vector<Row>&& trace);
void mutateIcInTrace(std::vector<Row>& trace, std::function<bool(Row)>&& selectRow, FF const& newValue);

} // namespace tests_avm