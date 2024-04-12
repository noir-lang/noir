import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type MergeRollupInputs,
  type ParityPublicInputs,
  type Proof,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import {
  BaseParityArtifact,
  MergeRollupArtifact,
  RootParityArtifact,
  RootRollupArtifact,
  SimulatedBaseRollupArtifact,
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
import { type SimulationProvider, WASMSimulator } from '@aztec/simulator';

import { type CircuitProver } from './interface.js';

/**
 * A class for use in testing situations (e2e, unit test etc)
 * Simulates circuits using the most efficient method and performs no proving
 */
export class TestCircuitProver implements CircuitProver {
  private wasmSimulator = new WASMSimulator();

  constructor(
    private simulationProvider: SimulationProvider,
    private logger = createDebugLogger('aztec:test-prover'),
  ) {}

  /**
   * Simulates the base parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getBaseParityProof(inputs: BaseParityInputs): Promise<[ParityPublicInputs, Proof]> {
    const witnessMap = convertBaseParityInputsToWitnessMap(inputs);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, BaseParityArtifact);

    const result = convertBaseParityOutputsFromWitnessMap(witness);

    return Promise.resolve([result, makeEmptyProof()]);
  }

  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getRootParityProof(inputs: RootParityInputs): Promise<[ParityPublicInputs, Proof]> {
    const witnessMap = convertRootParityInputsToWitnessMap(inputs);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, RootParityArtifact);

    const result = convertRootParityOutputsFromWitnessMap(witness);

    return Promise.resolve([result, makeEmptyProof()]);
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getBaseRollupProof(input: BaseRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    const witnessMap = convertSimulatedBaseRollupInputsToWitnessMap(input);

    const witness = await this.simulationProvider.simulateCircuit(witnessMap, SimulatedBaseRollupArtifact);

    const result = convertSimulatedBaseRollupOutputsFromWitnessMap(witness);

    return Promise.resolve([result, makeEmptyProof()]);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getMergeRollupProof(input: MergeRollupInputs): Promise<[BaseOrMergeRollupPublicInputs, Proof]> {
    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, MergeRollupArtifact);

    const result = convertMergeRollupOutputsFromWitnessMap(witness);

    return Promise.resolve([result, makeEmptyProof()]);
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getRootRollupProof(input: RootRollupInputs): Promise<[RootRollupPublicInputs, Proof]> {
    const witnessMap = convertRootRollupInputsToWitnessMap(input);

    // use WASM here as it is faster for small circuits
    const [duration, witness] = await elapsed(() => this.wasmSimulator.simulateCircuit(witnessMap, RootRollupArtifact));

    const result = convertRootRollupOutputsFromWitnessMap(witness);

    this.logger.debug(`Simulated root rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return Promise.resolve([result, makeEmptyProof()]);
  }
}
