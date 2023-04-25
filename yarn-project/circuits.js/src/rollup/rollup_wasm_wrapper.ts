import {
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '../index.js';
import { callAsyncWasm } from '../utils/call_wasm.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

export class RollupWasmWrapper {
  constructor(private wasm: CircuitsWasm) {}

  public simulateBaseRollup(baseRollupInputs: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return callAsyncWasm(this.wasm, 'base_rollup__sim', baseRollupInputs, BaseOrMergeRollupPublicInputs);
  }

  public simulateMergeRollup(mergeRollupInputs: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return callAsyncWasm(this.wasm, 'merge_rollup__sim', mergeRollupInputs, BaseOrMergeRollupPublicInputs);
  }

  public simulateRootRollup(rootRollupInputs: RootRollupInputs): Promise<RootRollupPublicInputs> {
    return callAsyncWasm(this.wasm, 'root_rollup__sim', rootRollupInputs, RootRollupPublicInputs);
  }
}
