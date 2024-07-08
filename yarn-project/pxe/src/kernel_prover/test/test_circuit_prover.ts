import { type AppCircuitProofOutput, type KernelProofOutput, type ProofCreator } from '@aztec/circuit-types';
import type { CircuitName, CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  NESTED_RECURSIVE_PROOF_LENGTH,
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputsVariants,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  VerificationKeyAsFields,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { siloNoteHash } from '@aztec/circuits.js/hash';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import {
  type ProtocolArtifact,
  ProtocolCircuitVks,
  executeInit,
  executeInner,
  executeReset,
  executeTail,
  executeTailForPublic,
} from '@aztec/noir-protocol-circuits-types';

/**
 * Test Proof Creator executes circuit simulations and provides fake proofs.
 */
export class TestProofCreator implements ProofCreator {
  constructor(private log = createDebugLogger('aztec:test_proof_creator')) {}

  public getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const contractAddress = publicInputs.callContext.storageContractAddress;

    return Promise.resolve(publicInputs.noteHashes.map(commitment => siloNoteHash(contractAddress, commitment.value)));
  }

  public async createProofInit(
    privateInputs: PrivateKernelInitCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    const [duration, result] = await elapsed(() => executeInit(privateInputs));
    this.log.debug(`Simulated private kernel init`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-init',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return this.makeEmptyKernelProofOutput<PrivateKernelCircuitPublicInputs>(result, 'PrivateKernelInitArtifact');
  }

  public async createProofInner(
    privateInputs: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    const [duration, result] = await elapsed(() => executeInner(privateInputs));
    this.log.debug(`Simulated private kernel inner`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-inner',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return this.makeEmptyKernelProofOutput<PrivateKernelCircuitPublicInputs>(result, 'PrivateKernelInnerArtifact');
  }

  public async createProofReset(
    privateInputs: PrivateKernelResetCircuitPrivateInputsVariants,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>> {
    const [duration, result] = await elapsed(() => executeReset(privateInputs));
    this.log.debug(`Simulated private kernel reset`, {
      eventName: 'circuit-simulation',
      circuitName: ('private-kernel-reset-' + privateInputs.sizeTag) as CircuitName,
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return this.makeEmptyKernelProofOutput<PrivateKernelCircuitPublicInputs>(result, 'PrivateKernelResetFullArtifact');
  }

  public async createProofTail(
    privateInputs: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    const isForPublic = privateInputs.isForPublic();
    const [duration, result] = await elapsed(() =>
      isForPublic ? executeTailForPublic(privateInputs) : executeTail(privateInputs),
    );
    this.log.debug(`Simulated private kernel ordering`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-tail',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return this.makeEmptyKernelProofOutput<PrivateKernelTailCircuitPublicInputs>(
      result,
      isForPublic ? 'PrivateKernelTailToPublicArtifact' : 'PrivateKernelTailArtifact',
    );
  }

  createAppCircuitProof(_1: Map<number, string>, _2: Buffer): Promise<AppCircuitProofOutput> {
    const appCircuitProofOutput: AppCircuitProofOutput = {
      proof: makeRecursiveProof<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH),
      verificationKey: VerificationKeyAsFields.makeEmpty(),
    };
    return Promise.resolve(appCircuitProofOutput);
  }

  private makeEmptyKernelProofOutput<PublicInputsType>(publicInputs: PublicInputsType, circuitType: ProtocolArtifact) {
    const kernelProofOutput: KernelProofOutput<PublicInputsType> = {
      publicInputs,
      proof: makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH),
      verificationKey: ProtocolCircuitVks[circuitType].keyAsFields,
    };
    return kernelProofOutput;
  }
}
