import * as c from '@aztec/circuits.js/constants';

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

function makeCost(l2Gas: number, daGas: number): Gas {
  return { l2Gas, daGas };
}

/** Dimensions of gas usage: L1, L2, and DA. */
export const GasDimensions = ['l2Gas', 'daGas'] as const;

/** Base gas costs for each instruction. Additional gas cost may be added on top due to memory or storage accesses, etc. */
const BaseGasCosts: Record<Opcode, Gas> = {
  [Opcode.ADD]: makeCost(c.AVM_ADD_BASE_L2_GAS, 0),
  [Opcode.SUB]: makeCost(c.AVM_SUB_BASE_L2_GAS, 0),
  [Opcode.MUL]: makeCost(c.AVM_MUL_BASE_L2_GAS, 0),
  [Opcode.DIV]: makeCost(c.AVM_DIV_BASE_L2_GAS, 0),
  [Opcode.FDIV]: makeCost(c.AVM_FDIV_BASE_L2_GAS, 0),
  [Opcode.EQ]: makeCost(c.AVM_EQ_BASE_L2_GAS, 0),
  [Opcode.LT]: makeCost(c.AVM_LT_BASE_L2_GAS, 0),
  [Opcode.LTE]: makeCost(c.AVM_LTE_BASE_L2_GAS, 0),
  [Opcode.AND]: makeCost(c.AVM_AND_BASE_L2_GAS, 0),
  [Opcode.OR]: makeCost(c.AVM_OR_BASE_L2_GAS, 0),
  [Opcode.XOR]: makeCost(c.AVM_XOR_BASE_L2_GAS, 0),
  [Opcode.NOT]: makeCost(c.AVM_NOT_BASE_L2_GAS, 0),
  [Opcode.SHL]: makeCost(c.AVM_SHL_BASE_L2_GAS, 0),
  [Opcode.SHR]: makeCost(c.AVM_SHR_BASE_L2_GAS, 0),
  [Opcode.CAST]: makeCost(c.AVM_CAST_BASE_L2_GAS, 0),
  [Opcode.ADDRESS]: makeCost(c.AVM_ADDRESS_BASE_L2_GAS, 0),
  [Opcode.STORAGEADDRESS]: makeCost(c.AVM_STORAGEADDRESS_BASE_L2_GAS, 0),
  [Opcode.SENDER]: makeCost(c.AVM_SENDER_BASE_L2_GAS, 0),
  [Opcode.FEEPERL2GAS]: makeCost(c.AVM_FEEPERL2GAS_BASE_L2_GAS, 0),
  [Opcode.FEEPERDAGAS]: makeCost(c.AVM_FEEPERDAGAS_BASE_L2_GAS, 0),
  [Opcode.TRANSACTIONFEE]: makeCost(c.AVM_TRANSACTIONFEE_BASE_L2_GAS, 0),
  [Opcode.FUNCTIONSELECTOR]: makeCost(c.AVM_FUNCTIONSELECTOR_BASE_L2_GAS, 0),
  [Opcode.CHAINID]: makeCost(c.AVM_CHAINID_BASE_L2_GAS, 0),
  [Opcode.VERSION]: makeCost(c.AVM_VERSION_BASE_L2_GAS, 0),
  [Opcode.BLOCKNUMBER]: makeCost(c.AVM_BLOCKNUMBER_BASE_L2_GAS, 0),
  [Opcode.TIMESTAMP]: makeCost(c.AVM_TIMESTAMP_BASE_L2_GAS, 0),
  [Opcode.COINBASE]: makeCost(c.AVM_COINBASE_BASE_L2_GAS, 0),
  [Opcode.BLOCKL2GASLIMIT]: makeCost(c.AVM_BLOCKL2GASLIMIT_BASE_L2_GAS, 0),
  [Opcode.BLOCKDAGASLIMIT]: makeCost(c.AVM_BLOCKDAGASLIMIT_BASE_L2_GAS, 0),
  [Opcode.CALLDATACOPY]: makeCost(c.AVM_CALLDATACOPY_BASE_L2_GAS, 0),
  [Opcode.L2GASLEFT]: makeCost(c.AVM_L2GASLEFT_BASE_L2_GAS, 0),
  [Opcode.DAGASLEFT]: makeCost(c.AVM_DAGASLEFT_BASE_L2_GAS, 0),
  [Opcode.JUMP]: makeCost(c.AVM_JUMP_BASE_L2_GAS, 0),
  [Opcode.JUMPI]: makeCost(c.AVM_JUMPI_BASE_L2_GAS, 0),
  [Opcode.INTERNALCALL]: makeCost(c.AVM_INTERNALCALL_BASE_L2_GAS, 0),
  [Opcode.INTERNALRETURN]: makeCost(c.AVM_INTERNALRETURN_BASE_L2_GAS, 0),
  [Opcode.SET]: makeCost(c.AVM_SET_BASE_L2_GAS, 0),
  [Opcode.MOV]: makeCost(c.AVM_MOV_BASE_L2_GAS, 0),
  [Opcode.CMOV]: makeCost(c.AVM_CMOV_BASE_L2_GAS, 0),
  [Opcode.SLOAD]: makeCost(c.AVM_SLOAD_BASE_L2_GAS, 0),
  [Opcode.SSTORE]: makeCost(c.AVM_SSTORE_BASE_L2_GAS, 0),
  [Opcode.NOTEHASHEXISTS]: makeCost(c.AVM_NOTEHASHEXISTS_BASE_L2_GAS, 0),
  [Opcode.EMITNOTEHASH]: makeCost(c.AVM_EMITNOTEHASH_BASE_L2_GAS, 0),
  [Opcode.NULLIFIEREXISTS]: makeCost(c.AVM_NULLIFIEREXISTS_BASE_L2_GAS, 0),
  [Opcode.EMITNULLIFIER]: makeCost(c.AVM_EMITNULLIFIER_BASE_L2_GAS, 0),
  [Opcode.L1TOL2MSGEXISTS]: makeCost(c.AVM_L1TOL2MSGEXISTS_BASE_L2_GAS, 0),
  [Opcode.HEADERMEMBER]: makeCost(c.AVM_HEADERMEMBER_BASE_L2_GAS, 0),
  [Opcode.EMITUNENCRYPTEDLOG]: makeCost(c.AVM_EMITUNENCRYPTEDLOG_BASE_L2_GAS, 0),
  [Opcode.SENDL2TOL1MSG]: makeCost(c.AVM_SENDL2TOL1MSG_BASE_L2_GAS, 0),
  [Opcode.GETCONTRACTINSTANCE]: makeCost(c.AVM_GETCONTRACTINSTANCE_BASE_L2_GAS, 0),
  [Opcode.CALL]: makeCost(c.AVM_CALL_BASE_L2_GAS, 0),
  [Opcode.STATICCALL]: makeCost(c.AVM_STATICCALL_BASE_L2_GAS, 0),
  [Opcode.DELEGATECALL]: makeCost(c.AVM_DELEGATECALL_BASE_L2_GAS, 0),
  [Opcode.RETURN]: makeCost(c.AVM_RETURN_BASE_L2_GAS, 0),
  [Opcode.REVERT]: makeCost(c.AVM_REVERT_BASE_L2_GAS, 0),
  [Opcode.DEBUGLOG]: makeCost(c.AVM_DEBUGLOG_BASE_L2_GAS, 0),
  [Opcode.KECCAK]: makeCost(c.AVM_KECCAK_BASE_L2_GAS, 0),
  [Opcode.POSEIDON2]: makeCost(c.AVM_POSEIDON2_BASE_L2_GAS, 0),
  [Opcode.SHA256]: makeCost(c.AVM_SHA256_BASE_L2_GAS, 0),
  [Opcode.PEDERSEN]: makeCost(c.AVM_PEDERSEN_BASE_L2_GAS, 0),
  [Opcode.ECADD]: makeCost(c.AVM_ECADD_BASE_L2_GAS, 0),
  [Opcode.MSM]: makeCost(c.AVM_MSM_BASE_L2_GAS, 0),
  [Opcode.PEDERSENCOMMITMENT]: makeCost(c.AVM_PEDERSENCOMMITMENT_BASE_L2_GAS, 0),
  [Opcode.TORADIXLE]: makeCost(c.AVM_TORADIXLE_BASE_L2_GAS, 0),
  [Opcode.SHA256COMPRESSION]: makeCost(c.AVM_SHA256COMPRESSION_BASE_L2_GAS, 0),
  [Opcode.KECCAKF1600]: makeCost(c.AVM_KECCAKF1600_BASE_L2_GAS, 0),
};

