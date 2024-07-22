import { TypeTag } from './avm_memory_types.js';
import { InstructionExecutionError } from './errors.js';
import { Addressing, AddressingMode } from './opcodes/addressing_mode.js';
import { Opcode } from './serialization/instruction_serialization.js';

/** Gas counters in L1, L2, and DA. */
export type Gas = {
  l2Gas: number;
  daGas: number;
};

/** Maps a Gas struct to gasLeft properties. */
export function gasToGasLeft(gas: Gas) {
  return { l2GasLeft: gas.l2Gas, daGasLeft: gas.daGas };
}

/** Maps gasLeft properties to a gas struct. */
export function gasLeftToGas(gasLeft: { l2GasLeft: number; daGasLeft: number }) {
  return { l2Gas: gasLeft.l2GasLeft, daGas: gasLeft.daGasLeft };
}

/** Creates a new instance with all values set to zero except the ones set. */
export function makeGas(gasCost: Partial<Gas>) {
  return { ...EmptyGas, ...gasCost };
}

/** Sums together multiple instances of Gas. */
export function sumGas(...gases: Partial<Gas>[]) {
  return gases.reduce(
    (acc: Gas, gas) => ({
      l2Gas: acc.l2Gas + (gas.l2Gas ?? 0),
      daGas: acc.daGas + (gas.daGas ?? 0),
    }),
    EmptyGas,
  );
}

/** Multiplies a gas instance by a scalar. */
export function mulGas(gas: Partial<Gas>, scalar: number) {
  return { l2Gas: (gas.l2Gas ?? 0) * scalar, daGas: (gas.daGas ?? 0) * scalar };
}

/** Zero gas across all gas dimensions. */
export const EmptyGas: Gas = {
  l2Gas: 0,
  daGas: 0,
};

/** Dimensions of gas usage: L1, L2, and DA. */
export const GasDimensions = ['l2Gas', 'daGas'] as const;

/** Default gas cost for an opcode. */
const DefaultBaseGasCost: Gas = { l2Gas: 10, daGas: 0 };

/** Base gas costs for each instruction. Additional gas cost may be added on top due to memory or storage accesses, etc. */
const BaseGasCosts: Record<Opcode, Gas> = {
  [Opcode.ADD]: DefaultBaseGasCost,
  [Opcode.SUB]: DefaultBaseGasCost,
  [Opcode.MUL]: DefaultBaseGasCost,
  [Opcode.DIV]: DefaultBaseGasCost,
  [Opcode.FDIV]: DefaultBaseGasCost,
  [Opcode.EQ]: DefaultBaseGasCost,
  [Opcode.LT]: DefaultBaseGasCost,
  [Opcode.LTE]: DefaultBaseGasCost,
  [Opcode.AND]: DefaultBaseGasCost,
  [Opcode.OR]: DefaultBaseGasCost,
  [Opcode.XOR]: DefaultBaseGasCost,
  [Opcode.NOT]: DefaultBaseGasCost,
  [Opcode.SHL]: DefaultBaseGasCost,
  [Opcode.SHR]: DefaultBaseGasCost,
  [Opcode.CAST]: DefaultBaseGasCost,
  // Execution environment
  [Opcode.ADDRESS]: DefaultBaseGasCost,
  [Opcode.STORAGEADDRESS]: DefaultBaseGasCost,
  [Opcode.SENDER]: DefaultBaseGasCost,
  [Opcode.FEEPERL2GAS]: DefaultBaseGasCost,
  [Opcode.FEEPERDAGAS]: DefaultBaseGasCost,
  [Opcode.TRANSACTIONFEE]: DefaultBaseGasCost,
  [Opcode.FUNCTIONSELECTOR]: DefaultBaseGasCost,
  [Opcode.CHAINID]: DefaultBaseGasCost,
  [Opcode.VERSION]: DefaultBaseGasCost,
  [Opcode.BLOCKNUMBER]: DefaultBaseGasCost,
  [Opcode.TIMESTAMP]: DefaultBaseGasCost,
  [Opcode.COINBASE]: DefaultBaseGasCost,
  [Opcode.BLOCKL2GASLIMIT]: DefaultBaseGasCost,
  [Opcode.BLOCKDAGASLIMIT]: DefaultBaseGasCost,
  [Opcode.CALLDATACOPY]: DefaultBaseGasCost,
  // Gas
  [Opcode.L2GASLEFT]: DefaultBaseGasCost,
  [Opcode.DAGASLEFT]: DefaultBaseGasCost,
  // Control flow
  [Opcode.JUMP]: DefaultBaseGasCost,
  [Opcode.JUMPI]: DefaultBaseGasCost,
  [Opcode.INTERNALCALL]: DefaultBaseGasCost,
  [Opcode.INTERNALRETURN]: DefaultBaseGasCost,
  // Memory
  [Opcode.SET]: DefaultBaseGasCost,
  [Opcode.MOV]: DefaultBaseGasCost,
  [Opcode.CMOV]: DefaultBaseGasCost,
  // World state
  [Opcode.SLOAD]: DefaultBaseGasCost,
  [Opcode.SSTORE]: DefaultBaseGasCost,
  [Opcode.NOTEHASHEXISTS]: DefaultBaseGasCost,
  [Opcode.EMITNOTEHASH]: DefaultBaseGasCost,
  [Opcode.NULLIFIEREXISTS]: DefaultBaseGasCost,
  [Opcode.EMITNULLIFIER]: DefaultBaseGasCost,
  [Opcode.L1TOL2MSGEXISTS]: DefaultBaseGasCost,
  [Opcode.HEADERMEMBER]: DefaultBaseGasCost,
  [Opcode.EMITUNENCRYPTEDLOG]: DefaultBaseGasCost,
  [Opcode.SENDL2TOL1MSG]: DefaultBaseGasCost,
  [Opcode.GETCONTRACTINSTANCE]: DefaultBaseGasCost,
  // External calls
  [Opcode.CALL]: DefaultBaseGasCost,
  [Opcode.STATICCALL]: DefaultBaseGasCost,
  [Opcode.DELEGATECALL]: DefaultBaseGasCost,
  [Opcode.RETURN]: DefaultBaseGasCost,
  [Opcode.REVERT]: DefaultBaseGasCost,
  // Misc
  [Opcode.DEBUGLOG]: DefaultBaseGasCost,
  // Gadgets
  [Opcode.KECCAK]: DefaultBaseGasCost,
  [Opcode.POSEIDON2]: DefaultBaseGasCost,
  [Opcode.SHA256]: DefaultBaseGasCost,
  [Opcode.PEDERSEN]: DefaultBaseGasCost,
  [Opcode.ECADD]: DefaultBaseGasCost,
  [Opcode.MSM]: DefaultBaseGasCost,
  // Conversions
  [Opcode.TORADIXLE]: DefaultBaseGasCost,
  // Other
  [Opcode.SHA256COMPRESSION]: DefaultBaseGasCost,
  [Opcode.KECCAKF1600]: DefaultBaseGasCost,
};

/** Returns the fixed base gas cost for a given opcode. */
export function getBaseGasCost(opcode: Opcode): Gas {
  return BaseGasCosts[opcode];
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
