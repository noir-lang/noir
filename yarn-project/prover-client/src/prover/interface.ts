import { type PublicKernelNonTailRequest, type PublicKernelTailRequest, PublicKernelType } from '@aztec/circuit-types';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type Proof,
  type PublicCircuitPublicInputs,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
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
  getBaseParityProof(inputs: BaseParityInputs): Promise<[ParityPublicInputs, Proof]>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootParityProof(inputs: RootParityInputs): Promise<[ParityPublicInputs, Proof]>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getBaseRollupProof(input: BaseRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getMergeRollupProof(input: MergeRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   */
  getRootRollupProof(input: RootRollupInputs): Promise<[RootRollupPublicInputs, Proof]>;

  /**
   * Create a public kernel proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicKernelProof(kernelRequest: PublicKernelNonTailRequest): Promise<[PublicKernelCircuitPublicInputs, Proof]>;

  /**
   * Create a public kernel tail proof.
   * @param kernelRequest - Object containing the details of the proof required
   */
  getPublicTailProof(kernelRequest: PublicKernelTailRequest): Promise<[KernelCircuitPublicInputs, Proof]>;

  /**
   * Verifies a circuit proof
   */
  verifyProof(artifact: ServerProtocolArtifact, proof: Proof): Promise<void>;
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
