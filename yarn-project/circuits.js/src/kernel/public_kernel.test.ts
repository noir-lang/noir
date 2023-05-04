import { simulatePublicKernelCircuit, simulatePublicKernelCircuitNoPreviousKernel } from '../index.js';
import { makePublicKernelInputsNoKernelInput, makePublicKernelInputsWithEmptyOutput } from '../tests/factories.js';

describe('kernel/public_kernel', () => {
  it('simulates public kernel circuit with previous public kernel', async function () {
    const input = makePublicKernelInputsWithEmptyOutput();
    // Fix validity
    input.publicCallData.callStackItem.functionData.isConstructor = false;
    input.publicCallData.callStackItem.functionData.isPrivate = false;
    input.previousKernel.publicInputs.isPrivateKernel = false;
    const result = await simulatePublicKernelCircuit(input);
    expect(result).toBeDefined();
  });

  it('simulates public kernel circuit with previous private kernel', async function () {
    const input = makePublicKernelInputsWithEmptyOutput();
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
