import { GasFees, GlobalVariables, Header } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AvmTestContractArtifact } from '@aztec/noir-contracts.js';

import { strict as assert } from 'assert';
import { mock } from 'jest-mock-extended';
import merge from 'lodash.merge';

import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from '../../index.js';
import { type PublicSideEffectTraceInterface } from '../../public/side_effect_trace_interface.js';
import { AvmContext } from '../avm_context.js';
import { AvmContextInputs, AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { AvmMachineState } from '../avm_machine_state.js';
import { Field, Uint8, Uint64 } from '../avm_memory_types.js';
import { HostStorage } from '../journal/host_storage.js';
import { AvmPersistableStateManager } from '../journal/journal.js';
import { NullifierManager } from '../journal/nullifiers.js';
import { PublicStorage } from '../journal/public_storage.js';

/**
 * Create a new AVM context with default values.
 */
export function initContext(overrides?: {
  persistableState?: AvmPersistableStateManager;
  env?: AvmExecutionEnvironment;
  machineState?: AvmMachineState;
}): AvmContext {
  return new AvmContext(
    overrides?.persistableState || initPersistableStateManager(),
    overrides?.env || initExecutionEnvironment(),
    overrides?.machineState || initMachineState(),
  );
}

/** Creates an empty host storage with mocked dbs. */
export function initHostStorage(overrides?: {
  publicDb?: PublicStateDB;
  contractsDb?: PublicContractsDB;
  commitmentsDb?: CommitmentsDB;
}): HostStorage {
  return new HostStorage(
    overrides?.publicDb || mock<PublicStateDB>(),
    overrides?.contractsDb || mock<PublicContractsDB>(),
    overrides?.commitmentsDb || mock<CommitmentsDB>(),
  );
}

/** Creates an empty state manager with mocked host storage. */
export function initPersistableStateManager(overrides?: {
  hostStorage?: HostStorage;
  trace?: PublicSideEffectTraceInterface;
  publicStorage?: PublicStorage;
  nullifiers?: NullifierManager;
}): AvmPersistableStateManager {
  const hostStorage = overrides?.hostStorage || initHostStorage();
  return new AvmPersistableStateManager(
    hostStorage,
    overrides?.trace || mock<PublicSideEffectTraceInterface>(),
    overrides?.publicStorage || new PublicStorage(hostStorage.publicStateDb),
    overrides?.nullifiers || new NullifierManager(hostStorage.commitmentsDb),
  );
}

/**
 * Create an empty instance of the Execution Environment where all values are zero, unless overridden in the overrides object
 */
export function initExecutionEnvironment(overrides?: Partial<AvmExecutionEnvironment>): AvmExecutionEnvironment {
  return new AvmExecutionEnvironment(
    overrides?.address ?? AztecAddress.zero(),
    overrides?.storageAddress ?? AztecAddress.zero(),
    overrides?.sender ?? AztecAddress.zero(),
    overrides?.functionSelector ?? FunctionSelector.empty(),
    overrides?.contractCallDepth ?? Fr.zero(),
    overrides?.transactionFee ?? Fr.zero(),
    overrides?.header ?? Header.empty(),
    overrides?.globals ?? GlobalVariables.empty(),
    overrides?.isStaticCall ?? false,
    overrides?.isDelegateCall ?? false,
    overrides?.calldata ?? [],
  );
}

/**
 * Create an empty instance of the Execution Environment where all values are zero, unless overridden in the overrides object
 */
export function initGlobalVariables(overrides?: Partial<GlobalVariables>): GlobalVariables {
  return new GlobalVariables(
    overrides?.chainId ?? Fr.zero(),
    overrides?.version ?? Fr.zero(),
    overrides?.blockNumber ?? Fr.zero(),
    overrides?.timestamp ?? Fr.zero(),
    overrides?.coinbase ?? EthAddress.ZERO,
    overrides?.feeRecipient ?? AztecAddress.zero(),
    overrides?.gasFees ?? GasFees.empty(),
  );
}

/**
 * Create an empty instance of the Machine State where all values are set to a large enough amount, unless overridden in the overrides object
 */
export function initMachineState(overrides?: Partial<AvmMachineState>): AvmMachineState {
  return AvmMachineState.fromState({
    l2GasLeft: overrides?.l2GasLeft ?? 1e8,
    daGasLeft: overrides?.daGasLeft ?? 1e8,
  });
}

/**
 * Create a new object with all the same properties as the original, except for the ones in the overrides object.
 */
export function allSameExcept(original: any, overrides: any): any {
  return merge({}, original, overrides);
}

/**
 * Adjust the user index to account for the AvmContextInputs size.
 * This is a hack for testing, and should go away once AvmContextInputs themselves go away.
 */
export function adjustCalldataIndex(userIndex: number): number {
  return userIndex + AvmContextInputs.SIZE;
}

export function anyAvmContextInputs() {
  const tv = [];
  for (let i = 0; i < AvmContextInputs.SIZE; i++) {
    tv.push(expect.any(Fr));
  }
  return tv;
}

export function randomMemoryBytes(length: number): Uint8[] {
  return [...Array(length)].map(_ => new Uint8(Math.floor(Math.random() * 255)));
}

export function randomMemoryUint64s(length: number): Uint64[] {
  return [...Array(length)].map(_ => new Uint64(Math.floor(Math.random() * 255)));
}

export function randomMemoryFields(length: number): Field[] {
  return [...Array(length)].map(_ => new Field(Fr.random()));
}

export function getAvmTestContractBytecode(functionName: string): Buffer {
  const artifact = AvmTestContractArtifact.functions.find(f => f.name === functionName)!;
  assert(
    !!artifact?.bytecode,
    `No bytecode found for function ${functionName}. Try re-running bootstrap.sh on the repository root.`,
  );
  return artifact.bytecode;
}
