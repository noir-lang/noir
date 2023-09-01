import {
  CircuitError,
  CircuitsWasm,
  KernelCircuitPublicInputs,
  KernelCircuitPublicInputsFinal,
  PreviousKernelData,
  PrivateCallData,
  PrivateCircuitPublicInputs,
  PrivateKernelInputsOrdering,
  Proof,
  TxRequest,
  makeEmptyProof,
  privateKernelSimInit,
  privateKernelSimInner,
  privateKernelSimOrdering,
} from '@aztec/circuits.js';
import { siloCommitment } from '@aztec/circuits.js/abis';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

/**
 * Represents the output of the proof creation process for init and inner private kernel circuit.
 * Contains the public inputs required for the init and inner private kernel circuit and the generated proof.
 */
export interface ProofOutput {
  /**
   * The public inputs required for the proof generation process.
   * Note: C++ side does not define the specific data structure PrivateKernelPublicInputs and therefore
   * would not generate a binding in circuits.gen.ts.
   */
  publicInputs: KernelCircuitPublicInputs;
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
   * Note: C++ side does not define the specific data structure PrivateKernelPublicInputsFinal and therefore
   * would not generate a binding in circuits.gen.ts.
   */
  publicInputs: KernelCircuitPublicInputsFinal;
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
   * @param publicInputs - The public inputs containing the contract address and new commitments to be used in generating siloed commitments.
   * @returns An array of Fr (finite field) elements representing the siloed commitments.
   */
  getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs): Promise<Fr[]>;

  /**
   * Creates a proof output for a given signed transaction request and private call data for the first iteration.
   *
   * @param txRequest - The signed transaction request object.
   * @param privateCallData - The private call data object.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofInit(txRequest: TxRequest, privateCallData: PrivateCallData): Promise<ProofOutput>;

  /**
   * Creates a proof output for a given previous kernel data and private call data for an inner iteration.
   *
   * @param previousKernelData - The previous kernel data object.
   * @param privateCallData - The private call data object.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofInner(previousKernelData: PreviousKernelData, privateCallData: PrivateCallData): Promise<ProofOutput>;

  /**
   * Creates a proof output based on the last inner kernel iteration kernel data for the final ordering iteration.
   *
   * @param previousKernelData - The previous kernel data object.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  createProofOrdering(previousKernelData: PrivateKernelInputsOrdering): Promise<ProofOutputFinal>;
}

/**
 * The KernelProofCreator class is responsible for generating siloed commitments and zero-knowledge proofs
 * for private kernel circuit. It leverages Barretenberg and Circuits Wasm libraries
 * to perform cryptographic operations and proof creation. The class provides methods to compute commitments
 * based on the given public inputs and to generate proofs based on signed transaction requests, previous kernel
 * data, private call data, and a flag indicating whether it's the first iteration or not.
 */
export class KernelProofCreator implements ProofCreator {
  constructor(private log = createDebugLogger('aztec:kernel_proof_creator')) {}

  public async getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const wasm = await CircuitsWasm.get();
    const contractAddress = publicInputs.callContext.storageContractAddress;

    return publicInputs.newCommitments.map(commitment => siloCommitment(wasm, contractAddress, commitment));
  }

  public async createProofInit(txRequest: TxRequest, privateCallData: PrivateCallData): Promise<ProofOutput> {
    const wasm = await CircuitsWasm.get();
    this.log('Executing private kernel simulation init...');
    const publicInputs = privateKernelSimInit(wasm, txRequest, privateCallData);
    this.log('Skipping private kernel init proving...');
    // TODO
    const proof = makeEmptyProof();
    this.log('Kernel Prover Init Completed!');

    return {
      publicInputs,
      proof,
    };
  }

  public async createProofInner(
    previousKernelData: PreviousKernelData,
    privateCallData: PrivateCallData,
  ): Promise<ProofOutput> {
    const wasm = await CircuitsWasm.get();
    this.log('Executing private kernel simulation inner...');
    const publicInputs = privateKernelSimInner(wasm, previousKernelData, privateCallData);
    this.log('Skipping private kernel inner proving...');
    // TODO
    const proof = makeEmptyProof();
    this.log('Kernel Prover Inner Completed!');

    return {
      publicInputs,
      proof,
    };
  }

  public async createProofOrdering(privateInputs: PrivateKernelInputsOrdering): Promise<ProofOutputFinal> {
    const wasm = await CircuitsWasm.get();
    this.log('Executing private kernel simulation ordering...');
    const result = privateKernelSimOrdering(wasm, privateInputs);
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }
    this.log('Skipping private kernel ordering proving...');
    // TODO
    const proof = makeEmptyProof();
    this.log('Ordering Kernel Prover Ordering Completed!');

    const publicInputs = result as KernelCircuitPublicInputsFinal;

    return {
      publicInputs,
      proof,
    };
  }
}
