import { Fr } from '@aztec/foundation/fields';

/**
 * AVM message call result.
 */
export class AvmMessageCallResult {
  /** - */
  public readonly reverted: boolean;
  /** .- */
  public readonly output: Fr[];

  constructor(reverted: boolean, output: Fr[]) {
    this.reverted = reverted;
    this.output = output;
  }

  /**
   * Terminate a call as a success
   * @param output - Return data
   * @returns instance of AvmMessageCallResult
   */
  public static success(output: Fr[]): AvmMessageCallResult {
    return new AvmMessageCallResult(false, output);
  }

  /**
   * Terminate a call as a revert
   * @param output - Return data ( revert message )
   * @returns instance of AvmMessageCallResult
   */
  public static revert(output: Fr[]): AvmMessageCallResult {
    return new AvmMessageCallResult(true, output);
  }
}
