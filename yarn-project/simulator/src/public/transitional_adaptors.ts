// All code in this file needs to die once the public executor is phased out in favor of the AVM.
import { UnencryptedFunctionL2Logs, UnencryptedL2Log } from '@aztec/circuit-types';
import {
  CallContext,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  FunctionData,
  type Gas,
  type GasSettings,
  type GlobalVariables,
  type Header,
  L2ToL1Message,
  NoteHash,
  Nullifier,
  ReadRequest,
  SideEffect,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { type AvmContext } from '../avm/avm_context.js';
import { AvmExecutionEnvironment } from '../avm/avm_execution_environment.js';
import { type AvmMachineState } from '../avm/avm_machine_state.js';
import { AvmContractCallResults } from '../avm/avm_message_call_result.js';
import { type JournalData } from '../avm/journal/journal.js';
import { Mov } from '../avm/opcodes/memory.js';
import { createSimulationError } from '../common/errors.js';
import { type PublicExecution, type PublicExecutionResult } from './execution.js';
import { type PublicExecutionContext } from './public_execution_context.js';

/**
 * Convert a PublicExecution(Environment) object to an AvmExecutionEnvironment
 *
 * @param current
 * @param globalVariables
 * @returns
 */
export function createAvmExecutionEnvironment(
  current: PublicExecution,
  header: Header,
  globalVariables: GlobalVariables,
  gasSettings: GasSettings,
  transactionFee: Fr,
): AvmExecutionEnvironment {
  return new AvmExecutionEnvironment(
    current.contractAddress,
    current.callContext.storageContractAddress,
    current.callContext.msgSender,
    globalVariables.gasFees.feePerL2Gas,
    globalVariables.gasFees.feePerDaGas,
    /*contractCallDepth=*/ Fr.zero(),
    header,
    globalVariables,
    current.callContext.isStaticCall,
    current.callContext.isDelegateCall,
    current.args,
    gasSettings,
    transactionFee,
    current.functionData.selector,
  );
}

export function createPublicExecution(
  startSideEffectCounter: number,
  avmEnvironment: AvmExecutionEnvironment,
  calldata: Fr[],
): PublicExecution {
  const callContext = CallContext.from({
    msgSender: avmEnvironment.sender,
    storageContractAddress: avmEnvironment.storageAddress,
    functionSelector: avmEnvironment.temporaryFunctionSelector,
    isDelegateCall: avmEnvironment.isDelegateCall,
    isStaticCall: avmEnvironment.isStaticCall,
    sideEffectCounter: startSideEffectCounter,
  });
  const functionData = new FunctionData(avmEnvironment.temporaryFunctionSelector, /*isPrivate=*/ false);
  const execution: PublicExecution = {
    contractAddress: avmEnvironment.address,
    callContext,
    args: calldata,
    functionData,
  };
  return execution;
}

export function convertAvmResultsToPxResult(
  avmResult: AvmContractCallResults,
  startSideEffectCounter: number,
  fromPx: PublicExecution,
  startGas: Gas,
  endAvmContext: AvmContext,
): PublicExecutionResult {
  const endPersistableState = endAvmContext.persistableState;
  const endMachineState = endAvmContext.machineState;
  return {
    ...endPersistableState.transitionalExecutionResult, // includes nestedExecutions
    execution: fromPx,
    returnValues: avmResult.output,
    startSideEffectCounter: new Fr(startSideEffectCounter),
    endSideEffectCounter: new Fr(endPersistableState.trace.accessCounter),
    unencryptedLogs: new UnencryptedFunctionL2Logs(endPersistableState.transitionalExecutionResult.unencryptedLogs),
    allUnencryptedLogs: new UnencryptedFunctionL2Logs(
      endPersistableState.transitionalExecutionResult.allUnencryptedLogs,
    ),
    reverted: avmResult.reverted,
    revertReason: avmResult.revertReason ? createSimulationError(avmResult.revertReason) : undefined,
    startGasLeft: startGas,
    endGasLeft: endMachineState.gasLeft,
    transactionFee: endAvmContext.environment.transactionFee,
  };
}

/**
 * Convert the result of an AVM contract call to a PublicExecutionResult for the public kernel
 *
 * @param execution
 * @param newWorldState
 * @param result
 * @returns
 */
export async function convertAvmResults(
  executionContext: PublicExecutionContext,
  newWorldState: JournalData,
  result: AvmContractCallResults,
  endMachineState: AvmMachineState,
): Promise<PublicExecutionResult> {
  const execution = executionContext.execution;

  const contractStorageReads: ContractStorageRead[] = newWorldState.storageReads.map(
    read => new ContractStorageRead(read.slot, read.value, read.counter.toNumber(), read.storageAddress),
  );
  const contractStorageUpdateRequests: ContractStorageUpdateRequest[] = newWorldState.storageWrites.map(
    write => new ContractStorageUpdateRequest(write.slot, write.value, write.counter.toNumber(), write.storageAddress),
  );
  // We need to write the storage updates to the DB, because that's what the ACVM expects.
  // Assumes the updates are in the right order.
  for (const write of newWorldState.storageWrites) {
    await executionContext.stateDb.storageWrite(write.storageAddress, write.slot, write.value);
  }

  const newNoteHashes = newWorldState.newNoteHashes.map(
    noteHash => new NoteHash(noteHash.noteHash, noteHash.counter.toNumber()),
  );
  const nullifierReadRequests: ReadRequest[] = newWorldState.nullifierChecks
    .filter(nullifierCheck => nullifierCheck.exists)
    .map(nullifierCheck => new ReadRequest(nullifierCheck.nullifier, nullifierCheck.counter.toNumber()));
  const nullifierNonExistentReadRequests: ReadRequest[] = newWorldState.nullifierChecks
    .filter(nullifierCheck => !nullifierCheck.exists)
    .map(nullifierCheck => new ReadRequest(nullifierCheck.nullifier, nullifierCheck.counter.toNumber()));
  const newNullifiers: Nullifier[] = newWorldState.newNullifiers.map(
    tracedNullifier =>
      new Nullifier(
        /*value=*/ tracedNullifier.nullifier,
        tracedNullifier.counter.toNumber(),
        /*noteHash=*/ Fr.ZERO, // NEEDED?
      ),
  );
  const unencryptedLogs: UnencryptedFunctionL2Logs = new UnencryptedFunctionL2Logs(
    newWorldState.newLogs.map(log => new UnencryptedL2Log(log.contractAddress, log.selector, log.data)),
  );
  const unencryptedLogsHashes = newWorldState.newLogsHashes.map(
    logHash => new SideEffect(logHash.logHash, logHash.counter),
  );
  const newL2ToL1Messages = newWorldState.newL1Messages.map(m => new L2ToL1Message(m.recipient, m.content));

  const returnValues = result.output;

  // TODO: Support nested executions.
  const nestedExecutions: PublicExecutionResult[] = [];
  const allUnencryptedLogs = unencryptedLogs;
  // TODO keep track of side effect counters
  const startSideEffectCounter = Fr.ZERO;
  const endSideEffectCounter = Fr.ZERO;

  return {
    execution,
    nullifierReadRequests,
    nullifierNonExistentReadRequests,
    newNoteHashes,
    newL2ToL1Messages,
    startSideEffectCounter,
    endSideEffectCounter,
    newNullifiers,
    contractStorageReads,
    contractStorageUpdateRequests,
    returnValues,
    nestedExecutions,
    unencryptedLogsHashes,
    unencryptedLogs,
    unencryptedLogPreimagesLength: new Fr(unencryptedLogs.getSerializedLength()),
    allUnencryptedLogs,
    reverted: result.reverted,
    revertReason: result.revertReason ? createSimulationError(result.revertReason) : undefined,
    startGasLeft: executionContext.availableGas,
    endGasLeft: endMachineState.gasLeft,
    transactionFee: executionContext.transactionFee,
  };
}

export function convertPublicExecutionResult(res: PublicExecutionResult): AvmContractCallResults {
  return new AvmContractCallResults(res.reverted, res.returnValues, res.revertReason);
}

const AVM_MAGIC_SUFFIX = Buffer.from([
  Mov.opcode, // opcode
  0x00, // indirect
  ...Buffer.from('000018ca', 'hex'), // srcOffset
  ...Buffer.from('000018ca', 'hex'), // dstOffset
]);

export function markBytecodeAsAvm(bytecode: Buffer): Buffer {
  return Buffer.concat([bytecode, AVM_MAGIC_SUFFIX]);
}

export function isAvmBytecode(bytecode: Buffer): boolean {
  const magicSize = AVM_MAGIC_SUFFIX.length;
  return bytecode.subarray(-magicSize).equals(AVM_MAGIC_SUFFIX);
}
