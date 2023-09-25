import { expect } from "chai";
import {
  createBlackBoxSolver,
  executeCircuit,
  executeCircuitWithBlackBoxSolver,
  WasmBlackBoxFunctionSolver,
  WitnessMap,
  ForeignCallHandler,
} from "../../../result/";

it("successfully executes circuit and extracts return value", async () => {
  const { bytecode, initialWitnessMap, resultWitness, expectedResult } =
    await import("../shared/addition");

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  // Solved witness should be consistent with initial witness
  initialWitnessMap.forEach((value, key) => {
    expect(solvedWitness.get(key) as string).to.be.eq(value);
  });

  // Solved witness should contain expected return value
  expect(solvedWitness.get(resultWitness)).to.be.eq(expectedResult);
});

it("successfully processes simple brillig foreign call opcodes", async () => {
  const {
    bytecode,
    initialWitnessMap,
    expectedWitnessMap,
    oracleResponse,
    oracleCallName,
    oracleCallInputs,
  } = await import("../shared/foreign_call");

  let observedName = "";
  let observedInputs: string[][] = [];
  const foreignCallHandler: ForeignCallHandler = async (
    name: string,
    inputs: string[][]
  ) => {
    // Throwing inside the oracle callback causes a timeout so we log the observed values
    // and defer the check against expected values until after the execution is complete.
    observedName = name;
    observedInputs = inputs;

    return oracleResponse;
  };

  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    foreignCallHandler
  );

  // Check that expected values were passed to oracle callback.
  expect(observedName).to.be.eq(oracleCallName);
  expect(observedInputs).to.be.deep.eq(oracleCallInputs);

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // arithmetic opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solved_witness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully processes complex brillig foreign call opcodes", async () => {
  const {
    bytecode,
    initialWitnessMap,
    expectedWitnessMap,
    oracleResponse,
    oracleCallName,
    oracleCallInputs,
  } = await import("../shared/complex_foreign_call");

  let observedName = "";
  let observedInputs: string[][] = [];
  const foreignCallHandler: ForeignCallHandler = async (
    name: string,
    inputs: string[][]
  ) => {
    // Throwing inside the oracle callback causes a timeout so we log the observed values
    // and defer the check against expected values until after the execution is complete.
    observedName = name;
    observedInputs = inputs;

    return oracleResponse;
  };

  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    foreignCallHandler
  );

  // Check that expected values were passed to oracle callback.
  expect(observedName).to.be.eq(oracleCallName);
  expect(observedInputs).to.be.deep.eq(oracleCallInputs);

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // arithmetic opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solved_witness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a Pedersen opcode", async function () {
  this.timeout(10000);
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/pedersen"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a FixedBaseScalarMul opcode", async () => {
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/fixed_base_scalar_mul"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a SchnorrVerify opcode", async () => {
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/schnorr_verify"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a MemoryOp opcode", async () => {
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/memory_op"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes two circuits with same backend", async function () {
  this.timeout(10000);

  // chose pedersen op here because it is the one with slow initialization
  // that led to the decision to pull backend initialization into a separate
  // function/wasmbind
  const solver: WasmBlackBoxFunctionSolver = await createBlackBoxSolver();

  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/pedersen"
  );

  const solvedWitness0 = await executeCircuitWithBlackBoxSolver(
    solver,
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  const solvedWitness1 = await executeCircuitWithBlackBoxSolver(
    solver,
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness0).to.be.deep.eq(expectedWitnessMap);
  expect(solvedWitness1).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes 500 circuits with same backend", async function () {
  this.timeout(100000);

  // chose pedersen op here because it is the one with slow initialization
  // that led to the decision to pull backend initialization into a separate
  // function/wasmbind
  const solver: WasmBlackBoxFunctionSolver = await createBlackBoxSolver();

  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/pedersen"
  );

  for (let i = 0; i < 500; i++) {
    const solvedWitness = await executeCircuitWithBlackBoxSolver(
      solver,
      bytecode,
      initialWitnessMap,
      () => {
        throw Error("unexpected oracle");
      }
    );

    expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
  }
});
