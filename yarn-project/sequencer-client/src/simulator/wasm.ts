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

export class WasmCircuitSimulator implements RollupSimulator {
  private rollupWasmWrapper: RollupWasmWrapper;

  constructor(wasm: CircuitsWasm) {
    this.rollupWasmWrapper = new RollupWasmWrapper(wasm);
  }

  public static async new() {
    return new this(await CircuitsWasm.get());
  }

  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return this.rollupWasmWrapper.simulateBaseRollup(input);
  }
  mergeRollupCircuit(input: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return this.rollupWasmWrapper.simulateMergeRollup(input);
  }
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    return this.rollupWasmWrapper.simulateRootRollup(input);
  }
}
