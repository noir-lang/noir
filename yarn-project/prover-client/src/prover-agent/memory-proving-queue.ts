import {
  type ProofAndVerificationKey,
  type ProvingJob,
  type ProvingJobSource,
  type ProvingRequest,
  type ProvingRequestResult,
  ProvingRequestType,
  type PublicInputsAndRecursiveProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  type ServerCircuitProver,
} from '@aztec/circuit-types';
import type {
  AvmCircuitInputs,
  BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  BaseRollupInputs,
  KernelCircuitPublicInputs,
  MergeRollupInputs,
  NESTED_RECURSIVE_PROOF_LENGTH,
  PrivateKernelEmptyInputData,
  PublicKernelCircuitPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  RecursiveProof,
  RootParityInput,
  RootParityInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  TubeInputs,
  VerificationKeyData,
} from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { AbortError, TimeoutError } from '@aztec/foundation/error';
import { MemoryFifo } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { type PromiseWithResolvers, RunningPromise, promiseWithResolvers } from '@aztec/foundation/promise';

type ProvingJobWithResolvers<T extends ProvingRequest = ProvingRequest> = {
  id: string;
  request: T;
  signal?: AbortSignal;
  attempts: number;
  heartbeat: number;
} & PromiseWithResolvers<ProvingRequestResult<T['type']>>;

const MAX_RETRIES = 3;

const defaultIdGenerator = () => randomBytes(4).toString('hex');
const defaultTimeSource = () => Date.now();

/**
 * A helper class that sits in between services that need proofs created and agents that can create them.
 * The queue accumulates jobs and provides them to agents in FIFO order.
 */
export class MemoryProvingQueue implements ServerCircuitProver, ProvingJobSource {
  private log = createDebugLogger('aztec:prover-client:prover-pool:queue');
  private queue = new MemoryFifo<ProvingJobWithResolvers>();
  private jobsInProgress = new Map<string, ProvingJobWithResolvers>();

  private runningPromise: RunningPromise;

  constructor(
    /** Timeout the job if an agent doesn't report back in this time */
    private jobTimeoutMs = 60 * 1000,
    /** How often to check for timed out jobs */
    pollingIntervalMs = 1000,
    private generateId = defaultIdGenerator,
    private timeSource = defaultTimeSource,
  ) {
    this.runningPromise = new RunningPromise(this.poll, pollingIntervalMs);
  }

  public start() {
    if (this.runningPromise.isRunning()) {
      this.log.warn('Proving queue is already running');
      return;
    }

    this.runningPromise.start();
    this.log.info('Proving queue started');
  }

  public async stop() {
    if (!this.runningPromise.isRunning()) {
      this.log.warn('Proving queue is already stopped');
      return;
    }

    await this.runningPromise.stop();
    this.log.info('Proving queue stopped');
  }

  public async getProvingJob({ timeoutSec = 1 } = {}): Promise<ProvingJob<ProvingRequest> | undefined> {
    if (!this.runningPromise.isRunning()) {
      throw new Error('Proving queue is not running. Start the queue before getting jobs.');
    }

    try {
      const job = await this.queue.get(timeoutSec);
      if (!job) {
        return undefined;
      }

      if (job.signal?.aborted) {
        return undefined;
      }

      job.heartbeat = this.timeSource();
      this.jobsInProgress.set(job.id, job);
      return {
        id: job.id,
        request: job.request,
      };
    } catch (err) {
      if (err instanceof TimeoutError) {
        return undefined;
      }

      throw err;
    }
  }

  resolveProvingJob<T extends ProvingRequestType>(jobId: string, result: ProvingRequestResult<T>): Promise<void> {
    if (!this.runningPromise.isRunning()) {
      throw new Error('Proving queue is not running.');
    }

    const job = this.jobsInProgress.get(jobId);
    if (!job) {
      this.log.warn(`Job id=${jobId} not found. Can't resolve`);
      return Promise.resolve();
    }

    this.jobsInProgress.delete(jobId);
    if (!job.signal?.aborted) {
      job.resolve(result);
    }

    return Promise.resolve();
  }

  rejectProvingJob(jobId: string, err: any): Promise<void> {
    if (!this.runningPromise.isRunning()) {
      throw new Error('Proving queue is not running.');
    }

    const job = this.jobsInProgress.get(jobId);
    if (!job) {
      this.log.warn(`Job id=${jobId} not found. Can't reject`);
      return Promise.resolve();
    }

    this.jobsInProgress.delete(jobId);

    if (job.signal?.aborted) {
      return Promise.resolve();
    }

    if (job.attempts < MAX_RETRIES) {
      job.attempts++;
      this.log.warn(
        `Job id=${job.id} type=${ProvingRequestType[job.request.type]} failed with error: ${err}. Retry ${
          job.attempts
        }/${MAX_RETRIES}`,
      );
      this.queue.put(job);
    } else {
      this.log.error(`Job id=${job.id} type=${ProvingRequestType[job.request.type]} failed with error: ${err}`);
      job.reject(err);
    }
    return Promise.resolve();
  }

