import { ProvingRequestType } from '@aztec/circuit-types';
import {
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  VerificationKeyAsFields,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { makeBaseParityInputs, makeBaseRollupInputs, makeParityPublicInputs } from '@aztec/circuits.js/testing';

import { MemoryProvingQueue } from './memory-proving-queue.js';

describe('MemoryProvingQueue', () => {
  let queue: MemoryProvingQueue;

  beforeEach(() => {
    queue = new MemoryProvingQueue();
  });

  it('returns jobs in order', async () => {
    void queue.getBaseParityProof(makeBaseParityInputs());
    void queue.getBaseRollupProof(makeBaseRollupInputs());

    const job1 = await queue.getProvingJob();
    expect(job1?.request.type).toEqual(ProvingRequestType.BASE_PARITY);

    const job2 = await queue.getProvingJob();
    expect(job2?.request.type).toEqual(ProvingRequestType.BASE_ROLLUP);
  });

  it('returns undefined when no jobs are available', async () => {
    await expect(queue.getProvingJob({ timeoutSec: 0 })).resolves.toBeUndefined();
  });

  it('notifies of completion', async () => {
    const inputs = makeBaseParityInputs();
    const promise = queue.getBaseParityProof(inputs);

    const job = await queue.getProvingJob();
    expect(job?.request.inputs).toEqual(inputs);

    const publicInputs = makeParityPublicInputs();
    const proof = makeRecursiveProof<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH);
    const vk = VerificationKeyAsFields.makeFake();
    await queue.resolveProvingJob(job!.id, new RootParityInput(proof, vk, publicInputs));
    await expect(promise).resolves.toEqual(new RootParityInput(proof, vk, publicInputs));
  });

  it('retries failed jobs', async () => {
    const inputs = makeBaseParityInputs();
    void queue.getBaseParityProof(inputs);

    const job = await queue.getProvingJob();
    expect(job?.request.inputs).toEqual(inputs);

    const error = new Error('test error');

    await queue.rejectProvingJob(job!.id, error);
    await expect(queue.getProvingJob()).resolves.toEqual(job);
  });

  it('notifies errors', async () => {
    const promise = queue.getBaseParityProof(makeBaseParityInputs());

    const error = new Error('test error');
    await queue.rejectProvingJob((await queue.getProvingJob())!.id, error);
    await queue.rejectProvingJob((await queue.getProvingJob())!.id, error);
    await queue.rejectProvingJob((await queue.getProvingJob())!.id, error);

    await expect(promise).rejects.toEqual(error);
  });
});
