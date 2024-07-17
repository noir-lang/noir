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
  TubeInputs,
  VerificationKeyData,
} from '@aztec/circuits.js';
import { createJsonRpcClient, makeFetch } from '@aztec/foundation/json-rpc/client';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';

import { ProvingError } from './proving-error.js';

export function createProvingJobSourceServer(queue: ProvingJobSource): JsonRpcServer {
  return new JsonRpcServer(
    queue,
    {
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
      ProvingError,
      PublicKernelCircuitPrivateInputs,
      PublicKernelCircuitPublicInputs,
      PublicKernelTailCircuitPrivateInputs,
      RecursiveProof,
      RootParityInput,
      RootParityInputs,
      RootRollupInputs,
      RootRollupPublicInputs,
      TubeInputs,
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
      ProvingError,
      PublicKernelCircuitPrivateInputs,
      PublicKernelCircuitPublicInputs,
      PublicKernelTailCircuitPrivateInputs,
      RecursiveProof,
      RootParityInput,
      RootParityInputs,
      RootRollupPublicInputs,
      RootRollupInputs,
      TubeInputs,
      VerificationKeyData,
    },
    {},
    false,
    namespace,
    fetch,
  ) as ProvingJobSource;
}
