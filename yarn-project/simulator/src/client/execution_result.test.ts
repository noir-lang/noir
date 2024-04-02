import {
  EncryptedFunctionL2Logs,
  EncryptedL2Log,
  UnencryptedFunctionL2Logs,
  UnencryptedL2Log,
} from '@aztec/circuit-types';
import { AztecAddress, PrivateCallStackItem } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';

import { type ExecutionResult, collectEncryptedLogs, collectUnencryptedLogs } from './execution_result.js';

function emptyExecutionResult(): ExecutionResult {
  return {
    acir: Buffer.from(''),
    vk: Buffer.from(''),
    partialWitness: new Map(),
    callStackItem: PrivateCallStackItem.empty(),
    noteHashReadRequestPartialWitnesses: [],
    newNotes: [],
    returnValues: [],
    nestedExecutions: [],
    enqueuedPublicFunctionCalls: [],
    encryptedLogs: EncryptedFunctionL2Logs.empty(),
    unencryptedLogs: UnencryptedFunctionL2Logs.empty(),
  };
}

describe('Execution Result test suite - collect encrypted logs', () => {
  function emptyExecutionResultWithEncryptedLogs(encryptedLogs = EncryptedFunctionL2Logs.empty()): ExecutionResult {
    const executionResult = emptyExecutionResult();
    executionResult.encryptedLogs = encryptedLogs;
    return executionResult;
  }

  function makeEncryptedFunctionLogs(contents: string[]) {
    return new EncryptedFunctionL2Logs(contents.map(s => new EncryptedL2Log(Buffer.from(s))));
  }

  it('collect encrypted logs with nested fn calls', () => {
    /*
    Create the following executionResult object: 
    fnA (log1) 
        |---------->fnB (log2) 
        |---------->fnC (log3) -> fnD (log4)
        |---------->fnE (log5) 
                     |-------->fnF (log6)
                     |-------->fnG (log7) 
    Circuits and ACVM process in a DFS + stack like format: [fnA, fnE, fnG, fnF, fnC, fnD, fnB]
    */
    const executionResult: ExecutionResult = emptyExecutionResultWithEncryptedLogs(
      makeEncryptedFunctionLogs(['Log 1']),
    );
    const fnB = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 2']));
    const fnC = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 3']));
    const fnD = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 4']));
    const fnE = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 5']));
    const fnF = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 6']));
    const fnG = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 7']));

    fnE.nestedExecutions.push(fnF, fnG);

    fnC.nestedExecutions.push(fnD);

    executionResult.nestedExecutions.push(fnB, fnC, fnE);

    const encryptedLogs = collectEncryptedLogs(executionResult);
    expect(encryptedLogs).toEqual([
      makeEncryptedFunctionLogs(['Log 1']),
      makeEncryptedFunctionLogs(['Log 5']),
      makeEncryptedFunctionLogs(['Log 7']),
      makeEncryptedFunctionLogs(['Log 6']),
      makeEncryptedFunctionLogs(['Log 3']),
      makeEncryptedFunctionLogs(['Log 4']),
      makeEncryptedFunctionLogs(['Log 2']),
    ]);
  });

  it('collect encrypted logs with multiple logs each function call', () => {
    /*
    Create the following executionResult object: 
    fnA (log1, log2) 
        |---------->fnB (log3, log4) 
        |---------->fnC (log5) -> fnD (log6)
    Circuits and ACVM process in a DFS + stack like format: [fnA, fnC, fnD, fnB]
    */
    const executionResult: ExecutionResult = emptyExecutionResultWithEncryptedLogs(
      makeEncryptedFunctionLogs(['Log 1', 'Log 2']),
    );
    const fnB = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 3', 'Log 4']));
    const fnC = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 5']));
    const fnD = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 6']));
    fnC.nestedExecutions.push(fnD);
    executionResult.nestedExecutions.push(fnB, fnC);
    const encryptedLogs = collectEncryptedLogs(executionResult);
    expect(encryptedLogs).toEqual([
      makeEncryptedFunctionLogs(['Log 1', 'Log 2']),
      makeEncryptedFunctionLogs(['Log 5']),
      makeEncryptedFunctionLogs(['Log 6']),
      makeEncryptedFunctionLogs(['Log 3', 'Log 4']),
    ]);
  });

  it('collect encrypted logs with nested functions where some have no logs', () => {
    /*
    Create the following executionResult object: 
    fnA () 
        |----------> fnB (log1) -> fnC ()
    Circuits and ACVM process in a DFS + stack like format: [fnA, fnB, fnC]
    */
    const executionResult: ExecutionResult = emptyExecutionResult();
    const fnB = emptyExecutionResultWithEncryptedLogs(makeEncryptedFunctionLogs(['Log 1']));
    const fnC = emptyExecutionResult();
    fnB.nestedExecutions.push(fnC);
    executionResult.nestedExecutions.push(fnB);
    const encryptedLogs = collectEncryptedLogs(executionResult);
    expect(encryptedLogs).toEqual([
      EncryptedFunctionL2Logs.empty(),
      makeEncryptedFunctionLogs(['Log 1']),
      EncryptedFunctionL2Logs.empty(),
    ]);
  });

  it('collect encrypted logs with no logs in any nested calls', () => {
    /*
    Create the following executionResult object:
    fnA ()
      |----------> fnB () -> fnC ()
      |----------> fnD () -> fnE ()
    Circuits and ACVM process in a DFS + stack like format: [fnA, fnD, fnE, fnB, fnC]
    */
    const executionResult: ExecutionResult = emptyExecutionResult();
    const fnB = emptyExecutionResult();
    const fnC = emptyExecutionResult();
    const fnD = emptyExecutionResult();
    const fnE = emptyExecutionResult();

    fnB.nestedExecutions.push(fnC);
    fnD.nestedExecutions.push(fnE);

    executionResult.nestedExecutions.push(fnB, fnD);

    const encryptedLogs = collectEncryptedLogs(executionResult);
    expect(encryptedLogs).toEqual([
      EncryptedFunctionL2Logs.empty(),
      EncryptedFunctionL2Logs.empty(),
      EncryptedFunctionL2Logs.empty(),
      EncryptedFunctionL2Logs.empty(),
      EncryptedFunctionL2Logs.empty(),
    ]);
  });
});

