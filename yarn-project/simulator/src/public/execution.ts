import { type SimulationError, type UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  type AztecAddress,
  type ContractStorageRead,
  type ContractStorageUpdateRequest,
  type Fr,
  type L2ToL1Message,
  type PublicCallRequest,
  PublicDataRead,
  PublicDataUpdateRequest,
  type ReadRequest,
  type SideEffect,
  type SideEffectLinkedToNoteHash,
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
  /** The side effect counter at the start of the function call. */
  startSideEffectCounter: Fr;
  /** The side effect counter after executing this function call */
  endSideEffectCounter: Fr;
  /** The new nullifiers to be inserted into the nullifier tree. */
  newNullifiers: SideEffectLinkedToNoteHash[];
  /** The nullifier read requests emitted in this call. */
  nullifierReadRequests: ReadRequest[];
  /** The nullifier non existent read requests emitted in this call. */
  nullifierNonExistentReadRequests: ReadRequest[];
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
  unencryptedLogs: UnencryptedFunctionL2Logs;
  /**
   * Whether the execution reverted.
   */
  reverted: boolean;
  /**
   * The revert reason if the execution reverted.
   */
  revertReason: SimulationError | undefined;
}

/**
 * The execution of a public function.
 */
export type PublicExecution = Pick<PublicCallRequest, 'contractAddress' | 'functionData' | 'callContext' | 'args'>;

/**
 * Returns if the input is a public execution result and not just a public execution.
 * @param input - Public execution or public execution result.
 * @returns Whether the input is a public execution result and not just a public execution.
 */
export function isPublicExecutionResult(
  input: PublicExecution | PublicExecutionResult,
): input is PublicExecutionResult {
  return 'execution' in input && input.execution !== undefined;
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
  unencryptedLogs: UnencryptedFunctionL2Logs,
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
