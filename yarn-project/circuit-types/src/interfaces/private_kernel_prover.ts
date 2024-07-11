import {
  type ClientIvcProof,
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputsVariants,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type WitnessMap } from '@noir-lang/acvm_js';

/**
 * Represents the output of the proof creation process for init and inner private kernel circuit.
 * Contains the public inputs required for the init and inner private kernel circuit and the generated proof.
 */
export type PrivateKernelSimulateOutput<PublicInputsType> = {
  /**
   * The public inputs required for the proof generation process.
   */
  publicInputs: PublicInputsType;

  clientIvcProof?: ClientIvcProof;

  verificationKey: VerificationKeyAsFields;

  outputWitness: WitnessMap;
};

/**
 * Represents the output of the circuit simulation process for init and inner private kernel circuit.
 */
export type AppCircuitSimulateOutput = {
  verificationKey: VerificationKeyAsFields;
};

/**
 * PrivateKernelProver provides functionality to simulate and validate circuits, and retrieve
 * siloed commitments necessary for maintaining transaction privacy and security on the network.
 */
export interface PrivateKernelProver {
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
  simulateProofInit(
    privateKernelInputsInit: PrivateKernelInitCircuitPrivateInputs,
  ): Promise<PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs>>;

  /**
   * Creates a proof output for a given previous kernel data and private call data for an inner iteration.
   *
   * @param privateKernelInputsInner - The private input data structure for the inner iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  simulateProofInner(
    privateKernelInputsInner: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs>>;

  /**
   * Creates a proof output by resetting the arrays using the reset circuit.
   *
   * @param privateKernelInputsTail - The private input data structure for the reset circuit.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  simulateProofReset(
    privateKernelInputsReset: PrivateKernelResetCircuitPrivateInputsVariants,
  ): Promise<PrivateKernelSimulateOutput<PrivateKernelCircuitPublicInputs>>;

  /**
   * Creates a proof output based on the last inner kernel iteration kernel data for the final ordering iteration.
   *
   * @param privateKernelInputsTail - The private input data structure for the final ordering iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  simulateProofTail(
    privateKernelInputsTail: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<PrivateKernelSimulateOutput<PrivateKernelTailCircuitPublicInputs>>;

  /**
   * Based of a program stack, create a folding proof.
   * @param acirs The program bytecode.
   * @param witnessStack The witnessses for each program bytecode.
   */
  createClientIvcProof(acirs: Buffer[], witnessStack: WitnessMap[]): Promise<ClientIvcProof>;

  /**
   * Creates a proof for an app circuit.
   *
   * @param partialWitness - The witness produced via circuit simulation
   * @param bytecode - The circuit bytecode in gzipped bincode format
   * @param appCircuitName - Optionally specify the name of the app circuit
   * @returns A Promise resolving to a Proof object
   */
  computeAppCircuitVerificationKey(bytecode: Buffer, appCircuitName?: string): Promise<AppCircuitSimulateOutput>;
}
