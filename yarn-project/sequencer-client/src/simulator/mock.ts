import {
  BaseRollupInputs,
  BaseOrMergeRollupPublicInputs,
  MergeRollupInputs,
  MergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';
import { Simulator } from './index.js';

/* eslint-disable */

export class MockSimulator implements Simulator {
  baseRollupCircuit(input: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
  mergeRollupCircuit(input: MergeRollupInputs): Promise<MergeRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
  rootRollupCircuit(input: RootRollupInputs): Promise<RootRollupPublicInputs> {
    throw new Error('Method not implemented.');
  }
}
