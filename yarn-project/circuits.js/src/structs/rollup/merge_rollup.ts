import { serializeToBuffer } from '../../utils/serialize.js';
import { PreviousRollupData } from './previous_rollup_data.js';

export class MergeRollupInputs {
  constructor(public previousRollupData: [PreviousRollupData, PreviousRollupData]) {}

  toBuffer() {
    return serializeToBuffer(this.previousRollupData);
  }
}
