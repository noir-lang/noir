import { type ProvingJobSource } from '@aztec/circuit-types';
import {
  AvmCircuitInputs,
  BaseOrMergeRollupPublicInputs,
  BaseParityInputs,
  BaseRollupInputs,
  Fr,
  Header,
  KernelCircuitPublicInputs,
  MergeRollupInputs,
  ParityPublicInputs,
  Proof,
  PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
  PublicKernelTailCircuitPrivateInputs,
  RecursiveProof,
  RootParityInput,
  RootParityInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  VerificationKeyData,
} from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { ProvingError } from './proving-error.js';

export function createProvingJobSourceServer(queue: ProvingJobSource): JsonRpcServer {
  return new JsonRpcServer(
    queue,
    {
      Header,
      Fr,
      AvmCircuitInputs,
      BaseParityInputs,
      BaseOrMergeRollupPublicInputs,
      BaseRollupInputs,
      MergeRollupInputs,
      ParityPublicInputs,
      Proof,
      RootParityInput,
      RootParityInputs,
      RootRollupInputs,
      RootRollupPublicInputs,
      PublicKernelCircuitPrivateInputs,
      PublicKernelCircuitPublicInputs,
      PublicKernelTailCircuitPrivateInputs,
      KernelCircuitPublicInputs,
      ProvingError,
      RecursiveProof,
      VerificationKeyData,
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
      Header,
      Fr,
      AvmCircuitInputs,
      BaseParityInputs,
      BaseOrMergeRollupPublicInputs,
      BaseRollupInputs,
      MergeRollupInputs,
      ParityPublicInputs,
      Proof,
      RootParityInput,
      RootParityInputs,
      RootRollupInputs,
      RootRollupPublicInputs,
      PublicKernelCircuitPrivateInputs,
      PublicKernelCircuitPublicInputs,
      PublicKernelTailCircuitPrivateInputs,
      KernelCircuitPublicInputs,
      ProvingError,
      RecursiveProof,
      VerificationKeyData,
    },
    {},
    false,
    namespace,
    fetch,
  ) as ProvingJobSource;
}
