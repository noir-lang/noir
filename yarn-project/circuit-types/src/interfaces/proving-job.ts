import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type Proof,
  type PublicKernelCircuitPublicInputs,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';

import type { PublicKernelNonTailRequest, PublicKernelTailRequest } from '../tx/processed_tx.js';

export type ProvingJob<T extends ProvingRequest> = {
  id: string;
  request: T;
};

export enum ProvingRequestType {
  PUBLIC_VM,

  PUBLIC_KERNEL_NON_TAIL,
  PUBLIC_KERNEL_TAIL,

  BASE_ROLLUP,
  MERGE_ROLLUP,
  ROOT_ROLLUP,

  BASE_PARITY,
  ROOT_PARITY,
}

export type ProvingRequest =
  | {
      type: ProvingRequestType.PUBLIC_VM;
      // prefer object over unknown so that we can run "in" checks, e.g. `'toBuffer' in request.inputs`
      inputs: object;
    }
  | {
      type: ProvingRequestType.PUBLIC_KERNEL_NON_TAIL;
      kernelType: PublicKernelNonTailRequest['type'];
      inputs: PublicKernelNonTailRequest['inputs'];
    }
  | {
      type: ProvingRequestType.PUBLIC_KERNEL_TAIL;
      kernelType: PublicKernelTailRequest['type'];
      inputs: PublicKernelTailRequest['inputs'];
    }
  | {
      type: ProvingRequestType.BASE_PARITY;
      inputs: BaseParityInputs;
    }
  | {
      type: ProvingRequestType.ROOT_PARITY;
      inputs: RootParityInputs;
    }
  | {
      type: ProvingRequestType.BASE_ROLLUP;
      inputs: BaseRollupInputs;
    }
  | {
      type: ProvingRequestType.MERGE_ROLLUP;
      inputs: MergeRollupInputs;
    }
  | {
      type: ProvingRequestType.ROOT_ROLLUP;
      inputs: RootRollupInputs;
    };

export type ProvingRequestPublicInputs = {
  [ProvingRequestType.PUBLIC_VM]: object;

  [ProvingRequestType.PUBLIC_KERNEL_NON_TAIL]: PublicKernelCircuitPublicInputs;
  [ProvingRequestType.PUBLIC_KERNEL_TAIL]: KernelCircuitPublicInputs;

  [ProvingRequestType.BASE_ROLLUP]: BaseOrMergeRollupPublicInputs;
  [ProvingRequestType.MERGE_ROLLUP]: BaseOrMergeRollupPublicInputs;
  [ProvingRequestType.ROOT_ROLLUP]: RootRollupPublicInputs;

  [ProvingRequestType.BASE_PARITY]: ParityPublicInputs;
  [ProvingRequestType.ROOT_PARITY]: ParityPublicInputs;
};

export type ProvingRequestResult<T extends ProvingRequestType> = [ProvingRequestPublicInputs[T], Proof];

export interface ProvingJobSource {
  getProvingJob(): Promise<ProvingJob<ProvingRequest> | null>;

  resolveProvingJob<T extends ProvingRequestType>(jobId: string, result: ProvingRequestResult<T>): Promise<void>;

  rejectProvingJob(jobId: string, reason: Error): Promise<void>;
}
