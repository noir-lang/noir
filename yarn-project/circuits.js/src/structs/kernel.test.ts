import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makePreviousKernelData, makePrivateKernelInputs, makePrivateKernelPublicInputs } from '../tests/factories.js';
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

  it(`serializes and prints private_kernel_inputs`, async () => {
    const wasm = await CircuitsWasm.new();
    const kernelInputs = makePrivateKernelInputs();
    await expectSerializeToMatchSnapshot(
      kernelInputs.toBuffer(),
      'abis__test_roundtrip_serialize_private_kernel_inputs',
      wasm,
    );
  });

  it(`serializes and prints private_kernel_public_inputs`, async () => {
    const wasm = await CircuitsWasm.new();
    const kernelInputs = makePrivateKernelPublicInputs();
    await expectSerializeToMatchSnapshot(
      kernelInputs.toBuffer(),
      'abis__test_roundtrip_serialize_private_kernel_public_inputs',
      wasm,
    );
  });
});
