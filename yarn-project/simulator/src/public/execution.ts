import { type SimulationError, type UnencryptedFunctionL2Logs } from '@aztec/circuit-types';
import {
  type AvmExecutionHints,
  type ContractStorageRead,
  type ContractStorageUpdateRequest,
  type Fr,
  type L2ToL1Message,
  type LogHash,
  type NoteHash,
  type Nullifier,
  type PublicCallRequest,
  type ReadRequest,
} from '@aztec/circuits.js';

import { type Gas } from '../avm/avm_gas.js';

/**
 * The public function execution result.
 */
export interface PublicExecutionResult {
  /** The execution request that triggered this result. */
  executionRequest: PublicExecutionRequest;

  /** The side effect counter at the start of the function call. */
  startSideEffectCounter: Fr;
  /** The side effect counter after executing this function call */
  endSideEffectCounter: Fr;
  /** How much gas was available for this public execution. */
  startGasLeft: Gas;
  /** How much gas was left after this public execution. */
  endGasLeft: Gas;
  /** Transaction fee set for this tx. */
  transactionFee: Fr;

  /** Bytecode used for this execution. */
  bytecode?: Buffer;
  /** Calldata used for this execution. */
  calldata: Fr[];
  /** The return values of the function. */
  returnValues: Fr[];
  /** Whether the execution reverted. */
  reverted: boolean;
  /** The revert reason if the execution reverted. */
  revertReason?: SimulationError;

  /** The contract storage reads performed by the function. */
  contractStorageReads: ContractStorageRead[];
  /** The contract storage update requests performed by the function. */
  contractStorageUpdateRequests: ContractStorageUpdateRequest[];
  /** The new note hashes to be inserted into the note hashes tree. */
  noteHashes: NoteHash[];
  /** The new l2 to l1 messages generated in this call. */
  l2ToL1Messages: L2ToL1Message[];
  /** The new nullifiers to be inserted into the nullifier tree. */
  nullifiers: Nullifier[];
  /** The note hash read requests emitted in this call. */
  noteHashReadRequests: ReadRequest[];
  /** The nullifier read requests emitted in this call. */
  nullifierReadRequests: ReadRequest[];
  /** The nullifier non existent read requests emitted in this call. */
  nullifierNonExistentReadRequests: ReadRequest[];
  /** L1 to L2 message read requests emitted in this call. */
  l1ToL2MsgReadRequests: ReadRequest[];
  /**
   * The hashed logs with side effect counter.
   * Note: required as we don't track the counter anywhere else.
   */
  unencryptedLogsHashes: LogHash[];
  /**
   * Unencrypted logs emitted during execution of this function call.
   * Note: These are preimages to `unencryptedLogsHashes`.
   */
  unencryptedLogs: UnencryptedFunctionL2Logs;
  /**
   * Unencrypted logs emitted during this call AND any nested calls.
   * Useful for maintaining correct ordering in ts.
   */
  allUnencryptedLogs: UnencryptedFunctionL2Logs;

  // TODO(dbanks12): add contract instance read requests

  /** The results of nested calls. */
  nestedExecutions: this[];

  /** Hints for proving AVM execution. */
  avmCircuitHints: AvmExecutionHints;

  /** The name of the function that was executed. Only used for logging. */
  functionName: string;
}

/**
 * The execution request of a public function.
 * A subset of PublicCallRequest
 */
export type PublicExecutionRequest = Pick<
  PublicCallRequest,
  'contractAddress' | 'functionSelector' | 'callContext' | 'args'
>;

/**
 * Returns if the input is a public execution result and not just a public execution.
 * @param input - Public execution or public execution result.
 * @returns Whether the input is a public execution result and not just a public execution.
 */
export function isPublicExecutionResult(
  input: PublicExecutionRequest | PublicExecutionResult,
): input is PublicExecutionResult {
  return 'executionRequest' in input && input.executionRequest !== undefined;
}

/**
 * Checks whether the child execution result is valid for a static call (no state modifications).
 * @param executionResult - The execution result of a public function
 */

export function checkValidStaticCall(
  noteHashes: NoteHash[],
  nullifiers: Nullifier[],
  contractStorageUpdateRequests: ContractStorageUpdateRequest[],
  l2ToL1Messages: L2ToL1Message[],
  unencryptedLogs: UnencryptedFunctionL2Logs,
) {
  if (
    contractStorageUpdateRequests.length > 0 ||
    noteHashes.length > 0 ||
    nullifiers.length > 0 ||
    l2ToL1Messages.length > 0 ||
    unencryptedLogs.logs.length > 0
  ) {
    throw new Error('Static call cannot update the state, emit L2->L1 messages or generate logs');
  }
}
