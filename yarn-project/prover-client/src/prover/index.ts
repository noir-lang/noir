import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type Proof,
  type PublicCircuitPublicInputs,
  type PublicKernelCircuitPublicInputs,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';

/**
 * Generates proofs for parity and rollup circuits.
 */
export interface RollupProver {
  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  getBaseParityProof(inputs: BaseParityInputs, publicInputs: ParityPublicInputs): Promise<Proof>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  getRootParityProof(inputs: RootParityInputs, publicInputs: ParityPublicInputs): Promise<Proof>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  getBaseRollupProof(input: BaseRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  getMergeRollupProof(input: MergeRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof>;

  /**
   * Creates a proof for the given input.
   * @param input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  getRootRollupProof(input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<Proof>;
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

  /**
   * Creates a proof for the given input.
   * @param publicInputs - Public inputs obtained via simulation.
   */
  getPublicTailKernelCircuitProof(publicInputs: KernelCircuitPublicInputs): Promise<Proof>;
}
