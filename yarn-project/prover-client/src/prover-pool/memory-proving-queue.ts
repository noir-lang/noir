import {
  type ProvingJob,
  type ProvingJobSource,
  type ProvingRequest,
  type ProvingRequestResult,
  ProvingRequestType,
  type PublicInputsAndProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
} from '@aztec/circuit-types';
import type {
  BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  BaseRollupInputs,
  KernelCircuitPublicInputs,
  MergeRollupInputs,
  NESTED_RECURSIVE_PROOF_LENGTH,
  PublicKernelCircuitPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  RootParityInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { AbortedError, TimeoutError } from '@aztec/foundation/error';
import { MemoryFifo } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { type PromiseWithResolvers, promiseWithResolvers } from '@aztec/foundation/promise';

import { type CircuitProver } from '../prover/interface.js';

type ProvingJobWithResolvers<T extends ProvingRequest = ProvingRequest> = {
  id: string;
  request: T;
  signal?: AbortSignal;
} & PromiseWithResolvers<ProvingRequestResult<T['type']>>;

export class MemoryProvingQueue implements CircuitProver, ProvingJobSource {
  private jobId = 0;
  private log = createDebugLogger('aztec:prover-client:prover-pool:queue');
  private queue = new MemoryFifo<ProvingJobWithResolvers>();
  private jobsInProgress = new Map<string, ProvingJobWithResolvers>();

  async getProvingJob({ timeoutSec = 1 } = {}): Promise<ProvingJob<ProvingRequest> | undefined> {
    try {
      const job = await this.queue.get(timeoutSec);
      if (!job) {
        return undefined;
      }

      if (job.signal?.aborted) {
        this.log.debug(`Job ${job.id} type=${job.request.type} has been aborted`);
        return undefined;
      }

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
    const job = this.jobsInProgress.get(jobId);
    if (!job) {
      return Promise.reject(new Error('Job not found'));
    }

    this.jobsInProgress.delete(jobId);

    if (job.signal?.aborted) {
      return Promise.resolve();
    }

    job.resolve(result);
    return Promise.resolve();
  }

  rejectProvingJob(jobId: string, err: any): Promise<void> {
    const job = this.jobsInProgress.get(jobId);
    if (!job) {
      return Promise.reject(new Error('Job not found'));
    }

    this.jobsInProgress.delete(jobId);

    if (job.signal?.aborted) {
      return Promise.resolve();
    }

    job.reject(err);
    return Promise.resolve();
  }

  private enqueue<T extends ProvingRequest>(
    request: T,
    signal?: AbortSignal,
  ): Promise<ProvingRequestResult<T['type']>> {
    const { promise, resolve, reject } = promiseWithResolvers<ProvingRequestResult<T['type']>>();
    const item: ProvingJobWithResolvers<T> = {
      id: String(this.jobId++),
      request,
      signal,
      promise,
      resolve,
      reject,
    };

    if (signal) {
      signal.addEventListener('abort', () => reject(new AbortedError('Operation has been aborted')));
    }

    this.log.info(`Adding ${ProvingRequestType[request.type]} proving job to queue`);
    // TODO (alexg) remove the `any`
    if (!this.queue.put(item as any)) {
      throw new Error();
    }

    return promise;
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
    input: BaseRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    return this.enqueue(
      {
        type: ProvingRequestType.BASE_ROLLUP,
        inputs: input,
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
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
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
  ): Promise<PublicInputsAndProof<RootRollupPublicInputs>> {
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
  ): Promise<PublicInputsAndProof<PublicKernelCircuitPublicInputs>> {
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
  ): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>> {
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
   * Verifies a circuit proof
   */
  verifyProof(): Promise<void> {
    return Promise.reject('not implemented');
  }
}
