import { FunctionL2Logs } from '@aztec/circuit-types';
import {
  AztecAddress,
  CallContext,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  FunctionData,
  L2ToL1Message,
  PublicDataRead,
  PublicDataUpdateRequest,
  SideEffect,
  SideEffectLinkedToNoteHash,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot, computePublicDataTreeValue } from '@aztec/circuits.js/hash';

/**
 * The public function execution result.
 */
export interface PublicExecutionResult {
  /** The execution that triggered this result. */
  execution: PublicExecution;
  /** The return values of the function. */
  returnValues: Fr[];
  /** The new note hashes to be inserted into the note hashes tree. */
  newNoteHashes: SideEffect[];
  /** The new l2 to l1 messages generated in this call. */
  newL2ToL1Messages: L2ToL1Message[];
  /** The new nullifiers to be inserted into the nullifier tree. */
  newNullifiers: SideEffectLinkedToNoteHash[];
  /** The contract storage reads performed by the function. */
  contractStorageReads: ContractStorageRead[];
  /** The contract storage update requests performed by the function. */
  contractStorageUpdateRequests: ContractStorageUpdateRequest[];
  /** The results of nested calls. */
  nestedExecutions: this[];
  /**
   * Unencrypted logs emitted during execution of this function call.
   * Note: These are preimages to `unencryptedLogsHash`.
   */
  unencryptedLogs: FunctionL2Logs;
}

/**
 * The execution of a public function.
 */
export interface PublicExecution {
  /** Address of the contract being executed. */
  contractAddress: AztecAddress;
  /** Function of the contract being called. */
  functionData: FunctionData;
  /** Arguments for the call. */
  args: Fr[];
  /** Context of the call. */
  callContext: CallContext;
}

/**
 * Returns if the input is a public execution result and not just a public execution.
 * @param input - Public execution or public execution result.
 * @returns Whether the input is a public execution result and not just a public execution.
 */
export function isPublicExecutionResult(
  input: PublicExecution | PublicExecutionResult,
): input is PublicExecutionResult {
  return !!(input as PublicExecutionResult).execution;
}

/**
 * Collect all public storage reads across all nested executions
 * and convert them to PublicDataReads (to match kernel output).
 * @param execResult - The topmost execution result.
 * @returns All public data reads (in execution order).
 */
export function collectPublicDataReads(execResult: PublicExecutionResult): PublicDataRead[] {
  // HACK(#1622): part of temporary hack - may be able to remove this function after public state ordering is fixed
  const contractAddress = execResult.execution.callContext.storageContractAddress;

  const thisExecPublicDataReads = execResult.contractStorageReads.map(read =>
    contractStorageReadToPublicDataRead(read, contractAddress),
  );
  const unsorted = [
    ...thisExecPublicDataReads,
    ...[...execResult.nestedExecutions].flatMap(result => collectPublicDataReads(result)),
  ];
  return unsorted.sort((a, b) => a.sideEffectCounter! - b.sideEffectCounter!);
}

/**
 * Collect all public storage update requests across all nested executions
 * and convert them to PublicDataUpdateRequests (to match kernel output).
 * @param execResult - The topmost execution result.
 * @returns All public data reads (in execution order).
 */
export function collectPublicDataUpdateRequests(execResult: PublicExecutionResult): PublicDataUpdateRequest[] {
  // HACK(#1622): part of temporary hack - may be able to remove this function after public state ordering is fixed
  const contractAddress = execResult.execution.callContext.storageContractAddress;

  const thisExecPublicDataUpdateRequests = execResult.contractStorageUpdateRequests.map(update =>
    contractStorageUpdateRequestToPublicDataUpdateRequest(update, contractAddress),
  );
  const unsorted = [
    ...thisExecPublicDataUpdateRequests,
    ...[...execResult.nestedExecutions].flatMap(result => collectPublicDataUpdateRequests(result)),
  ];
  return unsorted.sort((a, b) => a.sideEffectCounter! - b.sideEffectCounter!);
}

/**
 * Convert a Contract Storage Read to a Public Data Read.
 * @param read - the contract storage read to convert
 * @param contractAddress - the contract address of the read
 * @returns The public data read.
 */
function contractStorageReadToPublicDataRead(read: ContractStorageRead, contractAddress: AztecAddress): PublicDataRead {
  return new PublicDataRead(
    computePublicDataTreeLeafSlot(contractAddress, read.storageSlot),
    computePublicDataTreeValue(read.currentValue),
    read.sideEffectCounter!,
  );
}

/**
 * Convert a Contract Storage Update Request to a Public Data Update Request.
 * @param update - the contract storage update request to convert
 * @param contractAddress - the contract address of the data update request.
 * @returns The public data update request.
 */
function contractStorageUpdateRequestToPublicDataUpdateRequest(
  update: ContractStorageUpdateRequest,
  contractAddress: AztecAddress,
): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(
    computePublicDataTreeLeafSlot(contractAddress, update.storageSlot),
    computePublicDataTreeValue(update.newValue),
    update.sideEffectCounter!,
  );
}

/**
 * Checks whether the child execution result is valid for a static call (no state modifications).
 * @param executionResult - The execution result of a public function
 */

export function checkValidStaticCall(
  newNoteHashes: SideEffect[],
  newNullifiers: SideEffectLinkedToNoteHash[],
  contractStorageUpdateRequests: ContractStorageUpdateRequest[],
  newL2ToL1Messages: L2ToL1Message[],
  unencryptedLogs: FunctionL2Logs,
) {
  if (
    contractStorageUpdateRequests.length > 0 ||
    newNoteHashes.length > 0 ||
    newNullifiers.length > 0 ||
    newL2ToL1Messages.length > 0 ||
    unencryptedLogs.logs.length > 0
  ) {
    throw new Error('Static call cannot update the state, emit L2->L1 messages or generate logs');
  }
}
