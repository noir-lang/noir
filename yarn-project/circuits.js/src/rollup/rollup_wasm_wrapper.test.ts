import { AggregationObject, CircuitError, MergeRollupInputs, RootRollupInputs, VerificationKey } from '../index.js';
import { makeBaseRollupInputs, makeMergeRollupInputs, makeRootRollupInputs } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { RollupWasmWrapper } from './rollup_wasm_wrapper.js';

describe('rollup/rollup_wasm_wrapper', () => {
  let wasm: CircuitsWasm;
  let rollupWasm: RollupWasmWrapper;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
    rollupWasm = new RollupWasmWrapper(wasm);
  });

  const makeBaseRollupInputsForCircuit = () => {
    const input = makeBaseRollupInputs();
    for (const kd of input.kernelData) {
      kd.vk = VerificationKey.makeFake();
      kd.publicInputs.end.aggregationObject = AggregationObject.makeFake();
    }
    return input;
  };

  const fixPreviousRollupInputs = (input: MergeRollupInputs | RootRollupInputs) => {
    input.previousRollupData[1].publicInputs.constants = input.previousRollupData[0].publicInputs.constants;
    input.previousRollupData[1].publicInputs.startPrivateDataTreeSnapshot =
      input.previousRollupData[0].publicInputs.endPrivateDataTreeSnapshot;
    input.previousRollupData[1].publicInputs.startNullifierTreeSnapshot =
      input.previousRollupData[0].publicInputs.endNullifierTreeSnapshot;
    input.previousRollupData[1].publicInputs.startContractTreeSnapshot =
      input.previousRollupData[0].publicInputs.endContractTreeSnapshot;
    input.previousRollupData[1].publicInputs.startPublicDataTreeRoot =
      input.previousRollupData[0].publicInputs.endPublicDataTreeRoot;
  };

  const makeMergeRollupInputsForCircuit = () => {
    const input = makeMergeRollupInputs();
    for (const previousData of input.previousRollupData) {
      previousData.vk = VerificationKey.makeFake();
      previousData.publicInputs.endAggregationObject = AggregationObject.makeFake();
    }
    fixPreviousRollupInputs(input);
    return input;
  };

  // Task to repair this test: https://github.com/AztecProtocol/aztec-packages/issues/1586
  it.skip('calls base_rollup__sim', () => {
    const input = makeBaseRollupInputsForCircuit();

    const output = rollupWasm.simulateBaseRollup(input);
    expect(output.startContractTreeSnapshot).toEqual(input.startContractTreeSnapshot);
    expect(output.startNullifierTreeSnapshot).toEqual(input.startNullifierTreeSnapshot);
    expect(output.startPrivateDataTreeSnapshot).toEqual(input.startPrivateDataTreeSnapshot);
  });

  it('calls merge_rollup__sim', () => {
    const input = makeMergeRollupInputsForCircuit();

    const output = rollupWasm.simulateMergeRollup(input);
    expect(output.rollupType).toEqual(1);
    expect(output.startContractTreeSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.startContractTreeSnapshot,
    );
    expect(output.startNullifierTreeSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.startNullifierTreeSnapshot,
    );
    expect(output.startPrivateDataTreeSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.startPrivateDataTreeSnapshot,
    );
    expect(output.endPrivateDataTreeSnapshot).toEqual(
      input.previousRollupData[1].publicInputs.endPrivateDataTreeSnapshot,
    );
  });

  it('calling merge_rollup__sim with different constants should fail', () => {
    const input = makeMergeRollupInputs();
    try {
      rollupWasm.simulateMergeRollup(input);
    } catch (e) {
      expect(e).toBeInstanceOf(CircuitError);
      const err = e as CircuitError;
      expect(err.message).toEqual(
        `input proofs have different constants
Refer to https://docs.aztec.network/aztec/protocol/errors for more information.`,
      );
      expect(err.code).toEqual(7003);
    }
  });

  // Task to repair this test: https://github.com/AztecProtocol/aztec-packages/issues/1586
  it.skip('calls root_rollup__sim', () => {
    const input = makeRootRollupInputs();
    for (const rd of input.previousRollupData) {
      rd.vk = VerificationKey.makeFake();
      rd.publicInputs.endAggregationObject = AggregationObject.makeFake();
      rd.publicInputs = rollupWasm.simulateBaseRollup(makeBaseRollupInputsForCircuit());
    }
    fixPreviousRollupInputs(input);

    const output = rollupWasm.simulateRootRollup(input);
    expect(output.startNullifierTreeSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.startNullifierTreeSnapshot,
    );
  }, 15_000);
});
