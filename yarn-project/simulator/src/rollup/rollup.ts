import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import {
  SimulatedServerCircuitArtifacts,
  convertBaseParityInputsToWitnessMap,
  convertBaseParityOutputsFromWitnessMap,
  convertMergeRollupInputsToWitnessMap,
  convertMergeRollupOutputsFromWitnessMap,
  convertRootParityInputsToWitnessMap,
  convertRootParityOutputsFromWitnessMap,
  convertRootRollupInputsToWitnessMap,
  convertRootRollupOutputsFromWitnessMap,
  convertSimulatedBaseRollupInputsToWitnessMap,
  convertSimulatedBaseRollupOutputsFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';

import { WASMSimulator } from '../providers/acvm_wasm.js';
import { type SimulationProvider } from '../providers/simulation_provider.js';

/**
 * Circuit simulator for the rollup circuits.
 */
export interface RollupSimulator {
  /**
   * Simulates the base parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  baseParityCircuit(inputs: BaseParityInputs): Promise<ParityPublicInputs>;
  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  rootParityCircuit(inputs: RootParityInputs): Promise<ParityPublicInputs>;
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
 * Implements the rollup circuit simulator.
 */
export class RealRollupCircuitSimulator implements RollupSimulator {
  private log = createDebugLogger('aztec:rollup-simulator');

  // Some circuits are so small it is faster to use WASM
  private wasmSimulator: WASMSimulator = new WASMSimulator();

  constructor(private simulationProvider: SimulationProvider) {}

  /**
   * Simulates the base parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async baseParityCircuit(inputs: BaseParityInputs): Promise<ParityPublicInputs> {
    const witnessMap = convertBaseParityInputsToWitnessMap(inputs);

    const witness = await this.simulationProvider.simulateCircuit(
      witnessMap,
      SimulatedServerCircuitArtifacts.BaseParityArtifact,
    );

    const result = convertBaseParityOutputsFromWitnessMap(witness);

    return Promise.resolve(result);
  }

  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async rootParityCircuit(inputs: RootParityInputs): Promise<ParityPublicInputs> {
    const witnessMap = convertRootParityInputsToWitnessMap(inputs);

    const witness = await this.simulationProvider.simulateCircuit(
      witnessMap,
      SimulatedServerCircuitArtifacts.RootParityArtifact,
    );

    const result = convertRootParityOutputsFromWitnessMap(witness);

    return Promise.resolve(result);
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const witnessMap = convertSimulatedBaseRollupInputsToWitnessMap(input);

    const witness = await this.simulationProvider.simulateCircuit(
      witnessMap,
      SimulatedServerCircuitArtifacts.BaseRollupArtifact,
    );

    const result = convertSimulatedBaseRollupOutputsFromWitnessMap(witness);

    return Promise.resolve(result);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    const witness = await this.wasmSimulator.simulateCircuit(
      witnessMap,
      SimulatedServerCircuitArtifacts.MergeRollupArtifact,
    );

    const result = convertMergeRollupOutputsFromWitnessMap(witness);

    return result;
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    const witnessMap = convertRootRollupInputsToWitnessMap(input);

    const [duration, witness] = await elapsed(() =>
      this.wasmSimulator.simulateCircuit(witnessMap, SimulatedServerCircuitArtifacts.RootRollupArtifact),
    );

    const result = convertRootRollupOutputsFromWitnessMap(witness);

    this.log.debug(`Simulated root rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return result;
  }
}
