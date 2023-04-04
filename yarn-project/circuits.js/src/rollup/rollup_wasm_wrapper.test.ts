import { AggregationObject, PreviousKernelData, VerificationKey } from '../index.js';
import { makeBaseRollupInputs, makeRootRollupInputs } from '../tests/factories.js';
import { uint8ArrayToNum } from '../utils/serialize.js';
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

  it('should new malloc, transfer and slice mem', async () => {
    const ptr = wasm.call('bbmalloc', 4);
    const data = await wasm.asyncCall('private_kernel__dummy_previous_kernel', ptr);
    const outputBufSize = uint8ArrayToNum(wasm.getMemorySlice(ptr, ptr + 4));
    console.log(`size ${outputBufSize}`);
    wasm.call('bbfree', ptr);
    const result = Buffer.from(wasm.getMemorySlice(data, data + outputBufSize));
    const kernel = PreviousKernelData.fromBuffer(result);
    console.log(`kernel `, kernel);
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
