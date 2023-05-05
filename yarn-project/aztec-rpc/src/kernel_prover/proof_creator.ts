import { pedersenCompressWithHashIndex } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import {
  CircuitsWasm,
  PreviousKernelData,
  PrivateCallData,
  PrivateCircuitPublicInputs,
  KernelCircuitPublicInputs,
  SignedTxRequest,
  UInt8Vector,
  makeEmptyProof,
  privateKernelSim,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

/**
 * Represents the output of the proof creation process.
 * Contains the public inputs required for the kernel circuit and the generated proof as a UInt8Vector.
 */
export interface ProofOutput {
  /**
   * The public inputs required for the proof generation process.
   */
  publicInputs: KernelCircuitPublicInputs;
  /**
   * The zk-SNARK proof for the kernel execution.
   */
  proof: UInt8Vector;
}

/**
 * ProofCreator provides functionality to create and validate proofs, and retrieve
 * siloed commitments necessary for maintaining transaction privacy and security on the network.
 */
export interface ProofCreator {
  getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs): Promise<Fr[]>;
  createProof(
    signedTxRequest: SignedTxRequest,
    previousKernelData: PreviousKernelData,
    privateCallData: PrivateCallData,
    firstIteration: boolean,
  ): Promise<ProofOutput>;
}

const OUTER_COMMITMENT = 3;

/**
 * The KernelProofCreator class is responsible for generating siloed commitments and zero-knowledge proofs
 * for private kernel circuit. It leverages Barretenberg and Circuits Wasm libraries
 * to perform cryptographic operations and proof creation. The class provides methods to compute commitments
 * based on the given public inputs and to generate proofs based on signed transaction requests, previous kernel
 * data, private call data, and a flag indicating whether it's the first iteration or not.
 */
export class KernelProofCreator {
  constructor(private log = createDebugLogger('aztec:kernel_proof_creator')) {}

  /**
   * Computes the siloed commitments for a given set of public inputs.
   *
   * @param publicInputs - The public inputs containing the contract address and new commitments to be used in generating siloed commitments.
   * @returns An array of Fr (finite field) elements representing the siloed commitments.
   */
  public async getSiloedCommitments(publicInputs: PrivateCircuitPublicInputs) {
    const bbWasm = await BarretenbergWasm.get();
    const contractAddress = publicInputs.callContext.storageContractAddress.toBuffer();
    // TODO
    // Should match `add_contract_address_to_commitment` in hash.hpp.
    // Should use a function exported from circuits.js.
    return publicInputs.newCommitments.map(commitment =>
      Fr.fromBuffer(pedersenCompressWithHashIndex(bbWasm, [contractAddress, commitment.toBuffer()], OUTER_COMMITMENT)),
    );
  }

  /**
   * Creates a proof output for a given signed transaction request, previous kernel data, private call data, and first iteration flag.
   *
   * @param signedTxRequest - The signed transaction request object.
   * @param previousKernelData - The previous kernel data object.
   * @param privateCallData - The private call data object.
   * @param firstIteration - A boolean flag indicating if it's the first iteration of the kernel proof creation process.
   * @returns A Promise resolving to a ProofOutput object containing public inputs and the kernel proof.
   */
  public async createProof(
    signedTxRequest: SignedTxRequest,
    previousKernelData: PreviousKernelData,
    privateCallData: PrivateCallData,
    firstIteration: boolean,
  ): Promise<ProofOutput> {
    const wasm = await CircuitsWasm.get();
    this.log('Executing private kernel simulation...');
    const publicInputs = await privateKernelSim(
      wasm,
      signedTxRequest,
      previousKernelData,
      privateCallData,
      firstIteration,
    );
    this.log('Skipping private kernel proving...');
    // TODO
    const proof = makeEmptyProof();
    this.log('Kernel Prover Completed!');

    return {
      publicInputs,
      proof,
    };
  }
}