const DynamicGasCosts: Record<Opcode, Gas> = {
  [Opcode.ADD]: makeCost(c.AVM_ADD_DYN_L2_GAS, 0),
  [Opcode.SUB]: makeCost(c.AVM_SUB_DYN_L2_GAS, 0),
  [Opcode.MUL]: makeCost(c.AVM_MUL_DYN_L2_GAS, 0),
  [Opcode.DIV]: makeCost(c.AVM_DIV_DYN_L2_GAS, 0),
  [Opcode.FDIV]: makeCost(c.AVM_FDIV_DYN_L2_GAS, 0),
  [Opcode.EQ]: makeCost(c.AVM_EQ_DYN_L2_GAS, 0),
  [Opcode.LT]: makeCost(c.AVM_LT_DYN_L2_GAS, 0),
  [Opcode.LTE]: makeCost(c.AVM_LTE_DYN_L2_GAS, 0),
  [Opcode.AND]: makeCost(c.AVM_AND_DYN_L2_GAS, 0),
  [Opcode.OR]: makeCost(c.AVM_OR_DYN_L2_GAS, 0),
  [Opcode.XOR]: makeCost(c.AVM_XOR_DYN_L2_GAS, 0),
  [Opcode.NOT]: makeCost(c.AVM_NOT_DYN_L2_GAS, 0),
  [Opcode.SHL]: makeCost(c.AVM_SHL_DYN_L2_GAS, 0),
  [Opcode.SHR]: makeCost(c.AVM_SHR_DYN_L2_GAS, 0),
  [Opcode.CAST]: makeCost(c.AVM_CAST_DYN_L2_GAS, 0),
  [Opcode.ADDRESS]: makeCost(c.AVM_ADDRESS_DYN_L2_GAS, 0),
  [Opcode.STORAGEADDRESS]: makeCost(c.AVM_STORAGEADDRESS_DYN_L2_GAS, 0),
  [Opcode.SENDER]: makeCost(c.AVM_SENDER_DYN_L2_GAS, 0),
  [Opcode.FEEPERL2GAS]: makeCost(c.AVM_FEEPERL2GAS_DYN_L2_GAS, 0),
  [Opcode.FEEPERDAGAS]: makeCost(c.AVM_FEEPERDAGAS_DYN_L2_GAS, 0),
  [Opcode.TRANSACTIONFEE]: makeCost(c.AVM_TRANSACTIONFEE_DYN_L2_GAS, 0),
  [Opcode.FUNCTIONSELECTOR]: makeCost(c.AVM_FUNCTIONSELECTOR_DYN_L2_GAS, 0),
  [Opcode.CHAINID]: makeCost(c.AVM_CHAINID_DYN_L2_GAS, 0),
  [Opcode.VERSION]: makeCost(c.AVM_VERSION_DYN_L2_GAS, 0),
  [Opcode.BLOCKNUMBER]: makeCost(c.AVM_BLOCKNUMBER_DYN_L2_GAS, 0),
  [Opcode.TIMESTAMP]: makeCost(c.AVM_TIMESTAMP_DYN_L2_GAS, 0),
  [Opcode.COINBASE]: makeCost(c.AVM_COINBASE_DYN_L2_GAS, 0),
  [Opcode.BLOCKL2GASLIMIT]: makeCost(c.AVM_BLOCKL2GASLIMIT_DYN_L2_GAS, 0),
  [Opcode.BLOCKDAGASLIMIT]: makeCost(c.AVM_BLOCKDAGASLIMIT_DYN_L2_GAS, 0),
  [Opcode.CALLDATACOPY]: makeCost(c.AVM_CALLDATACOPY_DYN_L2_GAS, 0),
  [Opcode.L2GASLEFT]: makeCost(c.AVM_L2GASLEFT_DYN_L2_GAS, 0),
  [Opcode.DAGASLEFT]: makeCost(c.AVM_DAGASLEFT_DYN_L2_GAS, 0),
  [Opcode.JUMP]: makeCost(c.AVM_JUMP_DYN_L2_GAS, 0),
  [Opcode.JUMPI]: makeCost(c.AVM_JUMPI_DYN_L2_GAS, 0),
  [Opcode.INTERNALCALL]: makeCost(c.AVM_INTERNALCALL_DYN_L2_GAS, 0),
  [Opcode.INTERNALRETURN]: makeCost(c.AVM_INTERNALRETURN_DYN_L2_GAS, 0),
  [Opcode.SET]: makeCost(c.AVM_SET_DYN_L2_GAS, 0),
  [Opcode.MOV]: makeCost(c.AVM_MOV_DYN_L2_GAS, 0),
  [Opcode.CMOV]: makeCost(c.AVM_CMOV_DYN_L2_GAS, 0),
  [Opcode.SLOAD]: makeCost(c.AVM_SLOAD_DYN_L2_GAS, 0),
  [Opcode.SSTORE]: makeCost(c.AVM_SSTORE_DYN_L2_GAS, 0),
  [Opcode.NOTEHASHEXISTS]: makeCost(c.AVM_NOTEHASHEXISTS_DYN_L2_GAS, 0),
  [Opcode.EMITNOTEHASH]: makeCost(c.AVM_EMITNOTEHASH_DYN_L2_GAS, 0),
  [Opcode.NULLIFIEREXISTS]: makeCost(c.AVM_NULLIFIEREXISTS_DYN_L2_GAS, 0),
  [Opcode.EMITNULLIFIER]: makeCost(c.AVM_EMITNULLIFIER_DYN_L2_GAS, 0),
  [Opcode.L1TOL2MSGEXISTS]: makeCost(c.AVM_L1TOL2MSGEXISTS_DYN_L2_GAS, 0),
  [Opcode.HEADERMEMBER]: makeCost(c.AVM_HEADERMEMBER_DYN_L2_GAS, 0),
  [Opcode.EMITUNENCRYPTEDLOG]: makeCost(c.AVM_EMITUNENCRYPTEDLOG_DYN_L2_GAS, 0),
  [Opcode.SENDL2TOL1MSG]: makeCost(c.AVM_SENDL2TOL1MSG_DYN_L2_GAS, 0),
  [Opcode.GETCONTRACTINSTANCE]: makeCost(c.AVM_GETCONTRACTINSTANCE_DYN_L2_GAS, 0),
  [Opcode.CALL]: makeCost(c.AVM_CALL_DYN_L2_GAS, 0),
  [Opcode.STATICCALL]: makeCost(c.AVM_STATICCALL_DYN_L2_GAS, 0),
  [Opcode.DELEGATECALL]: makeCost(c.AVM_DELEGATECALL_DYN_L2_GAS, 0),
  [Opcode.RETURN]: makeCost(c.AVM_RETURN_DYN_L2_GAS, 0),
  [Opcode.REVERT]: makeCost(c.AVM_REVERT_DYN_L2_GAS, 0),
  [Opcode.DEBUGLOG]: makeCost(c.AVM_DEBUGLOG_DYN_L2_GAS, 0),
  [Opcode.KECCAK]: makeCost(c.AVM_KECCAK_DYN_L2_GAS, 0),
  [Opcode.POSEIDON2]: makeCost(c.AVM_POSEIDON2_DYN_L2_GAS, 0),
  [Opcode.SHA256]: makeCost(c.AVM_SHA256_DYN_L2_GAS, 0),
  [Opcode.PEDERSEN]: makeCost(c.AVM_PEDERSEN_DYN_L2_GAS, 0),
  [Opcode.ECADD]: makeCost(c.AVM_ECADD_DYN_L2_GAS, 0),
  [Opcode.MSM]: makeCost(c.AVM_MSM_DYN_L2_GAS, 0),
  [Opcode.PEDERSENCOMMITMENT]: makeCost(c.AVM_PEDERSENCOMMITMENT_DYN_L2_GAS, 0),
  [Opcode.TORADIXLE]: makeCost(c.AVM_TORADIXLE_DYN_L2_GAS, 0),
  [Opcode.SHA256COMPRESSION]: makeCost(c.AVM_SHA256COMPRESSION_DYN_L2_GAS, 0),
  [Opcode.KECCAKF1600]: makeCost(c.AVM_KECCAKF1600_DYN_L2_GAS, 0),
};

/** Returns the fixed base gas cost for a given opcode. */
export function getBaseGasCost(opcode: Opcode): Gas {
  return BaseGasCosts[opcode];
}

export function getDynamicGasCost(opcode: Opcode): Gas {
  return DynamicGasCosts[opcode];
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
