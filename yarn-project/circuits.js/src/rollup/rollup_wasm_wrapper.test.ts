import {
  AggregationObject,
  BaseOrMergeRollupPublicInputs,
  CircuitError,
  MergeRollupInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  VerificationKey,
  baseRollupSim,
  mergeRollupSim,
  rootRollupSim,
} from '../index.js';
import { makeBaseRollupInputs, makeMergeRollupInputs, makeRootRollupInputs } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

describe('rollup/rollup_wasm_wrapper', () => {
  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
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
    input.previousRollupData[1].baseOrMergeRollupPublicInputs.constants =
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.constants;
    input.previousRollupData[1].baseOrMergeRollupPublicInputs.startPrivateDataTreeSnapshot =
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.endPrivateDataTreeSnapshot;
    input.previousRollupData[1].baseOrMergeRollupPublicInputs.startNullifierTreeSnapshot =
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.endNullifierTreeSnapshot;
    input.previousRollupData[1].baseOrMergeRollupPublicInputs.startContractTreeSnapshot =
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.endContractTreeSnapshot;
    input.previousRollupData[1].baseOrMergeRollupPublicInputs.startPublicDataTreeRoot =
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.endPublicDataTreeRoot;
  };

  const makeMergeRollupInputsForCircuit = () => {
    const input = makeMergeRollupInputs();
    for (const previousData of input.previousRollupData) {
      previousData.vk = VerificationKey.makeFake();
      previousData.baseOrMergeRollupPublicInputs.endAggregationObject = AggregationObject.makeFake();
    }
    fixPreviousRollupInputs(input);
    return input;
  };

  // Task to repair this test: https://github.com/AztecProtocol/aztec-packages/issues/1586
  it.skip('calls base_rollup__sim', () => {
    const input = makeBaseRollupInputsForCircuit();
    const output = baseRollupSim(wasm, input);
    expect(output instanceof BaseOrMergeRollupPublicInputs).toBeTruthy();

    const publicInputs = output as BaseOrMergeRollupPublicInputs;
    expect(publicInputs.startContractTreeSnapshot).toEqual(input.startContractTreeSnapshot);
    expect(publicInputs.startNullifierTreeSnapshot).toEqual(input.startNullifierTreeSnapshot);
    expect(publicInputs.startPrivateDataTreeSnapshot).toEqual(input.startPrivateDataTreeSnapshot);
  });

  it('calls merge_rollup__sim', () => {
    const input = makeMergeRollupInputsForCircuit();

    const output = mergeRollupSim(wasm, input);
    if (output instanceof CircuitError) {
      throw new CircuitError(output.code, output.message);
    }

    expect(output.rollupType).toEqual(1);
    expect(output.startContractTreeSnapshot).toEqual(
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.startContractTreeSnapshot,
    );
    expect(output.startNullifierTreeSnapshot).toEqual(
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.startNullifierTreeSnapshot,
    );
    expect(output.startPrivateDataTreeSnapshot).toEqual(
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.startPrivateDataTreeSnapshot,
    );
    expect(output.endPrivateDataTreeSnapshot).toEqual(
      input.previousRollupData[1].baseOrMergeRollupPublicInputs.endPrivateDataTreeSnapshot,
    );
  });

  it('calling merge_rollup__sim with different constants should fail', () => {
    const input = makeMergeRollupInputs();

    const output = mergeRollupSim(wasm, input);
    expect(output instanceof CircuitError).toBeTruthy();

    const err = output as CircuitError;
    expect(err.message).toEqual(
      `input proofs have different constants`,
      // Refer to https://docs.aztec.network/aztec/protocol/errors for more information.`,
    );
    expect(err.code).toEqual(7003);
  });

  // Task to repair this test: https://github.com/AztecProtocol/aztec-packages/issues/1586
  it.skip('calls root_rollup__sim', () => {
    const input = makeRootRollupInputs();
    for (const rd of input.previousRollupData) {
      rd.vk = VerificationKey.makeFake();
      rd.baseOrMergeRollupPublicInputs.endAggregationObject = AggregationObject.makeFake();
      const output = baseRollupSim(wasm, makeBaseRollupInputsForCircuit());
      expect(output instanceof BaseOrMergeRollupPublicInputs).toBeTruthy();
      rd.baseOrMergeRollupPublicInputs = output as BaseOrMergeRollupPublicInputs;
    }
    fixPreviousRollupInputs(input);

    const output = rootRollupSim(wasm, input);
    expect(output instanceof RootRollupPublicInputs).toBeTruthy();

    const publicInputs = output as RootRollupPublicInputs;
    expect(publicInputs.startNullifierTreeSnapshot).toEqual(
      input.previousRollupData[0].baseOrMergeRollupPublicInputs.startNullifierTreeSnapshot,
    );
  }, 15_000);
});
