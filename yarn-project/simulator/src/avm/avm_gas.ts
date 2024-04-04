import { TypeTag } from './avm_memory_types.js';
import { InstructionExecutionError } from './errors.js';
import { Addressing, AddressingMode } from './opcodes/addressing_mode.js';
import { Opcode } from './serialization/instruction_serialization.js';

/** Gas counters in L1, L2, and DA. */
export type Gas = {
  l1Gas: number;
  l2Gas: number;
  daGas: number;
};

/** Maps a Gas struct to gasLeft properties. */
export function gasToGasLeft(gas: Gas) {
  return { l1GasLeft: gas.l1Gas, l2GasLeft: gas.l2Gas, daGasLeft: gas.daGas };
}

/** Maps gasLeft properties to a gas struct. */
export function gasLeftToGas(gasLeft: { l1GasLeft: number; l2GasLeft: number; daGasLeft: number }) {
  return { l1Gas: gasLeft.l1GasLeft, l2Gas: gasLeft.l2GasLeft, daGas: gasLeft.daGasLeft };
}

/** Creates a new instance with all values set to zero except the ones set. */
export function makeGas(gasCost: Partial<Gas>) {
  return { ...EmptyGas, ...gasCost };
}

/** Sums together multiple instances of Gas. */
export function sumGas(...gases: Partial<Gas>[]) {
  return gases.reduce(
    (acc: Gas, gas) => ({
      l1Gas: acc.l1Gas + (gas.l1Gas ?? 0),
      l2Gas: acc.l2Gas + (gas.l2Gas ?? 0),
      daGas: acc.daGas + (gas.daGas ?? 0),
    }),
    EmptyGas,
  );
}

/** Multiplies a gas instance by a scalar. */
export function mulGas(gas: Partial<Gas>, scalar: number) {
  return { l1Gas: (gas.l1Gas ?? 0) * scalar, l2Gas: (gas.l2Gas ?? 0) * scalar, daGas: (gas.daGas ?? 0) * scalar };
}

/** Zero gas across all gas dimensions. */
export const EmptyGas: Gas = {
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

/** Base gas costs for each instruction. Additional gas cost may be added on top due to memory or storage accesses, etc. */
export const GasCosts: Record<Opcode, Gas | typeof DynamicGasCost> = {
  [Opcode.ADD]: TemporaryDefaultGasCost,
  [Opcode.SUB]: TemporaryDefaultGasCost,
  [Opcode.MUL]: TemporaryDefaultGasCost,
  [Opcode.DIV]: TemporaryDefaultGasCost,
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
  [Opcode.CALLDATACOPY]: TemporaryDefaultGasCost,
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
  [Opcode.SET]: TemporaryDefaultGasCost,
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
};

/** Returns the fixed base gas cost for a given opcode, or throws if set to dynamic. */
export function getBaseGasCost(opcode: Opcode): Gas {
  const cost = GasCosts[opcode];
  if (cost === DynamicGasCost) {
    throw new Error(`Opcode ${Opcode[opcode]} has dynamic gas cost`);
  }
  return cost;
}

/** Returns the gas cost associated with the memory operations performed. */
export function getMemoryGasCost(args: { reads?: number; writes?: number; indirect?: number }) {
  const { reads, writes, indirect } = args;
  const indirectCount = Addressing.fromWire(indirect ?? 0).count(AddressingMode.INDIRECT);
  const l2MemoryGasCost =
    (reads ?? 0) * GasCostConstants.MEMORY_READ +
    (writes ?? 0) * GasCostConstants.MEMORY_WRITE +
    indirectCount * GasCostConstants.MEMORY_INDIRECT_READ_PENALTY;
  return makeGas({ l2Gas: l2MemoryGasCost });
}

/** Constants used in base cost calculations. */
export const GasCostConstants = {
  MEMORY_READ: 10,
  MEMORY_INDIRECT_READ_PENALTY: 10,
  MEMORY_WRITE: 100,
};

/** Returns gas cost for an operation on a given type tag based on the base cost per byte. */
export function getGasCostForTypeTag(tag: TypeTag, baseCost: Gas) {
  return mulGas(baseCost, getGasCostMultiplierFromTypeTag(tag));
}

/** Returns a multiplier based on the size of the type represented by the tag. Throws on uninitialized or invalid. */
function getGasCostMultiplierFromTypeTag(tag: TypeTag) {
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
      throw new InstructionExecutionError(`Invalid tag type for gas cost multiplier: ${TypeTag[tag]}`);
  }
}
