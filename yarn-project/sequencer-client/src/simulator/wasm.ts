import { CircuitsWasm } from '@aztec/circuits.js';
import { RollupWasmWrapper } from '@aztec/circuits.js';
import {
  BaseRollupInputs,
  BaseOrMergeRollupPublicInputs,
  MergeRollupInputs,
  MergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { Simulator } from './index.js';

export class WasmCircuitSimulator implements Simulator {
  private rollupWasmWrapper: RollupWasmWrapper;

  constructor(wasm: CircuitsWasm) {
    this.rollupWasmWrapper = new RollupWasmWrapper(wasm);
  }

  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return this.rollupWasmWrapper.simulateBaseRollup(input);
  }
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  mergeRollupCircuit(input: MergeRollupInputs): Promise<MergeRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    return this.rollupWasmWrapper.simulateRootRollup(input);
  }
}
