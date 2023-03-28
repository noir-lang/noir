import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makePreviousKernelData } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

describe('structs/kernel', () => {
  it(`serializes and prints previous_kernel_data`, async () => {
    const wasm = await CircuitsWasm.new();
    const previousKernelData = makePreviousKernelData();
    await expectSerializeToMatchSnapshot(
      previousKernelData.toBuffer(),
      'abis__test_roundtrip_serialize_previous_kernel_data',
      wasm,
    );
  });
});
