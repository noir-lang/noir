import { SiblingPath } from '@aztec/circuit-types';
import { GlobalVariables, L1_TO_L2_MSG_TREE_HEIGHT } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';
import merge from 'lodash.merge';

import {
  type CommitmentsDB,
  MessageLoadOracleInputs,
  type PublicContractsDB,
  type PublicStateDB,
} from '../../index.js';
import { AvmContext } from '../avm_context.js';
import { AvmContextInputs, AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { AvmMachineState } from '../avm_machine_state.js';
import { HostStorage } from '../journal/host_storage.js';
import { AvmPersistableStateManager } from '../journal/journal.js';

/**
 * Create a new AVM context with default values.
 */
export function initContext(overrides?: {
  persistableState?: AvmPersistableStateManager;
  env?: AvmExecutionEnvironment;
  machineState?: AvmMachineState;
}): AvmContext {
  return new AvmContext(
    overrides?.persistableState || initMockPersistableStateManager(),
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

/** Creates an empty state manager with mocked storage. */
export function initMockPersistableStateManager(): AvmPersistableStateManager {
  return new AvmPersistableStateManager(initHostStorage());
}

/**
 * Create an empty instance of the Execution Environment where all values are zero, unless overridden in the overrides object
 */
export function initExecutionEnvironment(overrides?: Partial<AvmExecutionEnvironment>): AvmExecutionEnvironment {
  return new AvmExecutionEnvironment(
    overrides?.address ?? AztecAddress.zero(),
    overrides?.storageAddress ?? AztecAddress.zero(),
    overrides?.origin ?? AztecAddress.zero(),
    overrides?.sender ?? AztecAddress.zero(),
    overrides?.portal ?? EthAddress.ZERO,
    overrides?.feePerL1Gas ?? Fr.zero(),
    overrides?.feePerL2Gas ?? Fr.zero(),
    overrides?.feePerDaGas ?? Fr.zero(),
    overrides?.contractCallDepth ?? Fr.zero(),
    overrides?.globals ?? GlobalVariables.empty(),
    overrides?.isStaticCall ?? false,
    overrides?.isDelegateCall ?? false,
    overrides?.calldata ?? [],
    overrides?.temporaryFunctionSelector ?? FunctionSelector.empty(),
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
  );
}

/**
 * Create an empty instance of the Machine State where all values are set to a large enough amount, unless overridden in the overrides object
 */
export function initMachineState(overrides?: Partial<AvmMachineState>): AvmMachineState {
  return AvmMachineState.fromState({
    l1GasLeft: overrides?.l1GasLeft ?? 100e6,
    l2GasLeft: overrides?.l2GasLeft ?? 100e6,
    daGasLeft: overrides?.daGasLeft ?? 100e6,
  });
}

/**
 * Create a new object with all the same properties as the original, except for the ones in the overrides object.
 */
export function allSameExcept(original: any, overrides: any): any {
  return merge({}, original, overrides);
}

/**
 * Create an empty L1ToL2Message oracle input
 */
export function initL1ToL2MessageOracleInput(
  leafIndex?: bigint,
): MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT> {
  return new MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>(
    leafIndex ?? 0n,
    new SiblingPath(L1_TO_L2_MSG_TREE_HEIGHT, Array(L1_TO_L2_MSG_TREE_HEIGHT)),
  );
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
