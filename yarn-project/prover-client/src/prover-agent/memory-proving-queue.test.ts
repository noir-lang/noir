import { ProvingRequestType } from '@aztec/circuit-types';
import {
  Fr,
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  VK_TREE_HEIGHT,
  VerificationKeyAsFields,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { makeBaseParityInputs, makeBaseRollupInputs, makeParityPublicInputs } from '@aztec/circuits.js/testing';
import { makeTuple } from '@aztec/foundation/array';
import { AbortError } from '@aztec/foundation/error';
import { sleep } from '@aztec/foundation/sleep';

import { MemoryProvingQueue } from './memory-proving-queue.js';

describe('MemoryProvingQueue', () => {
  let queue: MemoryProvingQueue;
  let jobTimeoutMs: number;
  let pollingIntervalMs: number;

  beforeEach(() => {
    jobTimeoutMs = 100;
    pollingIntervalMs = 10;
    queue = new MemoryProvingQueue(jobTimeoutMs, pollingIntervalMs);
    queue.start();
  });

  afterEach(async () => {
    await queue.stop();
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
    const vkPath = makeTuple(VK_TREE_HEIGHT, Fr.zero);
    await queue.resolveProvingJob(job!.id, new RootParityInput(proof, vk, vkPath, publicInputs));
    await expect(promise).resolves.toEqual(new RootParityInput(proof, vk, vkPath, publicInputs));
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

  it('reaps timed out jobs', async () => {
    const controller = new AbortController();
    const promise = queue.getBaseParityProof(makeBaseParityInputs(), controller.signal);
    const job = await queue.getProvingJob();

    expect(queue.isJobRunning(job!.id)).toBe(true);
    await sleep(jobTimeoutMs + 2 * pollingIntervalMs);
    expect(queue.isJobRunning(job!.id)).toBe(false);

    controller.abort();
    await expect(promise).rejects.toThrow(AbortError);
  });

  it('keeps jobs running while heartbeat is called', async () => {
    const promise = queue.getBaseParityProof(makeBaseParityInputs());
    const job = await queue.getProvingJob();

    expect(queue.isJobRunning(job!.id)).toBe(true);
    await sleep(pollingIntervalMs);
    expect(queue.isJobRunning(job!.id)).toBe(true);

    await queue.heartbeat(job!.id);
    expect(queue.isJobRunning(job!.id)).toBe(true);
    await sleep(pollingIntervalMs);
    expect(queue.isJobRunning(job!.id)).toBe(true);

    const output = new RootParityInput(
      makeRecursiveProof(RECURSIVE_PROOF_LENGTH),
      VerificationKeyAsFields.makeFake(),
      makeTuple(VK_TREE_HEIGHT, Fr.zero),
      makeParityPublicInputs(),
    );
    await queue.resolveProvingJob(job!.id, output);
    await expect(promise).resolves.toEqual(output);
  });
});
