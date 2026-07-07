import { expect } from 'chai';
import {
  executeCircuit,
  WitnessMap,
  ForeignCallHandler,
  executeProgram,
  WitnessStack,
  StackItem,
} from '@noir-lang/acvm_js';

import {
  bytecode as additionBytecode,
  initialWitnessMap as additionInitialWitnessMap,
  resultWitness as additionResultWitness,
  expectedResult as additionExpectedResult,
} from '../shared/addition';
import {
  bytecode as foreignCallBytecode,
  initialWitnessMap as foreignCallInitialWitnessMap,
  expectedWitnessMap as foreignCallExpectedWitnessMap,
  oracleResponse as foreignCallOracleResponse,
  oracleCallName as foreignCallOracleCallName,
  oracleCallInputs as foreignCallOracleCallInputs,
} from '../shared/foreign_call';
import {
  bytecode as complexForeignCallBytecode,
  initialWitnessMap as complexForeignCallInitialWitnessMap,
  expectedWitnessMap as complexForeignCallExpectedWitnessMap,
  oracleResponse as complexForeignCallOracleResponse,
  oracleCallName as complexForeignCallOracleCallName,
  oracleCallInputs as complexForeignCallOracleCallInputs,
} from '../shared/complex_foreign_call';
import {
  bytecode as multiScalarMulBytecode,
  initialWitnessMap as multiScalarMulInitialWitnessMap,
  expectedWitnessMap as multiScalarMulExpectedWitnessMap,
} from '../shared/multi_scalar_mul';
import {
  bytecode as memoryOpBytecode,
  initialWitnessMap as memoryOpInitialWitnessMap,
  expectedWitnessMap as memoryOpExpectedWitnessMap,
} from '../shared/memory_op';
import {
  bytecode as nestedCallBytecode,
  initialWitnessMap as nestedCallInitialWitnessMap,
  expectedWitnessStack as nestedCallExpectedWitnessStack,
} from '../shared/nested_acir_call';

it('successfully executes circuit and extracts return value', async () => {
  const solvedWitness: WitnessMap = await executeCircuit(additionBytecode, additionInitialWitnessMap, () => {
    throw Error('unexpected oracle');
  });

  // Solved witness should be consistent with initial witness
  additionInitialWitnessMap.forEach((value, key) => {
    expect(solvedWitness.get(key) as string).to.be.eq(value);
  });

  // Solved witness should contain expected return value
  expect(solvedWitness.get(additionResultWitness)).to.be.eq(additionExpectedResult);
});

it('successfully processes simple brillig foreign call opcodes', async () => {
  let observedName = '';
  let observedInputs: string[][] = [];
  const foreignCallHandler: ForeignCallHandler = async (name: string, inputs: string[][]) => {
    // Throwing inside the oracle callback causes a timeout so we log the observed values
    // and defer the check against expected values until after the execution is complete.
    observedName = name;
    observedInputs = inputs;

    return foreignCallOracleResponse;
  };

  const solvedWitness: WitnessMap = await executeCircuit(
    foreignCallBytecode,
    foreignCallInitialWitnessMap,
    foreignCallHandler,
  );

  // Check that expected values were passed to oracle callback.
  expect(observedName).to.be.eq(foreignCallOracleCallName);
  expect(observedInputs).to.be.deep.eq(foreignCallOracleCallInputs);

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // assert-zero opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solvedWitness).to.be.deep.eq(foreignCallExpectedWitnessMap);
});

it('successfully processes complex brillig foreign call opcodes', async () => {
  let observedName = '';
  let observedInputs: string[][] = [];
  const foreignCallHandler: ForeignCallHandler = async (name: string, inputs: string[][]) => {
    // Throwing inside the oracle callback causes a timeout so we log the observed values
    // and defer the check against expected values until after the execution is complete.
    observedName = name;
    observedInputs = inputs;

    return complexForeignCallOracleResponse;
  };

  const solvedWitness: WitnessMap = await executeCircuit(
    complexForeignCallBytecode,
    complexForeignCallInitialWitnessMap,
    foreignCallHandler,
  );

  // Check that expected values were passed to oracle callback.
  expect(observedName).to.be.eq(complexForeignCallOracleCallName);
  expect(observedInputs).to.be.deep.eq(complexForeignCallOracleCallInputs);

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // assert-zero opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solvedWitness).to.be.deep.eq(complexForeignCallExpectedWitnessMap);
});

it('successfully executes a MultiScalarMul opcode', async () => {
  const solvedWitness: WitnessMap = await executeCircuit(
    multiScalarMulBytecode,
    multiScalarMulInitialWitnessMap,
    () => {
      throw Error('unexpected oracle');
    },
  );

  expect(solvedWitness).to.be.deep.eq(multiScalarMulExpectedWitnessMap);
});

it('successfully executes a MemoryOp opcode', async () => {
  const solvedWitness: WitnessMap = await executeCircuit(memoryOpBytecode, memoryOpInitialWitnessMap, () => {
    throw Error('unexpected oracle');
  });

  expect(solvedWitness).to.be.deep.eq(memoryOpExpectedWitnessMap);
});

/**
 * Below are all the same tests as above but using `executeProgram`
 * TODO: also add a couple tests for executing multiple circuits
 */
it('executeProgram: successfully executes program and extracts return value', async () => {
  const witnessStack: WitnessStack = await executeProgram(additionBytecode, additionInitialWitnessMap, () => {
    throw Error('unexpected oracle');
  });

  const solvedStackItem: StackItem = witnessStack[0];
  expect(solvedStackItem.index).to.be.eq(0);
  const solvedWitnessMap: WitnessMap = solvedStackItem.witness;

  // Witness stack should be consistent with initial witness
  additionInitialWitnessMap.forEach((value, key) => {
    expect(solvedWitnessMap.get(key) as string).to.be.eq(value);
  });

  // Solved witness should contain expected return value
  expect(solvedWitnessMap.get(additionResultWitness)).to.be.eq(additionExpectedResult);
});

it('executeProgram: successfully process a program of acir functions with a nested call', async () => {
  const witnessStack: WitnessStack = await executeProgram(nestedCallBytecode, nestedCallInitialWitnessMap, () => {
    throw Error('unexpected oracle');
  });

  expect(witnessStack).to.be.deep.eq(nestedCallExpectedWitnessStack);
});
