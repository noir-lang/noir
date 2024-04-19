import type { ProvingRequest, ProvingRequestResult, ProvingRequestType } from './proving-request.js';

export type GetJobOptions = {
  timeoutSec?: number;
};

export type ProvingJob<T extends ProvingRequest> = {
  id: string;
  request: T;
};

export interface ProvingRequestProducer {
  prove<T extends ProvingRequest>(request: T): Promise<ProvingRequestResult<T['type']>>;
  cancelAll(): void;
}

export interface ProvingQueueConsumer {
  getProvingJob(options?: GetJobOptions): Promise<ProvingJob<ProvingRequest> | null>;
  resolveProvingJob<T extends ProvingRequestType>(jobId: string, result: ProvingRequestResult<T>): Promise<void>;
  rejectProvingJob(jobId: string, reason: Error): Promise<void>;
}

export interface ProvingQueue extends ProvingQueueConsumer, ProvingRequestProducer {}
