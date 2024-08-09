import {
  type ProvingJob,
  type ProvingJobSource,
  type ProvingRequest,
  type ProvingRequestResult,
  ProvingRequestType,
  type ServerCircuitProver,
} from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { elapsed } from '@aztec/foundation/timer';

import { ProvingError } from './proving-error.js';

const PRINT_THRESHOLD_NS = 6e10; // 60 seconds

/**
 * A helper class that encapsulates a circuit prover and connects it to a job source.
 */
export class ProverAgent {
  private inFlightPromises = new Map<
    string,
    {
      id: string;
      type: ProvingRequestType;
      promise: Promise<any>;
    }
  >();
  private runningPromise?: RunningPromise;

  constructor(
    /** The prover implementation to defer jobs to */
    private circuitProver: ServerCircuitProver,
    /** How many proving jobs this agent can handle in parallel */
    private maxConcurrency = 1,
    /** How long to wait between jobs */
    private pollIntervalMs = 100,
    private log = createDebugLogger('aztec:prover-client:prover-agent'),
  ) {}

  setMaxConcurrency(maxConcurrency: number): void {
    if (maxConcurrency < 1) {
      throw new Error('Concurrency must be at least 1');
    }
    this.maxConcurrency = maxConcurrency;
  }

  setCircuitProver(circuitProver: ServerCircuitProver): void {
    this.circuitProver = circuitProver;
  }

  isRunning() {
    return this.runningPromise?.isRunning() ?? false;
  }

  start(jobSource: ProvingJobSource): void {
    if (this.runningPromise) {
      throw new Error('Agent is already running');
    }

    let lastPrint = process.hrtime.bigint();

    this.runningPromise = new RunningPromise(async () => {
      for (const jobId of this.inFlightPromises.keys()) {
        await jobSource.heartbeat(jobId);
      }

      const now = process.hrtime.bigint();

      if (now - lastPrint >= PRINT_THRESHOLD_NS) {
        // only log if we're actually doing work
        if (this.inFlightPromises.size > 0) {
          const jobs = Array.from(this.inFlightPromises.values())
            .map(job => `id=${job.id},type=${ProvingRequestType[job.type]}`)
            .join(' ');
          this.log.info(`Agent is running with ${this.inFlightPromises.size} in-flight jobs: ${jobs}`);
        }
        lastPrint = now;
      }

      while (this.inFlightPromises.size < this.maxConcurrency) {
        try {
          const job = await jobSource.getProvingJob();
          if (!job) {
            // job source is fully drained, sleep for a bit and try again
            return;
          }

          try {
            const promise = this.work(jobSource, job).finally(() => this.inFlightPromises.delete(job.id));
            this.inFlightPromises.set(job.id, {
              id: job.id,
              type: job.request.type,
              promise,
            });
          } catch (err) {
            this.log.warn(
              `Error processing job! type=${ProvingRequestType[job.request.type]}: ${err}. ${(err as Error).stack}`,
            );
          }
        } catch (err) {
          this.log.error(`Error fetching job`, err);
        }
      }
    }, this.pollIntervalMs);

    this.runningPromise.start();
    this.log.info(`Agent started with concurrency=${this.maxConcurrency}`);
  }

  async stop(): Promise<void> {
    if (!this.runningPromise?.isRunning()) {
      return;
    }

    await this.runningPromise.stop();
    this.runningPromise = undefined;

    this.log.info('Agent stopped');
  }

  private async work(jobSource: ProvingJobSource, job: ProvingJob<ProvingRequest>): Promise<void> {
    try {
      this.log.debug(`Picked up proving job id=${job.id} type=${ProvingRequestType[job.request.type]}`);
      const [time, result] = await elapsed(this.getProof(job.request));
      if (this.isRunning()) {
        this.log.verbose(
          `Processed proving job id=${job.id} type=${ProvingRequestType[job.request.type]} duration=${time}ms`,
        );
        await jobSource.resolveProvingJob(job.id, result);
      } else {
        this.log.verbose(
          `Dropping proving job id=${job.id} type=${
            ProvingRequestType[job.request.type]
          } duration=${time}ms: agent stopped`,
        );
      }
    } catch (err) {
      if (this.isRunning()) {
        this.log.error(
          `Error processing proving job id=${job.id} type=${ProvingRequestType[job.request.type]}: ${
            (err as any).stack || err
          }`,
          err,
        );
        await jobSource.rejectProvingJob(job.id, new ProvingError((err as any)?.message ?? String(err)));
      } else {
        this.log.verbose(
          `Dropping proving job id=${job.id} type=${ProvingRequestType[job.request.type]}: agent stopped: ${
            (err as any).stack || err
          }`,
        );
      }
    }
  }

  private getProof(request: ProvingRequest): Promise<ProvingRequestResult<typeof type>> {
    const { type, inputs } = request;
    switch (type) {
      case ProvingRequestType.PUBLIC_VM: {
        return this.circuitProver.getAvmProof(inputs);
      }

      case ProvingRequestType.PUBLIC_KERNEL_NON_TAIL: {
        return this.circuitProver.getPublicKernelProof({
          type: request.kernelType,
          inputs,
        });
      }

      case ProvingRequestType.PUBLIC_KERNEL_TAIL: {
        return this.circuitProver.getPublicTailProof({
          type: request.kernelType,
          inputs,
        });
      }

      case ProvingRequestType.BASE_ROLLUP: {
        return this.circuitProver.getBaseRollupProof(inputs);
      }

      case ProvingRequestType.MERGE_ROLLUP: {
        return this.circuitProver.getMergeRollupProof(inputs);
      }

      case ProvingRequestType.ROOT_ROLLUP: {
        return this.circuitProver.getRootRollupProof(inputs);
      }

      case ProvingRequestType.BASE_PARITY: {
        return this.circuitProver.getBaseParityProof(inputs);
      }

      case ProvingRequestType.ROOT_PARITY: {
        return this.circuitProver.getRootParityProof(inputs);
      }

      case ProvingRequestType.PRIVATE_KERNEL_EMPTY: {
        return this.circuitProver.getEmptyPrivateKernelProof(inputs);
      }

      case ProvingRequestType.TUBE_PROOF: {
        return this.circuitProver.getTubeProof(inputs);
      }

      default: {
        const _exhaustive: never = type;
        return Promise.reject(new Error(`Invalid proof request type: ${type}`));
      }
    }
  }
}
