import { type Fr } from '@aztec/foundation/fields';

/**
 * Results of an contract call's execution in the AVM.
 */
export class AvmContractCallResults {
  public readonly reverted: boolean;
  public readonly output: Fr[];

  /** For exceptional halts */
  public readonly revertReason: Error | undefined;

  constructor(reverted: boolean, output: Fr[], revertReason?: Error) {
    this.reverted = reverted;
    this.output = output;
    this.revertReason = revertReason;
  }

  /**
   * Generate a string representation of call results.
   */
  toString(): string {
    let resultsStr = `reverted: ${this.reverted}, output: ${this.output}`;
    if (this.revertReason) {
      resultsStr += `, revertReason: ${this.revertReason}`;
    }
    return resultsStr;
  }
}
