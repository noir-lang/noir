import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  CircuitsWasm,
  MergeRollupInputs,
  RollupWasmWrapper,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';

import { RollupSimulator } from './index.js';

/**
 * Implements the rollup circuit simulator using the wasm circuits implementation.
 */
export class WasmRollupCircuitSimulator implements RollupSimulator {
  private rollupWasmWrapper: RollupWasmWrapper;

  constructor(wasm: CircuitsWasm) {
    this.rollupWasmWrapper = new RollupWasmWrapper(wasm);
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
    return Promise.resolve(this.rollupWasmWrapper.simulateBaseRollup(input));
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return Promise.resolve(this.rollupWasmWrapper.simulateMergeRollup(input));
  }
  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    return Promise.resolve(this.rollupWasmWrapper.simulateRootRollup(input));
  }
}
