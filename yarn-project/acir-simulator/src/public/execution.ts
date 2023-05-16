import {
  AztecAddress,
  CallContext,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  FunctionData,
} from '@aztec/circuits.js';

/**
 * The public function execution result.
 */
export interface PublicExecutionResult {
  /** The execution that triggered this result. */
  execution: PublicExecution;
  /** The return values of the function. */
  returnValues: Fr[];
  /** The contract storage reads performed by the function. */
  contractStorageReads: ContractStorageRead[];
  /** The contract storage update requests performed by the function. */
  contractStorageUpdateRequests: ContractStorageUpdateRequest[];
  /** The results of nested calls. */
  nestedExecutions: this[];
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
