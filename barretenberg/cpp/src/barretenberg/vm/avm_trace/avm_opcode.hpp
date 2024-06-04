#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"

#include <concepts>
#include <cstddef>
#include <cstdint>
#include <iomanip>
#include <sstream>
#include <string>

namespace bb::avm_trace {

/**
 * All AVM opcodes (Keep in sync with TS counterpart code opcodes.ts)
 * TODO: Once opcode values are definitive, we should assign them explicitly in the enum below
 *       and typescript code. This would increase robustness against unintended modifications.
 *       i.e.: ADD = 0, SUB = 1, etc, ....
 * CAUTION: Any change in the list below needs to be carefully followed by
 *          a potential adaptation of Bytecode::is_valid method.
 */
enum class OpCode : uint8_t {
    // Compute
    // Compute - Arithmetic
    ADD,
    SUB,
    MUL,
    DIV,
    FDIV,
    // Compute - Comparators
    EQ,
    LT,
    LTE,
    // Compute - Bitwise
    AND,
    OR,
    XOR,
    NOT,
    SHL,
    SHR,
    // Compute - Type Conversions
    CAST,

    // Execution Environment
    ADDRESS,
    STORAGEADDRESS,
    SENDER,
    FEEPERL2GAS,
    FEEPERDAGAS,
    TRANSACTIONFEE,
    CONTRACTCALLDEPTH,
    // Execution Environment - Globals
    CHAINID,
    VERSION,
    BLOCKNUMBER,
    TIMESTAMP,
    COINBASE,
    BLOCKL2GASLIMIT,
    BLOCKDAGASLIMIT,
    // Execution Environment - Calldata
    CALLDATACOPY,

    // Machine State
    // Machine State - Gas
    L2GASLEFT,
    DAGASLEFT,
    // Machine State - Internal Control Flow
    JUMP,
    JUMPI,
    INTERNALCALL,
    INTERNALRETURN,
    // Machine State - Memory
    SET,
    MOV,
    CMOV,

    // World State
    SLOAD,           // Public Storage
    SSTORE,          // Public Storage
    NOTEHASHEXISTS,  // Notes & Nullifiers
    EMITNOTEHASH,    // Notes & Nullifiers
    NULLIFIEREXISTS, // Notes & Nullifiers
    EMITNULLIFIER,   // Notes & Nullifiers
    L1TOL2MSGEXISTS, // Messages
    HEADERMEMBER,    // Archive tree & Headers
    GETCONTRACTINSTANCE,

    // Accrued Substate
    EMITUNENCRYPTEDLOG,
    SENDL2TOL1MSG, // Messages

    // Control Flow - Contract Calls
    CALL,
    STATICCALL,
    DELEGATECALL,
    RETURN,
    REVERT,

    // Misc
    DEBUGLOG,

    // Gadgets
    KECCAK,
    POSEIDON2,
    SHA256,
    PEDERSEN,
    // Conversions
    TORADIXLE,
    // Future Gadgets -- pending changes in noir
    SHA256COMPRESSION,
    KECCAKF1600, // Here for when we eventually support this

    // Sentinel
    LAST_OPCODE_SENTINEL,
};

class Bytecode {
  public:
    static bool is_valid(uint8_t byte);
};

// Look into whether we can do something with concepts here to enable OpCode as a parameter
template <typename T>
    requires(std::unsigned_integral<T>)
std::string to_hex(T value)
{
    std::ostringstream stream;
    auto num_bytes = static_cast<uint64_t>(sizeof(T));
    auto mask = static_cast<uint64_t>((static_cast<uint128_t>(1) << (num_bytes * 8)) - 1);
    auto padding = static_cast<int>(num_bytes * 2);
    stream << std::setfill('0') << std::setw(padding) << std::hex << (value & mask);
    return stream.str();
}
std::string to_hex(OpCode opcode);

std::string to_string(OpCode opcode);

} // namespace bb::avm_trace
