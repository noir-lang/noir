import {
  AztecAddress,
  CallContext,
  Fr,
  FunctionData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  TxRequest,
} from '@aztec/circuits.js';

/**
 * The public function execution result.
 */
export interface PublicExecutionResult {
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
 * Returns whether the input is a public execution.
 * @param input - Input to check.
 * @returns Whether it's a public execution.
 */
export function isPublicExecution(input: PublicExecution | TxRequest): input is PublicExecution {
  const execution = input as PublicExecution;
  return !!execution.callContext && !!execution.args && !!execution.contractAddress && !!execution.functionData;
}