  public heartbeat(jobId: string): Promise<void> {
    if (!this.runningPromise.isRunning()) {
      throw new Error('Proving queue is not running.');
    }

    const job = this.jobsInProgress.get(jobId);
    if (job) {
      job.heartbeat = this.timeSource();
    }

    return Promise.resolve();
  }

  public isJobRunning(jobId: string): boolean {
    return this.jobsInProgress.has(jobId);
  }

  private poll = () => {
    const now = this.timeSource();

    for (const job of this.jobsInProgress.values()) {
      if (job.signal?.aborted) {
        this.jobsInProgress.delete(job.id);
        continue;
      }

      if (job.heartbeat + this.jobTimeoutMs < now) {
        this.log.warn(`Job ${job.id} type=${ProvingRequestType[job.request.type]} has timed out`);

        this.jobsInProgress.delete(job.id);
        job.heartbeat = 0;
        this.queue.put(job);
      }
    }
  };

  private enqueue<T extends ProvingRequest>(
    request: T,
    signal?: AbortSignal,
  ): Promise<ProvingRequestResult<T['type']>> {
    if (!this.runningPromise.isRunning()) {
      return Promise.reject(new Error('Proving queue is not running.'));
    }

    const { promise, resolve, reject } = promiseWithResolvers<ProvingRequestResult<T['type']>>();
    const item: ProvingJobWithResolvers<T> = {
      id: this.generateId(),
      request,
      signal,
      promise,
      resolve,
      reject,
      attempts: 1,
      heartbeat: 0,
    };

    if (signal) {
      signal.addEventListener('abort', () => reject(new AbortError('Operation has been aborted')));
    }

    this.log.debug(
      `Adding id=${item.id} type=${ProvingRequestType[request.type]} proving job to queue depth=${this.queue.length()}`,
    );
    // TODO (alexg) remove the `any`
    if (!this.queue.put(item as any)) {
      throw new Error();
    }

    return promise;
  }

  getEmptyPrivateKernelProof(
    inputs: PrivateKernelEmptyInputData,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<KernelCircuitPublicInputs>> {
    return this.enqueue({ type: ProvingRequestType.PRIVATE_KERNEL_EMPTY, inputs }, signal);
  }

  getTubeProof(
    inputs: TubeInputs,
    signal?: AbortSignal | undefined,
  ): Promise<{ tubeVK: VerificationKeyData; tubeProof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH> }> {
    return this.enqueue({ type: ProvingRequestType.TUBE_PROOF, inputs }, signal);
  }

  getEmptyTubeProof(
    inputs: PrivateKernelEmptyInputData,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<KernelCircuitPublicInputs>> {
    return this.enqueue({ type: ProvingRequestType.PRIVATE_KERNEL_EMPTY, inputs }, signal);
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getBaseParityProof(
    inputs: BaseParityInputs,
    signal?: AbortSignal,
  ): Promise<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>> {
    return this.enqueue(
      {
        type: ProvingRequestType.BASE_PARITY,
        inputs,
      },
      signal,
    );
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootParityProof(
    inputs: RootParityInputs,
    signal?: AbortSignal,
  ): Promise<RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>> {
    return this.enqueue(
      {
        type: ProvingRequestType.ROOT_PARITY,
        inputs,
      },
      signal,
    );
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getBaseRollupProof(
    baseRollupInput: BaseRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<BaseOrMergeRollupPublicInputs>> {
    return this.enqueue(
      {
        type: ProvingRequestType.BASE_ROLLUP,
        inputs: baseRollupInput,
      },
      signal,
    );
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getMergeRollupProof(
    input: MergeRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<BaseOrMergeRollupPublicInputs>> {
    return this.enqueue(
      {
        type: ProvingRequestType.MERGE_ROLLUP,
        inputs: input,
      },
      signal,
    );
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootRollupProof(
    input: RootRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<RootRollupPublicInputs>> {
    return this.enqueue(
      {
        type: ProvingRequestType.ROOT_ROLLUP,
        inputs: input,
      },
      signal,
    );
  }

  /**
   * Create a public kernel proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<PublicKernelCircuitPublicInputs>> {
    return this.enqueue(
      {
        type: ProvingRequestType.PUBLIC_KERNEL_NON_TAIL,
        kernelType: kernelRequest.type,
        inputs: kernelRequest.inputs,
      },
      signal,
    );
  }

  /**
   * Create a public kernel tail proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicTailProof(
    kernelRequest: PublicKernelTailRequest,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndRecursiveProof<KernelCircuitPublicInputs>> {
    return this.enqueue(
      {
        type: ProvingRequestType.PUBLIC_KERNEL_TAIL,
        kernelType: kernelRequest.type,
        inputs: kernelRequest.inputs,
      },
      signal,
    );
  }

  /**
   * Creates an AVM proof.
   */
  getAvmProof(inputs: AvmCircuitInputs, signal?: AbortSignal | undefined): Promise<ProofAndVerificationKey> {
    return this.enqueue(
      {
        type: ProvingRequestType.PUBLIC_VM,
        inputs,
      },
      signal,
    );
  }

  /**
   * Verifies a circuit proof
   */
  verifyProof(): Promise<void> {
    return Promise.reject('not implemented');
  }
}
