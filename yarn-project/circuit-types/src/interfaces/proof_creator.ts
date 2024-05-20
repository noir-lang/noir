import {
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type PrivateCircuitPublicInputs,
  type PrivateKernelCircuitPublicInputs,
  type PrivateKernelInitCircuitPrivateInputs,
  type PrivateKernelInnerCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputsVariants,
  type PrivateKernelTailCircuitPrivateInputs,
  type PrivateKernelTailCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  type RecursiveProof,
  type VerificationKeyAsFields,
} from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type WitnessMap } from '@noir-lang/acvm_js';

/**
 * Represents the output of the proof creation process for init and inner private kernel circuit.
 * Contains the public inputs required for the init and inner private kernel circuit and the generated proof.
 */
export type KernelProofOutput<PublicInputsType> = {
  /**
   * The public inputs required for the proof generation process.
   */
  publicInputs: PublicInputsType;
  /**
   * The zk-SNARK proof for the kernel execution.
   */
  proof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>;

  verificationKey: VerificationKeyAsFields;
};

/**
 * Represents the output of the proof creation process for init and inner private kernel circuit.
 * Contains the public inputs required for the init and inner private kernel circuit and the generated proof.
 */
export type AppCircuitProofOutput = {
  /**
   * The zk-SNARK proof for the kernel execution.
   */
  proof: RecursiveProof<typeof RECURSIVE_PROOF_LENGTH>;

  verificationKey: VerificationKeyAsFields;
};

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
  createProofInit(
    privateKernelInputsInit: PrivateKernelInitCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>>;

  /**
   * Creates a proof output for a given previous kernel data and private call data for an inner iteration.
   *
   * @param privateKernelInputsInner - The private input data structure for the inner iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofInner(
    privateKernelInputsInner: PrivateKernelInnerCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>>;

  /**
   * Creates a proof output by resetting the arrays using the reset circuit.
   *
   * @param privateKernelInputsTail - The private input data structure for the reset circuit.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofReset(
    privateKernelInputsReset: PrivateKernelResetCircuitPrivateInputsVariants,
  ): Promise<KernelProofOutput<PrivateKernelCircuitPublicInputs>>;

  /**
   * Creates a proof output based on the last inner kernel iteration kernel data for the final ordering iteration.
   *
   * @param privateKernelInputsTail - The private input data structure for the final ordering iteration.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofTail(
    privateKernelInputsTail: PrivateKernelTailCircuitPrivateInputs,
  ): Promise<KernelProofOutput<PrivateKernelTailCircuitPublicInputs>>;

  /**
   * Creates a proof for an app circuit.
   *
   * @param partialWitness - The witness produced via circuit simulation
   * @param bytecode - The circuit bytecode in gzipped bincode format
   * @param appCircuitName - Optionally specify the name of the app circuit
   * @returns A Promise resolving to a Proof object
   */
  createAppCircuitProof(
    partialWitness: WitnessMap,
    bytecode: Buffer,
    appCircuitName?: string,
  ): Promise<AppCircuitProofOutput>;
}
