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
import { TimeoutError } from '@aztec/foundation/error';
import { MemoryFifo } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { type PromiseWithResolvers, promiseWithResolvers } from '@aztec/foundation/promise';

import { type CircuitProver } from '../prover/interface.js';

type ProvingJobWithResolvers<T extends ProvingRequest = ProvingRequest> = {
  id: string;
  request: T;
} & PromiseWithResolvers<ProvingRequestResult<T['type']>>;

export class MemoryProvingQueue implements CircuitProver, ProvingJobSource {
  private jobId = 0;
  private log = createDebugLogger('aztec:prover-client:prover-pool:queue');
  private queue = new MemoryFifo<ProvingJobWithResolvers>();
  private jobsInProgress = new Map<string, ProvingJobWithResolvers>();

  async getProvingJob({ timeoutSec = 1 } = {}): Promise<ProvingJob<ProvingRequest> | null> {
    try {
      const job = await this.queue.get(timeoutSec);
      if (!job) {
        return null;
      }

      this.jobsInProgress.set(job.id, job);
      return {
        id: job.id,
        request: job.request,
      };
    } catch (err) {
      if (err instanceof TimeoutError) {
        return null;
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
    job.resolve(result);
    return Promise.resolve();
  }

  rejectProvingJob(jobId: string, err: any): Promise<void> {
    const job = this.jobsInProgress.get(jobId);
    if (!job) {
      return Promise.reject(new Error('Job not found'));
    }

    this.jobsInProgress.delete(jobId);
    job.reject(err);
    return Promise.resolve();
  }

  private enqueue<T extends ProvingRequest>(request: T): Promise<ProvingRequestResult<T['type']>> {
    const { promise, resolve, reject } = promiseWithResolvers<ProvingRequestResult<T['type']>>();
    const item: ProvingJobWithResolvers<T> = {
      id: String(this.jobId++),
      request,
      promise,
      resolve,
      reject,
    };

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
  getBaseParityProof(inputs: BaseParityInputs): Promise<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>> {
    return this.enqueue({
      type: ProvingRequestType.BASE_PARITY,
      inputs,
    });
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootParityProof(inputs: RootParityInputs): Promise<RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>> {
    return this.enqueue({
      type: ProvingRequestType.ROOT_PARITY,
      inputs,
    });
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getBaseRollupProof(input: BaseRollupInputs): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    return this.enqueue({
      type: ProvingRequestType.BASE_ROLLUP,
      inputs: input,
    });
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getMergeRollupProof(input: MergeRollupInputs): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    return this.enqueue({
      type: ProvingRequestType.MERGE_ROLLUP,
      inputs: input,
    });
  }

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootRollupProof(input: RootRollupInputs): Promise<PublicInputsAndProof<RootRollupPublicInputs>> {
    return this.enqueue({
      type: ProvingRequestType.ROOT_ROLLUP,
      inputs: input,
    });
  }

  /**
   * Create a public kernel proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
  ): Promise<PublicInputsAndProof<PublicKernelCircuitPublicInputs>> {
    return this.enqueue({
      type: ProvingRequestType.PUBLIC_KERNEL_NON_TAIL,
      kernelType: kernelRequest.type,
      inputs: kernelRequest.inputs,
    });
  }

  /**
   * Create a public kernel tail proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicTailProof(kernelRequest: PublicKernelTailRequest): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>> {
    return this.enqueue({
      type: ProvingRequestType.PUBLIC_KERNEL_TAIL,
      kernelType: kernelRequest.type,
      inputs: kernelRequest.inputs,
    });
  }

  /**
   * Verifies a circuit proof
   */
  verifyProof(): Promise<void> {
    // no-op
    return Promise.resolve();
  }
}
