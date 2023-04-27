import { AggregationObject, CircuitError, MergeRollupInputs, RootRollupInputs, VerificationKey } from '../index.js';
import { makeBaseRollupInputs, makeMergeRollupInputs, makeRootRollupInputs } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { RollupWasmWrapper } from './rollup_wasm_wrapper.js';

describe('rollup/rollup_wasm_wrapper', () => {
  let wasm: CircuitsWasm;
  let rollupWasm: RollupWasmWrapper;

  beforeAll(async () => {
    wasm = new CircuitsWasm();
    await wasm.init();
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
    input.previousRollupData[1].publicInputs.startPublicDataTreeSnapshot =
      input.previousRollupData[0].publicInputs.endPublicDataTreeSnapshot;
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

  it.skip('calls base_rollup__sim', async () => {
    const input = makeBaseRollupInputsForCircuit();

    const output = await rollupWasm.simulateBaseRollup(input);
    expect(output.startContractTreeSnapshot).toEqual(input.startContractTreeSnapshot);
    expect(output.startNullifierTreeSnapshot).toEqual(input.startNullifierTreeSnapshot);
    expect(output.startPrivateDataTreeSnapshot).toEqual(input.startPrivateDataTreeSnapshot);
  });

  it('calls merge_rollup__sim', async () => {
    const input = makeMergeRollupInputsForCircuit();

    const output = await rollupWasm.simulateMergeRollup(input);
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
    expect(output.constants.startTreeOfHistoricContractTreeRootsSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.constants.startTreeOfHistoricContractTreeRootsSnapshot,
    );
  });

  it('calling merge_rollup__sim with different constants should fail', async () => {
    const input = makeMergeRollupInputs();
    try {
      await rollupWasm.simulateMergeRollup(input);
    } catch (e) {
      expect(e).toBeInstanceOf(CircuitError);
      const err = e as CircuitError;
      expect(err.message).toEqual('input proofs have different constants');
      expect(err.code).toEqual(7003);
    }
  });

  it('calls root_rollup__sim', async () => {
    const input = makeRootRollupInputs();
    for (const rd of input.previousRollupData) {
      rd.vk = VerificationKey.makeFake();
      rd.publicInputs.endAggregationObject = AggregationObject.makeFake();
      rd.publicInputs = await rollupWasm.simulateBaseRollup(makeBaseRollupInputsForCircuit());
    }
    fixPreviousRollupInputs(input);

    const output = await rollupWasm.simulateRootRollup(input);
    expect(output.startNullifierTreeSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.startNullifierTreeSnapshot,
    );
  }, 15_000);
});
