import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  Proof,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { siloNoteHash } from '@aztec/circuits.js/hash';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import { executeInit, executeInner, executeTail, executeTailForPublic } from '@aztec/noir-protocol-circuits-types';

import { type ProofCreator, type ProofOutput } from '../interface/proof_creator.js';

/**
 * Test Proof Creator executes circuit simulations and provides fake proofs.
 */
export class TestProofCreator implements ProofCreator {
  constructor(private log = createDebugLogger('aztec:test_proof_creator')) {}

  public getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const contractAddress = publicInputs.callContext.storageContractAddress;

    return Promise.resolve(
      publicInputs.newNoteHashes.map(commitment => siloNoteHash(contractAddress, commitment.value)),
    );
  }

  public async createProofInit(
    privateInputs: PrivateKernelInitCircuitPrivateInputs,
  ): Promise<ProofOutput<PrivateKernelCircuitPublicInputs>> {
    const [duration, result] = await elapsed(() => executeInit(privateInputs));
    this.log.debug(`Simulated private kernel init`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-init',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    const proof = makeEmptyProof();

    return {
      publicInputs: result,
      proof: proof,
    };
  }

  public async createProofInner(
    privateInputs: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<ProofOutput<PrivateKernelCircuitPublicInputs>> {
    const [duration, result] = await elapsed(() => executeInner(privateInputs));
    this.log.debug(`Simulated private kernel inner`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-inner',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    const proof = makeEmptyProof();

    return {
      publicInputs: result,
      proof: proof,
    };
  }

  public async createProofTail(
    privateInputs: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<ProofOutput<PrivateKernelTailCircuitPublicInputs>> {
    const isForPublic = privateInputs.isForPublic();
    const [duration, result] = await elapsed(() =>
      isForPublic ? executeTailForPublic(privateInputs) : executeTail(privateInputs),
    );
    this.log.debug(`Simulated private kernel ordering`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-ordering',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    const proof = makeEmptyProof();

    return {
      publicInputs: result,
      proof: proof,
    };
  }

  createAppCircuitProof(_1: Map<number, string>, _2: Buffer): Promise<Proof> {
    return Promise.resolve(new Proof(Buffer.alloc(0)));
  }
}
