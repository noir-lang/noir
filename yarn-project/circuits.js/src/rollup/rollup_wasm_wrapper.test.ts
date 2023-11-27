import { AggregationObject, BaseOrMergeRollupPublicInputs, VerificationKey, baseRollupSim } from '../index.js';
import { makeBaseRollupInputs } from '../tests/factories.js';
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

  // Task to repair this test: https://github.com/AztecProtocol/aztec-packages/issues/1586
  it.skip('calls base_rollup__sim', () => {
    const input = makeBaseRollupInputsForCircuit();
    const output = baseRollupSim(wasm, input);
    expect(output instanceof BaseOrMergeRollupPublicInputs).toBeTruthy();

    const publicInputs = output as BaseOrMergeRollupPublicInputs;
    expect(publicInputs.startContractTreeSnapshot).toEqual(input.startContractTreeSnapshot);
    expect(publicInputs.startNullifierTreeSnapshot).toEqual(input.startNullifierTreeSnapshot);
    expect(publicInputs.startNoteHashTreeSnapshot).toEqual(input.startNoteHashTreeSnapshot);
  });
});
