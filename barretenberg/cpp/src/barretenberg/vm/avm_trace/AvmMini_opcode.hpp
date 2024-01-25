#pragma once

#include <cstdint>
#include <unordered_map>

namespace avm_trace {
using std::size_t;

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
    ORIGIN,
    SENDER,
    PORTAL,
    FEEPERL1GAS,
    FEEPERL2GAS,
    FEEPERDAGAS,
    CONTRACTCALLDEPTH,
    // Execution Environment - Globals
    CHAINID,
    VERSION,
    BLOCKNUMBER,
    TIMESTAMP,
    COINBASE,
    BLOCKL1GASLIMIT,
    BLOCKL2GASLIMIT,
    BLOCKDAGASLIMIT,
    // Execution Environment - Calldata
    CALLDATACOPY,

    // Machine State
    // Machine State - Gas
    L1GASLEFT,
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
    BLOCKHEADERBYNUMBER,
    SLOAD,         // Public Storage
    SSTORE,        // Public Storage
    READL1TOL2MSG, // Messages
    SENDL2TOL1MSG, // Messages
    EMITNOTEHASH,  // Notes & Nullifiers
    EMITNULLIFIER, // Notes & Nullifiers

    // Accrued Substate
    EMITUNENCRYPTEDLOG,

    // Control Flow - Contract Calls
    CALL,
    STATICCALL,
    RETURN,
    REVERT,

    // Gadgets
    KECCAK,
    POSEIDON,
};

class Bytecode {
  public:
    static bool is_valid(uint8_t byte);
    static bool has_in_tag(OpCode);
    static const std::unordered_map<OpCode, size_t> OPERANDS_NUM;
};

} // namespace avm_trace