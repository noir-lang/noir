import { PublicExecutionRequest } from '@aztec/circuit-types';
import { PrivateCallStackItem } from '@aztec/circuits.js';

import {
  type ExecutionResult,
  collectNoteHashLeafIndexMap,
  collectNullifiedNoteHashCounters,
} from './execution_result.js';

function emptyExecutionResult(): ExecutionResult {
  return {
    acir: Buffer.from(''),
    vk: Buffer.from(''),
    partialWitness: new Map(),
    callStackItem: PrivateCallStackItem.empty(),
    noteHashLeafIndexMap: new Map(),
    newNotes: [],
    nullifiedNoteHashCounters: new Map(),
    returnValues: [],
    nestedExecutions: [],
    enqueuedPublicFunctionCalls: [],
    publicTeardownFunctionCall: PublicExecutionRequest.empty(),
    noteEncryptedLogs: [],
    encryptedLogs: [],
    unencryptedLogs: [],
  };
}

describe('collectNoteHashLeafIndexMap', () => {
  let executionResult: ExecutionResult;

  beforeEach(() => {
    executionResult = emptyExecutionResult();
  });

  it('returns a map for note hash leaf indexes', () => {
    executionResult.noteHashLeafIndexMap = new Map();
    executionResult.noteHashLeafIndexMap.set(12n, 99n);
    executionResult.noteHashLeafIndexMap.set(34n, 88n);
    const res = collectNoteHashLeafIndexMap(executionResult);
    expect(res.size).toBe(2);
    expect(res.get(12n)).toBe(99n);
    expect(res.get(34n)).toBe(88n);
  });

  it('returns a map containing note hash leaf indexes for nested executions', () => {
    executionResult.noteHashLeafIndexMap.set(12n, 99n);
    executionResult.noteHashLeafIndexMap.set(34n, 88n);

    const childExecution0 = emptyExecutionResult();
    childExecution0.noteHashLeafIndexMap.set(56n, 77n);

    const childExecution1 = emptyExecutionResult();
    childExecution1.noteHashLeafIndexMap.set(78n, 66n);
    const grandchildExecution = emptyExecutionResult();
    grandchildExecution.noteHashLeafIndexMap.set(90n, 55n);
    childExecution1.nestedExecutions = [grandchildExecution];

    executionResult.nestedExecutions = [childExecution0, childExecution1];

    const res = collectNoteHashLeafIndexMap(executionResult);
    expect(res.size).toBe(5);
    expect(res.get(12n)).toBe(99n);
    expect(res.get(34n)).toBe(88n);
    expect(res.get(56n)).toBe(77n);
    expect(res.get(78n)).toBe(66n);
    expect(res.get(90n)).toBe(55n);
  });
});

describe('collectNullifiedNoteHashCounters', () => {
  let executionResult: ExecutionResult;

  beforeEach(() => {
    executionResult = emptyExecutionResult();
  });

  it('returns a map for note hash leaf indexes', () => {
    executionResult.nullifiedNoteHashCounters = new Map();
    executionResult.nullifiedNoteHashCounters.set(12, 99);
    executionResult.nullifiedNoteHashCounters.set(34, 88);
    const res = collectNullifiedNoteHashCounters(executionResult);
    expect(res.size).toBe(2);
    expect(res.get(12)).toBe(99);
    expect(res.get(34)).toBe(88);
  });

  it('returns a map containing note hash leaf indexes for nested executions', () => {
    executionResult.nullifiedNoteHashCounters.set(12, 99);
    executionResult.nullifiedNoteHashCounters.set(34, 88);

    const childExecution0 = emptyExecutionResult();
    childExecution0.nullifiedNoteHashCounters.set(56, 77);

    const childExecution1 = emptyExecutionResult();
    childExecution1.nullifiedNoteHashCounters.set(78, 66);
    const grandchildExecution = emptyExecutionResult();
    grandchildExecution.nullifiedNoteHashCounters.set(90, 55);
    childExecution1.nestedExecutions = [grandchildExecution];

    executionResult.nestedExecutions = [childExecution0, childExecution1];

    const res = collectNullifiedNoteHashCounters(executionResult);
    expect(res.size).toBe(5);
    expect(res.get(12)).toBe(99);
    expect(res.get(34)).toBe(88);
    expect(res.get(56)).toBe(77);
    expect(res.get(78)).toBe(66);
    expect(res.get(90)).toBe(55);
  });
});
