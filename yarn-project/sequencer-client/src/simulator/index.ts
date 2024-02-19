import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  PublicKernelCircuitPrivateInputs,
  PublicKernelCircuitPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';

/**
 * Circuit simulator for the rollup circuits.
 */
export interface RollupSimulator {
  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs>;
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs>;
  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs>;
}

/**
 * Circuit simulator for the public kernel circuits.
 */
export interface PublicKernelCircuitSimulator {
  /**
   * Simulates the public kernel setup circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitSetup(inputs: PublicKernelCircuitPrivateInputs): Promise<PublicKernelCircuitPublicInputs>;
  /**
   * Simulates the public kernel app logic circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitAppLogic(inputs: PublicKernelCircuitPrivateInputs): Promise<PublicKernelCircuitPublicInputs>;
  /**
   * Simulates the public kernel teardown circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitTeardown(inputs: PublicKernelCircuitPrivateInputs): Promise<PublicKernelCircuitPublicInputs>;
}
