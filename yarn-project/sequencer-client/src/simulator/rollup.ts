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

import { RollupSimulator } from './index.js';

/**
 * Implements the rollup circuit simulator using the wasm circuits implementation.
 */
export class WasmRollupCircuitSimulator implements RollupSimulator {
  private wasm: CircuitsWasm;

  constructor(wasm: CircuitsWasm) {
    this.wasm = wasm;
  }

  /**
   * Creates a new instance using the default CircuitsWasm module.
   * @returns A new instance.
   */
  public static async new() {
    return new this(await CircuitsWasm.get());
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const result = baseRollupSim(this.wasm, input);
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }

    return Promise.resolve(result);
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    const result = mergeRollupSim(this.wasm, input);
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }

    return Promise.resolve(result);
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    const result = rootRollupSim(this.wasm, input);
    if (result instanceof CircuitError) {
      throw new CircuitError(result.code, result.message);
    }

    return Promise.resolve(result);
  }
}
