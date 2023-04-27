import { simulatePublicKernelCircuit, simulatePublicKernelCircuitNoPreviousKernel } from '../index.js';
import { makePublicKernelInputs, makePublicKernelInputsNoKernelInput } from '../tests/factories.js';

describe('kernel/public_kernel', () => {
  it.skip('simulates public kernel circuit with previous public kernel', async function () {
    const input = makePublicKernelInputs();
    input.previousKernel.publicInputs.isPrivateKernel = false;
    const result = await simulatePublicKernelCircuit(input);
    expect(result).toBeDefined();
  });

  it.skip('simulates public kernel circuit with previous private kernel', async function () {
    const input = makePublicKernelInputs();
    input.previousKernel.publicInputs.isPrivateKernel = true;
    const result = await simulatePublicKernelCircuit(input);
    expect(result).toBeDefined();
  });

  it('simulates public kernel circuit with no previous kernel', async function () {
    const input = makePublicKernelInputsNoKernelInput();
    const result = await simulatePublicKernelCircuitNoPreviousKernel(input);
    expect(result).toBeDefined();
  });
});
