#include "avm_opcode.hpp"

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
    return to_hex(static_cast<uint8_t>(opcode));
}

} // namespace bb::avm_trace
