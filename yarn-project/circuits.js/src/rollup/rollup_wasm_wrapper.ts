import { BaseOrMergeRollupPublicInputs, BaseRollupInputs } from '../index.js';
import { callWasm } from '../utils/call_wasm.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

export { mergeRollupSim, rootRollupSim } from '../cbind/circuits.gen.js';

/**
 * A wrapper around `CircuitsWasm` used to expose only the functions relevant for rollup circuits.
 */
export class RollupWasmWrapper {
  constructor(private wasm: CircuitsWasm) {}

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param baseRollupInputs - Inputs to the circuit.
   * @returns The result of the simulation. Since the circuits are recursive the result is in a form which can be used
   * as an input of the next iteration.
   */
  public simulateBaseRollup(baseRollupInputs: BaseRollupInputs): BaseOrMergeRollupPublicInputs {
    return callWasm(this.wasm, 'base_rollup__sim', baseRollupInputs, BaseOrMergeRollupPublicInputs);
  }
}
