import { AggregationObject, VerificationKey } from '../index.js';
import { makeBaseRollupInputs, makeRootRollupInputs } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { RollupWasmWrapper } from './rollup_wasm_wrapper.js';

// TODO: All these tests are currently failing with segfaults.
// Note that base and root rollup sim are called ok from the circuit_powered_block_builder,
// so the problem must be with an invalid input we're providing.
describe.skip('rollup/rollup_wasm_wrapper', () => {
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

  it('calls base_rollup__sim', async () => {
    const input = makeBaseRollupInputsForCircuit();

    const output = await rollupWasm.simulateBaseRollup(input);
    expect(output.startContractTreeSnapshot).toEqual(input.startContractTreeSnapshot);
    expect(output.startNullifierTreeSnapshot).toEqual(input.startNullifierTreeSnapshot);
    expect(output.startPrivateDataTreeSnapshot).toEqual(input.startPrivateDataTreeSnapshot);
  });

  it('calls root_rollup__sim', async () => {
    const input = makeRootRollupInputs();

    for (const rd of input.previousRollupData) {
      rd.vk = VerificationKey.makeFake();
      rd.publicInputs.endAggregationObject = AggregationObject.makeFake();
      rd.publicInputs = await rollupWasm.simulateBaseRollup(makeBaseRollupInputsForCircuit());
    }

    const output = await rollupWasm.simulateRootRollup(input);
    expect(output.startNullifierTreeSnapshot).toEqual(
      input.previousRollupData[0].publicInputs.startNullifierTreeSnapshot,
    );
  });
});
