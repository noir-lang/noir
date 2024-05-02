import {
  EncryptedFunctionL2Logs,
  type EncryptedL2Log,
  type Note,
  UnencryptedFunctionL2Logs,
  type UnencryptedL2Log,
} from '@aztec/circuit-types';
import {
  type IsEmpty,
  type NoteHashReadRequestMembershipWitness,
  type PrivateCallStackItem,
  type PublicCallRequest,
  sortByCounter,
} from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type ACVMField } from '../acvm/index.js';

/**
 * The contents of a new note.
 */
export interface NoteAndSlot {
  /** The note. */
  note: Note;
  /** The storage slot of the note. */
  storageSlot: Fr;
  /** The note type identifier. */
  noteTypeId: Fr;
}

export class CountedLog<TLog extends UnencryptedL2Log | EncryptedL2Log> implements IsEmpty {
  constructor(public log: TLog, public counter: number) {}

  isEmpty(): boolean {
    return !this.log.data.length && !this.counter;
  }
}

export interface NullifiedNoteHashCounter {
  noteHashCounter: number;
  nullifierCounter: number;
}

/**
 * The result of executing a private function.
 */
export interface ExecutionResult {
  // Needed for prover
  /** The ACIR bytecode. */
  acir: Buffer;
  /** The verification key. */
  vk: Buffer;
  /** The partial witness. */
  partialWitness: Map<number, ACVMField>;
  // Needed for the verifier (kernel)
  /** The call stack item. */
  callStackItem: PrivateCallStackItem;
  /** The partially filled-in read request membership witnesses for commitments being read. */
  noteHashReadRequestPartialWitnesses: NoteHashReadRequestMembershipWitness[];
  /** The notes created in the executed function. */
  newNotes: NoteAndSlot[];
  nullifiedNoteHashCounters: NullifiedNoteHashCounter[];
  /** The raw return values of the executed function. */
  returnValues: Fr[];
  /** The nested executions. */
  nestedExecutions: this[];
  /** Enqueued public function execution requests to be picked up by the sequencer. */
  enqueuedPublicFunctionCalls: PublicCallRequest[];
  /**
   * Encrypted logs emitted during execution of this function call.
   * Note: These are preimages to `encryptedLogsHashes`.
   */
  encryptedLogs: CountedLog<EncryptedL2Log>[];
  /**
   * Unencrypted logs emitted during execution of this function call.
   * Note: These are preimages to `unencryptedLogsHashes`.
   */
  unencryptedLogs: CountedLog<UnencryptedL2Log>[];
}

export function collectNullifiedNoteHashCounters(execResult: ExecutionResult): NullifiedNoteHashCounter[] {
  return [
    execResult.nullifiedNoteHashCounters,
    ...execResult.nestedExecutions.flatMap(collectNullifiedNoteHashCounters),
  ].flat();
}

/**
 * Collect all encrypted logs across all nested executions.
 * @param execResult - The topmost execution result.
 * @returns All encrypted logs.
 */
function collectEncryptedLogs(execResult: ExecutionResult): CountedLog<EncryptedL2Log>[] {
  return [execResult.encryptedLogs, ...[...execResult.nestedExecutions].flatMap(collectEncryptedLogs)].flat();
}

/**
 * Collect all encrypted logs across all nested executions and sorts by counter.
 * @param execResult - The topmost execution result.
 * @returns All encrypted logs.
 */
export function collectSortedEncryptedLogs(execResult: ExecutionResult): EncryptedFunctionL2Logs {
  const allLogs = collectEncryptedLogs(execResult);
  const sortedLogs = sortByCounter(allLogs);
  return new EncryptedFunctionL2Logs(sortedLogs.map(l => l.log));
}

/**
 * Collect all unencrypted logs across all nested executions.
 * @param execResult - The topmost execution result.
 * @returns All unencrypted logs.
 */
function collectUnencryptedLogs(execResult: ExecutionResult): CountedLog<UnencryptedL2Log>[] {
  return [execResult.unencryptedLogs, ...[...execResult.nestedExecutions].flatMap(collectUnencryptedLogs)].flat();
}

/**
 * Collect all unencrypted logs across all nested executions and sorts by counter.
 * @param execResult - The topmost execution result.
 * @returns All unencrypted logs.
 */
export function collectSortedUnencryptedLogs(execResult: ExecutionResult): UnencryptedFunctionL2Logs {
  const allLogs = collectUnencryptedLogs(execResult);
  const sortedLogs = sortByCounter(allLogs);
  return new UnencryptedFunctionL2Logs(sortedLogs.map(l => l.log));
}

/**
 * Collect all enqueued public function calls across all nested executions.
 * @param execResult - The topmost execution result.
 * @returns All enqueued public function calls.
 */
export function collectEnqueuedPublicFunctionCalls(execResult: ExecutionResult): PublicCallRequest[] {
  // without the reverse sort, the logs will be in a queue like fashion which is wrong
  // as the kernel processes it like a stack, popping items off and pushing them to output
  return [
    ...execResult.enqueuedPublicFunctionCalls,
    ...[...execResult.nestedExecutions].flatMap(collectEnqueuedPublicFunctionCalls),
  ].sort((a, b) => b.callContext.sideEffectCounter - a.callContext.sideEffectCounter);
}
