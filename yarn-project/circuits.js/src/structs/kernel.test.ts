import { expectSerializeToMatchSnapshot } from '../tests/expectSerialize.js';
import { makePreviousKernelData, makePrivateKernelInputs, makePrivateKernelPublicInputs } from '../tests/factories.js';

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

  it(`serializes and prints private_kernel_public_inputs`, async () => {
    const kernelInputs = makePrivateKernelPublicInputs();
    await expectSerializeToMatchSnapshot(
      kernelInputs.toBuffer(),
      'abis__test_roundtrip_serialize_private_kernel_public_inputs',
    );
  });
});
