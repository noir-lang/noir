import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makePreviousKernelData, makePrivateKernelInputs, makePrivateKernelPublicInputs } from '../tests/factories.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

describe('structs/kernel', () => {
  it(`serializes and prints previous_kernel_data`, async () => {
    const previousKernelData = makePreviousKernelData();
    await expectSerializeToMatchSnapshot(
      previousKernelData.toBuffer(),
      'abis__test_roundtrip_serialize_previous_kernel_data',
    );
  });

  it(`serializes and prints private_kernel_inputs`, async () => {
    const kernelInputs = makePrivateKernelInputs();
    await expectSerializeToMatchSnapshot(
      kernelInputs.toBuffer(),
      'abis__test_roundtrip_serialize_private_kernel_inputs',
    );
  });

  // TODO: Reenable once we can move back to circuits master and have this c_bind available
  it.skip(`serializes and prints private_kernel_public_inputs`, async () => {
    const kernelInputs = makePrivateKernelPublicInputs();
    await expectSerializeToMatchSnapshot(
      kernelInputs.toBuffer(),
      'abis__test_roundtrip_serialize_private_kernel_public_inputs',
    );
  });
});
