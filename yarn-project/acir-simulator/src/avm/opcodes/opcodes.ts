/**
 * All AVM opcodes
 */
export enum Opcode {
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
  SLOAD, // Public Storage
  SSTORE, // Public Storage
  READL1TOL2MSG, // Messages
  SENDL2TOL1MSG, // Messages
  EMITNOTEHASH, // Notes & Nullifiers
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
}
