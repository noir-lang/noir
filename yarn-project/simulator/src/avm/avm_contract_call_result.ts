import { type Fr } from '@aztec/foundation/fields';

import { type AvmRevertReason } from './errors.js';

/**
 * Results of an contract call's execution in the AVM.
 */
export class AvmContractCallResult {
  constructor(public reverted: boolean, public output: Fr[], public revertReason?: AvmRevertReason) {}

  toString(): string {
    let resultsStr = `reverted: ${this.reverted}, output: ${this.output}`;
    if (this.revertReason) {
      resultsStr += `, revertReason: ${this.revertReason}`;
    }
    return resultsStr;
  }
}
