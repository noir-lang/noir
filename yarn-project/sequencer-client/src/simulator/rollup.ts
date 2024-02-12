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
import { executeBaseRollup, executeMergeRollup, executeRootRollup } from '@aztec/noir-protocol-circuits-types';

import { RollupSimulator } from './index.js';

/**
 * Implements the rollup circuit simulator.
 */
export class RealRollupCircuitSimulator implements RollupSimulator {
  private log = createDebugLogger('aztec:rollup-simulator');

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const [duration, result] = await elapsed(() => executeBaseRollup(input));

    this.log(`Simulated base rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'base-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);

    return Promise.resolve(result);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const [duration, result] = await elapsed(() => executeMergeRollup(input));

    this.log(`Simulated merge rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'merge-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);

    return result;
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    const [duration, result] = await elapsed(() => executeRootRollup(input));

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
