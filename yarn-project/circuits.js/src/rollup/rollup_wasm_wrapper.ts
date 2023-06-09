import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '../index.js';
import { callWasm } from '../utils/call_wasm.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

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

  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param mergeRollupInputs - Inputs to the circuit.
   * @returns The result of the simulation. Since the circuits are recursive the result is in a form which can be used
   * as an input of the next iteration.
   */
  public simulateMergeRollup(mergeRollupInputs: MergeRollupInputs): BaseOrMergeRollupPublicInputs {
    return callWasm(this.wasm, 'merge_rollup__sim', mergeRollupInputs, BaseOrMergeRollupPublicInputs);
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param rootRollupInputs - Inputs to the circuit.
   * @returns Public inputs of the root rollup circuit.
   */
  public simulateRootRollup(rootRollupInputs: RootRollupInputs): RootRollupPublicInputs {
    return callWasm(this.wasm, 'root_rollup__sim', rootRollupInputs, RootRollupPublicInputs);
  }
}
