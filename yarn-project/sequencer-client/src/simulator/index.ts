import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  EthAddress,
  MergeRollupInputs,
  PublicCircuitPublicInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelInputs,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  TxRequest,
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
 * Circuit simulator for the public circuit.
 */
export interface PublicCircuitSimulator {
  /**
   * Simulates the public circuit given a public tx and bytecode to execute.
   * @param tx - Transaction request to execute.
   * @param functionBytecode - Corresponding bytecode to run.
   * @param portalAddress - Portal contract address for the contract being run.
   * @returns The public inputs as outputs of the simulation.
   */
  publicCircuit(tx: TxRequest, functionBytecode: Buffer, portalAddress: EthAddress): Promise<PublicCircuitPublicInputs>;
}

/**
 * Circuit simulator for the public kernel circuits.
 */
export interface PublicKernelCircuitSimulator {
  /**
   * Simulates the public kernel circuit (with no previous kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitNoInput(inputs: PublicKernelInputsNoPreviousKernel): Promise<PublicKernelPublicInputs>;
  /**
   * Simulates the public kernel circuit (with a previous private kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitPrivateInput(inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs>;
  /**
   * Simulates the public kernel circuit (with no previous public kernel circuit run) from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  publicKernelCircuitNonFirstIteration(inputs: PublicKernelInputs): Promise<PublicKernelPublicInputs>;
}
