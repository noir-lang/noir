import {
  BaseRollupInputs,
  BaseRollupPublicInputs,
  MergeRollupInputs,
  MergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { Prover } from './index.js';

/* eslint-disable */

export class MockProver implements Prover {
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
  mergeRollupCircuit(input: MergeRollupInputs): Promise<MergeRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
}
