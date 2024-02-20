#include "avm_opcode.hpp"

#include <cstdint>
#include <iomanip>
#include <sstream>

namespace bb::avm_trace {

const std::unordered_map<OpCode, size_t> Bytecode::OPERANDS_NUM = {
    // Compute
    // Compute - Arithmetic
    { OpCode::ADD, 3 },
    { OpCode::SUB, 3 },
    { OpCode::MUL, 3 },
    { OpCode::DIV, 3 },
    //// Compute - Comparators
    //{OpCode::EQ, },
    //{OpCode::LT, },
    //{OpCode::LTE, },
    //// Compute - Bitwise
    //{OpCode::AND, },
    //{OpCode::OR, },
    //{OpCode::XOR, },
    // { OpCode::NOT, 2 },
    //{OpCode::SHL, },
    //{OpCode::SHR, },
    //// Compute - Type Conversions
    //{OpCode::CAST, },

    //// Execution Environment
    //{OpCode::ADDRESS, },
    //{OpCode::STORAGEADDRESS, },
    //{OpCode::ORIGIN, },
    //{OpCode::SENDER, },
    //{OpCode::PORTAL, },
    //{OpCode::FEEPERL1GAS, },
    //{OpCode::FEEPERL2GAS, },
    //{OpCode::FEEPERDAGAS, },
    //{OpCode::CONTRACTCALLDEPTH, },
    //// Execution Environment - Globals
    //{OpCode::CHAINID, },
    //{OpCode::VERSION, },
    //{OpCode::BLOCKNUMBER, },
    //{OpCode::TIMESTAMP, },
    //{OpCode::COINBASE, },
    //{OpCode::BLOCKL1GASLIMIT, },
    //{OpCode::BLOCKL2GASLIMIT, },
    //{OpCode::BLOCKDAGASLIMIT, },
    // Execution Environment - Calldata
    { OpCode::CALLDATACOPY, 3 },

    //// Machine State
    // Machine State - Gas
    //{ OpCode::L1GASLEFT, },
    //{ OpCode::L2GASLEFT, },
    //{ OpCode::DAGASLEFT, },
    //// Machine State - Internal Control Flow
    { OpCode::JUMP, 1 },
    { OpCode::JUMPI, 1 },
    { OpCode::INTERNALCALL, 1 },
    { OpCode::INTERNALRETURN, 0 },

    //// Machine State - Memory
    { OpCode::SET, 5 },
    //{ OpCode::MOV, },
    //{ OpCode::CMOV, },

    //// World State
    //{ OpCode::SLOAD, }, // Public Storage
    //{ OpCode::SSTORE, }, // Public Storage
    //{ OpCode::NOTEHASHEXISTS, }, // Notes & Nullifiers
    //{ OpCode::EMITNOTEHASH, }, // Notes & Nullifiers
    //{ OpCode::NULLIFIEREXISTS, }, // Notes & Nullifiers
    //{ OpCode::EMITNULLIFIER, }, // Notes & Nullifiers
    //{ OpCode::READL1TOL2MSG, }, // Messages
    //{ OpCode::HEADERMEMBER, },

    //// Accrued Substate
    //{ OpCode::EMITUNENCRYPTEDLOG, },
    //{ OpCode::SENDL2TOL1MSG, },

    //// Control Flow - Contract Calls
    //{ OpCode::CALL, },
    //{ OpCode::STATICCALL, },
    //{ OpCode::DELEGATECALL, },
    { OpCode::RETURN, 2 },
    // { OpCode::REVERT, },

    //// Gadgets
    //{ OpCode::KECCAK, },
    //{ OpCode::POSEIDON, },
    //{ OpCode::SHA256, },
    //{ OpCode::PEDERSEN, },
};

/**
 * @brief Test whether a given byte reprents a valid opcode.
 *
 * @param byte The input byte.
 * @return A boolean telling whether a corresponding opcode does match the input byte.
 */
bool Bytecode::is_valid(const uint8_t byte)
{
    return byte <= static_cast<uint8_t>(OpCode::POSEIDON);
}

/**
 * @brief A function returning whether a supplied opcode has an instruction tag as argument.
 *
 * @param op_code The opcode
 * @return A boolean set to true if the corresponding instruction needs a tag as argument.
 */
bool Bytecode::has_in_tag(OpCode const op_code)
{
    switch (op_code) {
    case OpCode::ADDRESS:
    case OpCode::STORAGEADDRESS:
    case OpCode::ORIGIN:
    case OpCode::SENDER:
    case OpCode::PORTAL:
    case OpCode::FEEPERL1GAS:
    case OpCode::FEEPERL2GAS:
    case OpCode::FEEPERDAGAS:
    case OpCode::CONTRACTCALLDEPTH:
    case OpCode::CHAINID:
    case OpCode::VERSION:
    case OpCode::BLOCKNUMBER:
    case OpCode::TIMESTAMP:
    case OpCode::COINBASE:
    case OpCode::BLOCKL1GASLIMIT:
    case OpCode::BLOCKL2GASLIMIT:
    case OpCode::BLOCKDAGASLIMIT:
    case OpCode::CALLDATACOPY:
    case OpCode::L1GASLEFT:
    case OpCode::L2GASLEFT:
    case OpCode::DAGASLEFT:
    case OpCode::JUMP:
    case OpCode::JUMPI:
    case OpCode::INTERNALCALL:
    case OpCode::INTERNALRETURN:
    case OpCode::MOV:
    case OpCode::CMOV:
    case OpCode::SLOAD:
    case OpCode::SSTORE:
    case OpCode::NOTEHASHEXISTS:
    case OpCode::EMITNOTEHASH:
    case OpCode::NULLIFIEREXISTS:
    case OpCode::EMITNULLIFIER:
    case OpCode::READL1TOL2MSG:
    case OpCode::HEADERMEMBER:
    case OpCode::EMITUNENCRYPTEDLOG:
    case OpCode::SENDL2TOL1MSG:
    case OpCode::CALL:
    case OpCode::STATICCALL:
    case OpCode::RETURN:
    case OpCode::REVERT:
        return false;
    default:
        return true;
    }
}

std::string to_hex(OpCode opcode)
{
    std::ostringstream stream;
    // pad with 0s to fill exactly 2 hex characters
    stream << std::setfill('0') << std::setw(2) << std::hex << (static_cast<uint8_t>(opcode) & 0xFF);
    return stream.str();
}

} // namespace bb::avm_trace
