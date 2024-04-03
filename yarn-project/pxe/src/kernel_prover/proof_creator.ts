import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type Proof,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { siloNoteHash } from '@aztec/circuits.js/hash';
import { type Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import { executeInit, executeInner, executeTail, executeTailForPublic } from '@aztec/noir-protocol-circuits-types';

/**
 * Represents the output of the proof creation process for init and inner private kernel circuit.
 * Contains the public inputs required for the init and inner private kernel circuit and the generated proof.
 */
export interface ProofOutput {
  /**
   * The public inputs required for the proof generation process.
   */
  publicInputs: PrivateKernelCircuitPublicInputs;
  /**
   * The zk-SNARK proof for the kernel execution.
   */
  proof: Proof;
}

/**
 * Represents the output of the proof creation process for final ordering private kernel circuit.
 * Contains the public inputs required for the final ordering private kernel circuit and the generated proof.
 */
export interface ProofOutputFinal {
  /**
   * The public inputs required for the proof generation process.
   */
  publicInputs: PrivateKernelTailCircuitPublicInputs;
  /**
   * The zk-SNARK proof for the kernel execution.
   */
  proof: Proof;
}

/**
 * ProofCreator provides functionality to create and validate proofs, and retrieve
 * siloed commitments necessary for maintaining transaction privacy and security on the network.
 */
export interface ProofCreator {
  /**
   * Computes the siloed commitments for a given set of public inputs.
   *
   * @param publicInputs - The public inputs containing the contract address and new note hashes to be used in generating siloed note hashes.
   * @returns An array of Fr (finite field) elements representing the siloed commitments.
   */
  getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs): Promise<Fr[]>;

  /**
   * Creates a proof output for a given signed transaction request and private call data for the first iteration.
   *
   * @param privateKernelInputsInit - The private data structure for the initial iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofInit(privateKernelInputsInit: PrivateKernelInitCircuitPrivateInputs): Promise<ProofOutput>;

  /**
   * Creates a proof output for a given previous kernel data and private call data for an inner iteration.
   *
   * @param privateKernelInputsInner - The private input data structure for the inner iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofInner(privateKernelInputsInner: PrivateKernelInnerCircuitPrivateInputs): Promise<ProofOutput>;

  /**
   * Creates a proof output based on the last inner kernel iteration kernel data for the final ordering iteration.
   *
   * @param privateKernelInputsTail - The private input data structure for the final ordering iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofTail(privateKernelInputsTail: PrivateKernelTailCircuitPrivateInputs): Promise<ProofOutputFinal>;
}

/**
 * The KernelProofCreator class is responsible for generating siloed commitments and zero-knowledge proofs
 * for private kernel circuit. It leverages Barretenberg to perform cryptographic operations and proof creation.
 * The class provides methods to compute commitments based on the given public inputs and to generate proofs based on
 * signed transaction requests, previous kernel data, private call data, and a flag indicating whether it's the first
 * iteration or not.
 */
export class KernelProofCreator implements ProofCreator {
  constructor(private log = createDebugLogger('aztec:kernel_proof_creator')) {}

  public getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const contractAddress = publicInputs.callContext.storageContractAddress;

    return Promise.resolve(
      publicInputs.newNoteHashes.map(commitment => siloNoteHash(contractAddress, commitment.value)),
    );
  }

  public async createProofInit(privateInputs: PrivateKernelInitCircuitPrivateInputs): Promise<ProofOutput> {
    const [duration, result] = await elapsed(() => executeInit(privateInputs));
    this.log(`Simulated private kernel init`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-init',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    this.log('Skipping private kernel init proving...');
    const proof = makeEmptyProof();

    return {
      publicInputs: result,
      proof: proof,
    };
  }

  public async createProofInner(privateInputs: PrivateKernelInnerCircuitPrivateInputs): Promise<ProofOutput> {
    const [duration, result] = await elapsed(() => executeInner(privateInputs));
    this.log(`Simulated private kernel inner`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-inner',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    this.log('Skipping private kernel inner proving...');
    const proof = makeEmptyProof();

    return {
      publicInputs: result,
      proof: proof,
    };
  }

  public async createProofTail(privateInputs: PrivateKernelTailCircuitPrivateInputs): Promise<ProofOutputFinal> {
    const isForPublic = privateInputs.isForPublic();
    const [duration, result] = await elapsed(() =>
      isForPublic ? executeTailForPublic(privateInputs) : executeTail(privateInputs),
    );
    this.log(`Simulated private kernel ordering`, {
      eventName: 'circuit-simulation',
      circuitName: 'private-kernel-ordering',
      duration,
      inputSize: privateInputs.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    this.log('Skipping private kernel ordering proving...');
    const proof = makeEmptyProof();

    return {
      publicInputs: result,
      proof: proof,
    };
  }
}
