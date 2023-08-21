import {
  AztecAddress,
  CallContext,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  FunctionData,
  PublicDataRead,
  PublicDataUpdateRequest,
} from '@aztec/circuits.js';
import { computePublicDataTreeIndex, computePublicDataTreeValue } from '@aztec/circuits.js/abis';
import { IWasmModule } from '@aztec/foundation/wasm';
import { FunctionL2Logs } from '@aztec/types';

/**
 * The public function execution result.
 */
export interface PublicExecutionResult {
  /** The execution that triggered this result. */
  execution: PublicExecution;
  /** The return values of the function. */
  returnValues: Fr[];
  /** The new commitments to be inserted into the commitments tree. */
  newCommitments: Fr[];
  /** The new l2 to l1 messages generated in this call. */
  newL2ToL1Messages: Fr[];
  /** The new nullifiers to be inserted into the nullifier tree. */
  newNullifiers: Fr[];
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
 * @param wasm - A module providing low-level wasm access.
 * @param execResult - The topmost execution result.
 * @returns All public data reads (in execution order).
 */
export function collectPublicDataReads(wasm: IWasmModule, execResult: PublicExecutionResult): PublicDataRead[] {
  // HACK(#1622): part of temporary hack - may be able to remove this function after public state ordering is fixed
  const contractAddress = execResult.execution.contractAddress;

  const thisExecPublicDataReads = execResult.contractStorageReads.map(read =>
    contractStorageReadToPublicDataRead(wasm, read, contractAddress),
  );
  const unsorted = [
    ...thisExecPublicDataReads,
    ...[...execResult.nestedExecutions].flatMap(result => collectPublicDataReads(wasm, result)),
  ];
  return unsorted.sort((a, b) => a.sideEffectCounter! - b.sideEffectCounter!);
}

/**
 * Collect all public storage update requests across all nested executions
 * and convert them to PublicDataUpdateRequests (to match kernel output).
 * @param wasm - A module providing low-level wasm access.
 * @param execResult - The topmost execution result.
 * @returns All public data reads (in execution order).
 */
export function collectPublicDataUpdateRequests(
  wasm: IWasmModule,
  execResult: PublicExecutionResult,
): PublicDataUpdateRequest[] {
  // HACK(#1622): part of temporary hack - may be able to remove this function after public state ordering is fixed
  const contractAddress = execResult.execution.contractAddress;

  const thisExecPublicDataUpdateRequests = execResult.contractStorageUpdateRequests.map(update =>
    contractStorageUpdateRequestToPublicDataUpdateRequest(wasm, update, contractAddress),
  );
  const unsorted = [
    ...thisExecPublicDataUpdateRequests,
    ...[...execResult.nestedExecutions].flatMap(result => collectPublicDataUpdateRequests(wasm, result)),
  ];
  return unsorted.sort((a, b) => a.sideEffectCounter! - b.sideEffectCounter!);
}

/**
 * Convert a Contract Storage Read to a Public Data Read.
 * @param wasm - A module providing low-level wasm access.
 * @param read - the contract storage read to convert
 * @param contractAddress - the contract address of the read
 * @returns The public data read.
 */
function contractStorageReadToPublicDataRead(
  wasm: IWasmModule,
  read: ContractStorageRead,
  contractAddress: AztecAddress,
): PublicDataRead {
  return new PublicDataRead(
    computePublicDataTreeIndex(wasm, contractAddress.toField(), read.storageSlot),
    computePublicDataTreeValue(wasm, read.currentValue),
    read.sideEffectCounter!,
  );
}

/**
 * Convert a Contract Storage Update Request to a Public Data Update Request.
 * @param wasm - A module providing low-level wasm access.
 * @param update - the contract storage update request to convert
 * @param contractAddress - the contract address of the data update request.
 * @returns The public data update request.
 */
function contractStorageUpdateRequestToPublicDataUpdateRequest(
  wasm: IWasmModule,
  update: ContractStorageUpdateRequest,
  contractAddress: AztecAddress,
): PublicDataUpdateRequest {
  return new PublicDataUpdateRequest(
    computePublicDataTreeIndex(wasm, contractAddress.toField(), update.storageSlot),
    computePublicDataTreeValue(wasm, update.oldValue),
    computePublicDataTreeValue(wasm, update.newValue),
    update.sideEffectCounter!,
  );
}
