import {
  type PublicInputsAndProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  type Tx,
} from '@aztec/circuit-types';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type PrivateKernelEmptyInputData,
  type Proof,
  type PublicCircuitPublicInputs,
  type PublicKernelCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  type RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
  type VerificationKeys,
} from '@aztec/circuits.js';

/**
 * Generates proofs for parity and rollup circuits.
 */
export interface ServerCircuitProver {
  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getBaseParityProof(
    inputs: BaseParityInputs,
    signal?: AbortSignal,
  ): Promise<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootParityProof(
    inputs: RootParityInputs,
    signal?: AbortSignal,
  ): Promise<RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getBaseRollupProof(
    input: BaseRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getMergeRollupProof(
    input: MergeRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootRollupProof(
    input: RootRollupInputs,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<RootRollupPublicInputs>>;

  /**
   * Create a public kernel proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<PublicKernelCircuitPublicInputs>>;

  /**
   * Create a public kernel tail proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicTailProof(
    kernelRequest: PublicKernelTailRequest,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>>;

  getEmptyPrivateKernelProof(
    inputs: PrivateKernelEmptyInputData,
    signal?: AbortSignal,
  ): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>>;
}

/**
 * Generates proofs for the public and public kernel circuits.
 */
export interface PublicProver {
  /**
   * Creates a proof for the given input.
   * @param publicInputs - Public inputs obtained via simulation.
   */
  getPublicCircuitProof(publicInputs: PublicCircuitPublicInputs): Promise<Proof>;

  /**
   * Creates a proof for the given input.
   * @param publicInputs - Public inputs obtained via simulation.
   */
  getPublicKernelCircuitProof(publicInputs: PublicKernelCircuitPublicInputs): Promise<Proof>;
}

/**
 * A verifier used by nodes to check tx proofs are valid.
 */
export interface ClientProtocolCircuitVerifier {
  /**
   * Verifies the private protocol circuit's proof.
   * @param tx - The tx to verify the proof of
   * @returns True if the proof is valid, false otherwise
   */
  verifyProof(tx: Tx): Promise<boolean>;

  /**
   * Returns the verification keys used to verify tx proofs.
   */
  getVerificationKeys(): Promise<VerificationKeys>;
}
