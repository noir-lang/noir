/**
 * All AVM opcodes. (Keep in sync with cpp counterpart code AvmMini_opcode.hpp).
 * Source: https://yp-aztec.netlify.app/docs/public-vm/instruction-set
 */
export enum Opcode {
  // Compute
  // Compute - Arithmetic
  ADD = 0x00,
  SUB = 0x01,
  MUL = 0x02,
  DIV = 0x03,
  // Compute - Comparators
  EQ = 0x04,
  LT = 0x05,
  LTE = 0x06,
  // Compute - Bitwise
  AND = 0x07,
  OR = 0x08,
  XOR = 0x09,
  NOT = 0x0a,
  SHL = 0x0b,
  SHR = 0x0c,
  // Compute - Type Conversions
  CAST = 0x0d,

  // Execution Environment
  ADDRESS = 0x0e,
  STORAGEADDRESS = 0x0f,
  ORIGIN = 0x10,
  SENDER = 0x11,
  PORTAL = 0x12,
  FEEPERL1GAS = 0x13,
  FEEPERL2GAS = 0x14,
  FEEPERDAGAS = 0x15,
  CONTRACTCALLDEPTH = 0x16,
  // Execution Environment - Globals
  CHAINID = 0x17,
  VERSION = 0x18,
  BLOCKNUMBER = 0x19,
  TIMESTAMP = 0x1a,
  COINBASE = 0x1b,
  BLOCKL1GASLIMIT = 0x1c,
  BLOCKL2GASLIMIT = 0x1d,
  BLOCKDAGASLIMIT = 0x1e,
  // Execution Environment - Calldata
  CALLDATACOPY = 0x1f,

  // Machine State
  // Machine State - Gas
  L1GASLEFT = 0x20,
  L2GASLEFT = 0x21,
  DAGASLEFT = 0x22,
  // Machine State - Internal Control Flow
  JUMP = 0x23,
  JUMPI = 0x24,
  INTERNALCALL = 0x25,
  INTERNALRETURN = 0x26,
  // Machine State - Memory
  SET = 0x27,
  MOV = 0x28,
  CMOV = 0x29,

  // World State
  BLOCKHEADERBYNUMBER = 0x2a,
  SLOAD = 0x2b, // Public Storage
  SSTORE = 0x2c, // Public Storage
  READL1TOL2MSG = 0x2d, // Messages
  SENDL2TOL1MSG = 0x2e, // Messages
  EMITNOTEHASH = 0x2f, // Notes & Nullifiers
  EMITNULLIFIER = 0x30, // Notes & Nullifiers

  // Accrued Substate
  EMITUNENCRYPTEDLOG = 0x31,

  // Control Flow - Contract Calls
  CALL = 0x32,
  STATICCALL = 0x33,
  RETURN = 0x34,
  REVERT = 0x35,

  // Gadgets
  KECCAK = 0x36,
  POSEIDON = 0x37,

  // Add new opcodes before this
  TOTAL_OPCODES_NUMBER,
}
