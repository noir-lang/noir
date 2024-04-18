#include "avm_opcode.hpp"

#include <cstdint>
#include <iomanip>
#include <sstream>

namespace bb::avm_trace {

/**
 * @brief Test whether a given byte represents a valid opcode.
 *
 * @param byte The input byte.
 * @return A boolean telling whether a corresponding opcode does match the input byte.
 */
bool Bytecode::is_valid(const uint8_t byte)
{
    return byte < static_cast<uint8_t>(OpCode::LAST_OPCODE_SENTINEL);
}

std::string to_hex(OpCode opcode)
{
    std::ostringstream stream;
    // pad with 0s to fill exactly 2 hex characters
    stream << std::setfill('0') << std::setw(2) << std::hex << (static_cast<uint8_t>(opcode) & 0xFF);
    return stream.str();
}

} // namespace bb::avm_trace