describe('collect unencrypted logs', () => {
  // collection of unencrypted logs work similar to encrypted logs, so lets write other kinds of test cases:

  function emptyExecutionResultWithUnencryptedLogs(
    unencryptedLogs = UnencryptedFunctionL2Logs.empty(),
  ): ExecutionResult {
    const executionResult = emptyExecutionResult();
    executionResult.unencryptedLogs = unencryptedLogs;
    return executionResult;
  }

  function makeUnencryptedFunctionLogs(contents: string[]) {
    return new UnencryptedFunctionL2Logs(
      contents.map(s => new UnencryptedL2Log(AztecAddress.ZERO, EventSelector.empty(), Buffer.from(s))),
    );
  }

  it('collect unencrypted logs even when no logs and no recursion', () => {
    // fnA()
    const executionResult: ExecutionResult = emptyExecutionResult();
    const unencryptedLogs = collectUnencryptedLogs(executionResult);
    expect(unencryptedLogs).toEqual([UnencryptedFunctionL2Logs.empty()]);
  });

  it('collect unencrypted logs with no logs in some nested calls', () => {
    /*
    Create the following executionResult object: 
    fnA () 
        |----------> fnB () -> fnC (log1, log2, log3)
    Circuits and ACVM process in a DFS + stack like format: [fnA, fnC, fnB]
    */
    const executionResult: ExecutionResult = emptyExecutionResult();
    const fnB = emptyExecutionResult();
    const fnC = emptyExecutionResultWithUnencryptedLogs(makeUnencryptedFunctionLogs(['Log 1', 'Log 2', 'Log 3']));

    executionResult.nestedExecutions.push(fnB, fnC);

    const unencryptedLogs = collectUnencryptedLogs(executionResult);
    expect(unencryptedLogs).toEqual([
      UnencryptedFunctionL2Logs.empty(),
      makeUnencryptedFunctionLogs(['Log 1', 'Log 2', 'Log 3']),
      UnencryptedFunctionL2Logs.empty(),
    ]);
  });

  it('collect unencrypted logs with multiple logs in each function call leaves', () => {
    /*
    Create the following executionResult object:
    fnA()
      |->fnB
          |->fnC(log1, log2, log3)
          |->fnD(log4, log5, log6)
    Circuits and ACVM process in a DFS + stack like format: [fnA, fnB, fnD, fnC]
    */
    const executionResult: ExecutionResult = emptyExecutionResult();
    const fnB = emptyExecutionResult();
    const fnC = emptyExecutionResultWithUnencryptedLogs(makeUnencryptedFunctionLogs(['Log 1', 'Log 2', 'Log 3']));
    const fnD = emptyExecutionResultWithUnencryptedLogs(makeUnencryptedFunctionLogs(['Log 4', 'Log 5', 'Log 6']));
    fnB.nestedExecutions.push(fnC, fnD);
    executionResult.nestedExecutions.push(fnB);
    const unencryptedLogs = collectUnencryptedLogs(executionResult);
    expect(unencryptedLogs).toEqual([
      UnencryptedFunctionL2Logs.empty(),
      UnencryptedFunctionL2Logs.empty(),
      makeUnencryptedFunctionLogs(['Log 4', 'Log 5', 'Log 6']),
      makeUnencryptedFunctionLogs(['Log 1', 'Log 2', 'Log 3']),
    ]);
  });
});
