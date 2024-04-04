import { TypeTag } from './avm_memory_types.js';
import { Opcode } from './serialization/instruction_serialization.js';

/** Gas cost in L1, L2, and DA for a given instruction. */
export type GasCost = {
  l1Gas: number;
  l2Gas: number;
  daGas: number;
};

/** Creates a new instance with all values set to zero except the ones set. */
export function makeGasCost(gasCost: Partial<GasCost>) {
  return { ...EmptyGasCost, ...gasCost };
}

/** Gas cost of zero across all gas dimensions. */
export const EmptyGasCost = {
  l1Gas: 0,
  l2Gas: 0,
  daGas: 0,
};

/** Dimensions of gas usage: L1, L2, and DA. */
export const GasDimensions = ['l1Gas', 'l2Gas', 'daGas'] as const;

/** Null object to represent a gas cost that's dynamic instead of fixed for a given instruction. */
export const DynamicGasCost = Symbol('DynamicGasCost');

/** Temporary default gas cost. We should eventually remove all usage of this variable in favor of actual gas for each opcode. */
const TemporaryDefaultGasCost = { l1Gas: 0, l2Gas: 10, daGas: 0 };

/** Gas costs for each instruction. */
export const GasCosts = {
  [Opcode.ADD]: DynamicGasCost,
  [Opcode.SUB]: DynamicGasCost,
  [Opcode.MUL]: DynamicGasCost,
  [Opcode.DIV]: DynamicGasCost,
  [Opcode.FDIV]: TemporaryDefaultGasCost,
  [Opcode.EQ]: TemporaryDefaultGasCost,
  [Opcode.LT]: TemporaryDefaultGasCost,
  [Opcode.LTE]: TemporaryDefaultGasCost,
  [Opcode.AND]: TemporaryDefaultGasCost,
  [Opcode.OR]: TemporaryDefaultGasCost,
  [Opcode.XOR]: TemporaryDefaultGasCost,
  [Opcode.NOT]: TemporaryDefaultGasCost,
  [Opcode.SHL]: TemporaryDefaultGasCost,
  [Opcode.SHR]: TemporaryDefaultGasCost,
  [Opcode.CAST]: TemporaryDefaultGasCost,
  // Execution environment
  [Opcode.ADDRESS]: TemporaryDefaultGasCost,
  [Opcode.STORAGEADDRESS]: TemporaryDefaultGasCost,
  [Opcode.ORIGIN]: TemporaryDefaultGasCost,
  [Opcode.SENDER]: TemporaryDefaultGasCost,
  [Opcode.PORTAL]: TemporaryDefaultGasCost,
  [Opcode.FEEPERL1GAS]: TemporaryDefaultGasCost,
  [Opcode.FEEPERL2GAS]: TemporaryDefaultGasCost,
  [Opcode.FEEPERDAGAS]: TemporaryDefaultGasCost,
  [Opcode.CONTRACTCALLDEPTH]: TemporaryDefaultGasCost,
  [Opcode.CHAINID]: TemporaryDefaultGasCost,
  [Opcode.VERSION]: TemporaryDefaultGasCost,
  [Opcode.BLOCKNUMBER]: TemporaryDefaultGasCost,
  [Opcode.TIMESTAMP]: TemporaryDefaultGasCost,
  [Opcode.COINBASE]: TemporaryDefaultGasCost,
  [Opcode.BLOCKL1GASLIMIT]: TemporaryDefaultGasCost,
  [Opcode.BLOCKL2GASLIMIT]: TemporaryDefaultGasCost,
  [Opcode.BLOCKDAGASLIMIT]: TemporaryDefaultGasCost,
  [Opcode.CALLDATACOPY]: DynamicGasCost,
  // Gas
  [Opcode.L1GASLEFT]: TemporaryDefaultGasCost,
  [Opcode.L2GASLEFT]: TemporaryDefaultGasCost,
  [Opcode.DAGASLEFT]: TemporaryDefaultGasCost,
  // Control flow
  [Opcode.JUMP]: TemporaryDefaultGasCost,
  [Opcode.JUMPI]: TemporaryDefaultGasCost,
  [Opcode.INTERNALCALL]: TemporaryDefaultGasCost,
  [Opcode.INTERNALRETURN]: TemporaryDefaultGasCost,
  // Memory
  [Opcode.SET]: DynamicGasCost,
  [Opcode.MOV]: TemporaryDefaultGasCost,
  [Opcode.CMOV]: TemporaryDefaultGasCost,
  // World state
  [Opcode.SLOAD]: TemporaryDefaultGasCost,
  [Opcode.SSTORE]: TemporaryDefaultGasCost,
  [Opcode.NOTEHASHEXISTS]: TemporaryDefaultGasCost,
  [Opcode.EMITNOTEHASH]: TemporaryDefaultGasCost,
  [Opcode.NULLIFIEREXISTS]: TemporaryDefaultGasCost,
  [Opcode.EMITNULLIFIER]: TemporaryDefaultGasCost,
  [Opcode.L1TOL2MSGEXISTS]: TemporaryDefaultGasCost,
  [Opcode.HEADERMEMBER]: TemporaryDefaultGasCost,
  [Opcode.EMITUNENCRYPTEDLOG]: TemporaryDefaultGasCost,
  [Opcode.SENDL2TOL1MSG]: TemporaryDefaultGasCost,
  [Opcode.GETCONTRACTINSTANCE]: TemporaryDefaultGasCost,
  // External calls
  [Opcode.CALL]: TemporaryDefaultGasCost,
  [Opcode.STATICCALL]: TemporaryDefaultGasCost,
  [Opcode.DELEGATECALL]: TemporaryDefaultGasCost,
  [Opcode.RETURN]: TemporaryDefaultGasCost,
  [Opcode.REVERT]: TemporaryDefaultGasCost,
  // Gadgets
  [Opcode.KECCAK]: TemporaryDefaultGasCost,
  [Opcode.POSEIDON]: TemporaryDefaultGasCost,
  [Opcode.SHA256]: TemporaryDefaultGasCost, // temp - may be removed, but alot of contracts rely on i: TemporaryDefaultGasCost,
  [Opcode.PEDERSEN]: TemporaryDefaultGasCost, // temp - may be removed, but alot of contracts rely on i: TemporaryDefaultGasCost,t
} as const;

/** Constants used in base cost calculations. */
export const GasCostConstants = {
  SET_COST_PER_BYTE: 100,
  CALLDATACOPY_COST_PER_BYTE: 10,
  ARITHMETIC_COST_PER_BYTE: 10,
  ARITHMETIC_COST_PER_INDIRECT_ACCESS: 5,
};

/** Returns a multiplier based on the size of the type represented by the tag. Throws on uninitialized or invalid. */
export function getGasCostMultiplierFromTypeTag(tag: TypeTag) {
  switch (tag) {
    case TypeTag.UINT8:
      return 1;
    case TypeTag.UINT16:
      return 2;
    case TypeTag.UINT32:
      return 4;
    case TypeTag.UINT64:
      return 8;
    case TypeTag.UINT128:
      return 16;
    case TypeTag.FIELD:
      return 32;
    case TypeTag.INVALID:
    case TypeTag.UNINITIALIZED:
      throw new Error(`Invalid tag type for gas cost multiplier: ${TypeTag[tag]}`);
  }
}
