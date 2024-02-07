import { GlobalVariables } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { mock } from 'jest-mock-extended';

import { CommitmentsDB, PublicContractsDB, PublicStateDB } from '../../index.js';
import { AvmContext } from '../avm_context.js';
import { AvmExecutionEnvironment } from '../avm_execution_environment.js';
import { AvmMachineState } from '../avm_machine_state.js';
import { HostStorage } from '../journal/host_storage.js';
import { AvmWorldStateJournal } from '../journal/journal.js';

/**
 * Create a new AVM context with default values.
 */
export function initContext(overrides?: {
  worldState?: AvmWorldStateJournal;
  env?: AvmExecutionEnvironment;
  machineState?: AvmMachineState;
}): AvmContext {
  return new AvmContext(
    overrides?.worldState || initMockWorldStateJournal(),
    overrides?.env || initExecutionEnvironment(),
    overrides?.machineState || initMachineState(),
  );
}

/** Creates an empty world state with mocked storage. */
export function initMockWorldStateJournal(): AvmWorldStateJournal {
  const hostStorage = new HostStorage(mock<PublicStateDB>(), mock<PublicContractsDB>(), mock<CommitmentsDB>());
  return new AvmWorldStateJournal(hostStorage);
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
 * Create an empty instance of the Machine State where all values are zero, unless overridden in the overrides object
 */
export function initMachineState(overrides?: Partial<AvmMachineState>): AvmMachineState {
  return AvmMachineState.fromState({
    l1GasLeft: overrides?.l1GasLeft ?? 0,
    l2GasLeft: overrides?.l2GasLeft ?? 0,
    daGasLeft: overrides?.daGasLeft ?? 0,
  });
}
