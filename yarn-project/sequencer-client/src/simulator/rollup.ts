import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CircuitError,
  CircuitsWasm,
  MergeRollupInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  baseRollupSim,
  mergeRollupSim,
  rootRollupSim,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';

import { RollupSimulator } from './index.js';

/**
 * Implements the rollup circuit simulator using the wasm circuits implementation.
 */
export class WasmRollupCircuitSimulator implements RollupSimulator {
  private log = createDebugLogger('aztec:rollup-simulator');

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const wasm = await CircuitsWasm.get();
    const [duration, result] = await elapsed(() => baseRollupSim(wasm, input));
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }

    this.log(`Simulated base rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'base-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    });

    return Promise.resolve(result);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const wasm = await CircuitsWasm.get();
    const [duration, result] = await elapsed(() => mergeRollupSim(wasm, input));
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }

    this.log(`Simulated merge rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'merge-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    });

    return result;
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    const wasm = await CircuitsWasm.get();
    const [duration, result] = await elapsed(() => rootRollupSim(wasm, input));
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }

    this.log(`Simulated root rollup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'root-rollup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    });

    return result;
  }
}
