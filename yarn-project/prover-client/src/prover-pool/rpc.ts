import { type ProvingJobSource } from '@aztec/circuit-types';
import {
  BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  BaseRollupInputs,
  KernelCircuitPublicInputs,
  MergeRollupInputs,
  ParityPublicInputs,
  Proof,
  PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
  PublicKernelTailCircuitPrivateInputs,
  RootParityInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { ProvingError } from './proving-error.js';

export function createProvingJobSourceServer(queue: ProvingJobSource): JsonRpcServer {
  return new JsonRpcServer(
    queue,
    {
      BaseParityInputs,
      BaseOrMergeRollupPublicInputs,
      BaseRollupInputs,
      MergeRollupInputs,
      ParityPublicInputs,
      Proof,
      RootParityInputs,
      RootRollupInputs,
      RootRollupPublicInputs,
      PublicKernelCircuitPrivateInputs,
      PublicKernelCircuitPublicInputs,
      PublicKernelTailCircuitPrivateInputs,
      KernelCircuitPublicInputs,
      ProvingError,
    },
    {},
  );
}

export function createProvingJobSourceClient(
  url: string,
  namespace?: string,
  fetch = makeFetch([1, 2, 3], false),
): ProvingJobSource {
  return createJsonRpcClient(
    url,
    {
      BaseParityInputs,
      BaseOrMergeRollupPublicInputs,
      BaseRollupInputs,
      MergeRollupInputs,
      ParityPublicInputs,
      Proof,
      RootParityInputs,
      RootRollupInputs,
      RootRollupPublicInputs,
      PublicKernelCircuitPrivateInputs,
      PublicKernelCircuitPublicInputs,
      PublicKernelTailCircuitPrivateInputs,
      KernelCircuitPublicInputs,
      ProvingError,
    },
    {},
    false,
    namespace,
    fetch,
  ) as ProvingJobSource;
}
