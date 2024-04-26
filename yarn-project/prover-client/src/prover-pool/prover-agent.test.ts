import {
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  VerificationKeyAsFields,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { makeBaseParityInputs, makeParityPublicInputs } from '@aztec/circuits.js/testing';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type CircuitProver } from '../prover/interface.js';
import { MemoryProvingQueue } from './memory-proving-queue.js';
import { ProverAgent } from './prover-agent.js';

describe('ProverAgent', () => {
  let queue: MemoryProvingQueue;
  let agent: ProverAgent;
  let prover: MockProxy<CircuitProver>;

  beforeEach(() => {
    prover = mock<CircuitProver>();
    queue = new MemoryProvingQueue();
    agent = new ProverAgent(prover);
  });

  beforeEach(() => {
    agent.start(queue);
  });

  afterEach(async () => {
    await agent.stop();
  });

  it('takes jobs from the queue', async () => {
    const publicInputs = makeParityPublicInputs();
    const proof = makeRecursiveProof<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH);
    const vk = VerificationKeyAsFields.makeFake();
    prover.getBaseParityProof.mockResolvedValue(
      new RootParityInput<typeof RECURSIVE_PROOF_LENGTH>(proof, vk, publicInputs),
    );

    const inputs = makeBaseParityInputs();

    const promise = queue.getBaseParityProof(inputs);
    await expect(promise).resolves.toEqual(new RootParityInput<typeof RECURSIVE_PROOF_LENGTH>(proof, vk, publicInputs));

    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs);
  });

  it('reports errors', async () => {
    const error = new Error('test error');
    prover.getBaseParityProof.mockRejectedValue(error);

    const inputs = makeBaseParityInputs();
    const promise = queue.getBaseParityProof(inputs);

    await expect(promise).rejects.toEqual(error);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs);
  });

  it('continues to process jobs', async () => {
    const publicInputs = makeParityPublicInputs();
    const proof = makeRecursiveProof<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH);
    const vk = VerificationKeyAsFields.makeFake();
    prover.getBaseParityProof.mockResolvedValue(
      new RootParityInput<typeof RECURSIVE_PROOF_LENGTH>(proof, vk, publicInputs),
    );

    const inputs = makeBaseParityInputs();
    const promise1 = queue.getBaseParityProof(inputs);

    await expect(promise1).resolves.toEqual(
      new RootParityInput<typeof RECURSIVE_PROOF_LENGTH>(proof, vk, publicInputs),
    );

    const inputs2 = makeBaseParityInputs();
    const promise2 = queue.getBaseParityProof(inputs2);

    await expect(promise2).resolves.toEqual(
      new RootParityInput<typeof RECURSIVE_PROOF_LENGTH>(proof, vk, publicInputs),
    );

    expect(prover.getBaseParityProof).toHaveBeenCalledTimes(2);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs);
    expect(prover.getBaseParityProof).toHaveBeenCalledWith(inputs2);
  });
});
