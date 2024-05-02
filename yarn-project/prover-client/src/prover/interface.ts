import {
  type PublicInputsAndProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  PublicKernelType,
} from '@aztec/circuit-types';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type Proof,
  type PublicCircuitPublicInputs,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  type RECURSIVE_PROOF_LENGTH,
  type RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';
import {
  type ServerProtocolArtifact,
  convertPublicInnerRollupInputsToWitnessMap,
  convertPublicInnerRollupOutputFromWitnessMap,
  convertPublicSetupRollupInputsToWitnessMap,
  convertPublicSetupRollupOutputFromWitnessMap,
  convertPublicTeardownRollupInputsToWitnessMap,
  convertPublicTeardownRollupOutputFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';

import { type WitnessMap } from '@noir-lang/types';

export type PublicKernelProvingOps = {
  artifact: ServerProtocolArtifact;
  convertInputs: (inputs: PublicKernelCircuitPrivateInputs) => WitnessMap;
  convertOutputs: (outputs: WitnessMap) => PublicKernelCircuitPublicInputs;
};

export type KernelTypeToArtifact = Record<PublicKernelType, PublicKernelProvingOps | undefined>;

export const KernelArtifactMapping: KernelTypeToArtifact = {
  [PublicKernelType.NON_PUBLIC]: undefined,
  [PublicKernelType.APP_LOGIC]: {
    artifact: 'PublicKernelAppLogicArtifact',
    convertInputs: convertPublicInnerRollupInputsToWitnessMap,
    convertOutputs: convertPublicInnerRollupOutputFromWitnessMap,
  },
  [PublicKernelType.SETUP]: {
    artifact: 'PublicKernelSetupArtifact',
    convertInputs: convertPublicSetupRollupInputsToWitnessMap,
    convertOutputs: convertPublicSetupRollupOutputFromWitnessMap,
  },
  [PublicKernelType.TEARDOWN]: {
    artifact: 'PublicKernelTeardownArtifact',
    convertInputs: convertPublicTeardownRollupInputsToWitnessMap,
    convertOutputs: convertPublicTeardownRollupOutputFromWitnessMap,
  },
  [PublicKernelType.TAIL]: undefined,
};

/**
 * Generates proofs for parity and rollup circuits.
 */
export interface CircuitProver {
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

  /**
   * Verifies a circuit proof
   */
  verifyProof(artifact: ServerProtocolArtifact, proof: Proof, signal?: AbortSignal): Promise<void>;
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
