import { type ServerCircuitProver } from '@aztec/circuit-types';
import { RECURSIVE_PROOF_LENGTH, type RootParityInput } from '@aztec/circuits.js';
import { makeBaseParityInputs, makeRootParityInput } from '@aztec/circuits.js/testing';
import { AbortError } from '@aztec/foundation/error';
import { promiseWithResolvers } from '@aztec/foundation/promise';
import { sleep } from '@aztec/foundation/sleep';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { type MockProxy, mock } from 'jest-mock-extended';

import { MemoryProvingQueue } from './memory-proving-queue.js';
import { ProverAgent } from './prover-agent.js';

describe('Prover agent <-> queue integration', () => {
  let queue: MemoryProvingQueue;
  let agent: ProverAgent;
  let prover: MockProxy<ServerCircuitProver>;
  let agentPollInterval: number;
  let queuePollInterval: number;
  let queueJobTimeout: number;

  beforeEach(() => {
    prover = mock<ServerCircuitProver>();

    queueJobTimeout = 100;
    queuePollInterval = 10;
    queue = new MemoryProvingQueue(new NoopTelemetryClient(), queueJobTimeout, queuePollInterval);

    agentPollInterval = 10;
    agent = new ProverAgent(prover, 1, agentPollInterval);

    queue.start();
    agent.start(queue);
  });

  afterEach(async () => {
    await agent.stop();
    await queue.stop();
  });

  it('picks up jobs from the queue', async () => {
    const { promise, resolve } = promiseWithResolvers<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>();
    const output = makeRootParityInput(RECURSIVE_PROOF_LENGTH, 1);
    prover.getBaseParityProof.mockResolvedValueOnce(promise);
    const proofPromise = queue.getBaseParityProof(makeBaseParityInputs());

    await sleep(agentPollInterval);
    resolve(output);
    await expect(proofPromise).resolves.toEqual(output);
  });

  it('keeps job alive', async () => {
    const { promise, resolve } = promiseWithResolvers<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>();
    const output = makeRootParityInput(RECURSIVE_PROOF_LENGTH, 1);
    prover.getBaseParityProof.mockResolvedValueOnce(promise);
    const proofPromise = queue.getBaseParityProof(makeBaseParityInputs());

    await sleep(2 * queueJobTimeout);
    resolve(output);
    await expect(proofPromise).resolves.toEqual(output);
  });

  it('reports cancellations', async () => {
    const { promise, resolve } = promiseWithResolvers<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>();
    const output = makeRootParityInput(RECURSIVE_PROOF_LENGTH, 1);
    prover.getBaseParityProof.mockResolvedValueOnce(promise);
    const controller = new AbortController();
    const proofPromise = queue.getBaseParityProof(makeBaseParityInputs(), controller.signal);
    await sleep(agentPollInterval);
    controller.abort();
    resolve(output);
    await expect(proofPromise).rejects.toThrow(AbortError);
  });

  it('re-queues timed out jobs', async () => {
    const firstRun = promiseWithResolvers<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>();
    const output = makeRootParityInput(RECURSIVE_PROOF_LENGTH, 1);
    prover.getBaseParityProof.mockResolvedValueOnce(firstRun.promise);
    const proofPromise = queue.getBaseParityProof(makeBaseParityInputs());

    // stop the agent to simulate a machine going down
    await agent.stop();

    // give the queue a chance to figure out the node is timed out and re-queue the job
    await sleep(queueJobTimeout);
    // reset the mock
    const secondRun = promiseWithResolvers<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>();
    prover.getBaseParityProof.mockResolvedValueOnce(secondRun.promise);
    const newAgent = new ProverAgent(prover, 1, agentPollInterval);
    newAgent.start(queue);
    // test that the job is re-queued and kept alive by the new agent
    await sleep(queueJobTimeout * 2);
    secondRun.resolve(output);
    await expect(proofPromise).resolves.toEqual(output);

    firstRun.reject(new Error('stop this promise otherwise it hangs jest'));

    await newAgent.stop();
  });
});
