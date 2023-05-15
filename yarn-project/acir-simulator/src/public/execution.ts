import {
  AztecAddress,
  CallContext,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  FunctionData,
} from '@aztec/circuits.js';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { keccak } from '@aztec/foundation/crypto';

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
