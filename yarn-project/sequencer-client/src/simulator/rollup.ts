import { CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import {
  BaseRollupArtifact,
  MergeRollupArtifact,
  RootRollupArtifact,
  convertBaseRollupInputsToWitnessMap,
  convertBaseRollupOutputsFromWitnessMap,
  convertMergeRollupInputsToWitnessMap,
  convertMergeRollupOutputsFromWitnessMap,
  convertRootRollupInputsToWitnessMap,
  convertRootRollupOutputsFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';

import { RollupSimulator, WASMSimulator } from './index.js';
import { SimulationProvider } from './simulation_provider.js';

/**
 * Implements the rollup circuit simulator.
 */
export class RealRollupCircuitSimulator implements RollupSimulator {
  private log = createDebugLogger('aztec:rollup-simulator');

  // Some circuits are so small it is faster to use WASM
  private wasmSimulator: WASMSimulator = new WASMSimulator();

  constructor(private simulationProvider: SimulationProvider) {}

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const witnessMap = convertBaseRollupInputsToWitnessMap(input);

    const witness = await this.simulationProvider.simulateCircuit(witnessMap, BaseRollupArtifact);

    const result = convertBaseRollupOutputsFromWitnessMap(witness);

    return Promise.resolve(result);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, MergeRollupArtifact);

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

    const [duration, witness] = await elapsed(() => this.wasmSimulator.simulateCircuit(witnessMap, RootRollupArtifact));

    const result = convertRootRollupOutputsFromWitnessMap(witness);

    this.log(`Simulated root rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);

    return result;
  }
}
