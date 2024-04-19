import { makeBaseParityInputs, makeParityPublicInputs, makeProof } from '@aztec/circuits.js/testing';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type CircuitProver } from '../prover/interface.js';
import { CircuitProverAgent } from './circuit-prover-agent.js';
import { MemoryProvingQueue } from './memory-proving-queue.js';
import { type ProvingAgent } from './prover-agent.js';
import { type ProvingQueue } from './proving-queue.js';
import { ProvingRequestType } from './proving-request.js';

describe('LocalProvingAgent', () => {
  let queue: ProvingQueue;
  let agent: ProvingAgent;
  let prover: MockProxy<CircuitProver>;

  beforeEach(() => {
    prover = mock<CircuitProver>();
    queue = new MemoryProvingQueue();
    agent = new CircuitProverAgent(prover);
  });

  beforeEach(() => {
    agent.start(queue);
  });

  afterEach(async () => {
    await agent.stop();
  });

  it('takes jobs from the queue', async () => {
    const publicInputs = makeParityPublicInputs();
    const proof = makeProof();
    prover.getBaseParityProof.mockResolvedValue([publicInputs, proof]);

    const inputs = makeBaseParityInputs();
    const promise = queue.prove({
      type: ProvingRequestType.BASE_PARITY,
      inputs,
    });

    await expect(promise).resolves.toEqual([publicInputs, proof]);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs);
  });

  it('reports errors', async () => {
    const error = new Error('test error');
    prover.getBaseParityProof.mockRejectedValue(error);

    const inputs = makeBaseParityInputs();
    const promise = queue.prove({
      type: ProvingRequestType.BASE_PARITY,
      inputs,
    });

    await expect(promise).rejects.toEqual(error);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs);
  });

  it('continues to process jobs', async () => {
    const publicInputs = makeParityPublicInputs();
    const proof = makeProof();
    prover.getBaseParityProof.mockResolvedValue([publicInputs, proof]);

    const inputs = makeBaseParityInputs();
    const promise1 = queue.prove({
      type: ProvingRequestType.BASE_PARITY,
      inputs,
    });

    await expect(promise1).resolves.toEqual([publicInputs, proof]);

    const inputs2 = makeBaseParityInputs();
    const promise2 = queue.prove({
      type: ProvingRequestType.BASE_PARITY,
      inputs: inputs2,
    });

    await expect(promise2).resolves.toEqual([publicInputs, proof]);

    expect(prover.getBaseParityProof).toHaveBeenCalledTimes(2);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs2);
  });
});
