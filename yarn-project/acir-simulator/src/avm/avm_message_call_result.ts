import { Fr } from '@aztec/foundation/fields';

/**
 * AVM message call result.
 */
export class AvmMessageCallResult {
  /** - */
  public readonly reverted: boolean;
  /** - */
  public readonly revertReason: Error | undefined;
  /** .- */
  public readonly output: Fr[];

  private constructor(reverted: boolean, output: Fr[], revertReason?: Error) {
    this.reverted = reverted;
    this.output = output;
    this.revertReason = revertReason;
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
   * @param reason - Optional reason for revert
   * @returns instance of AvmMessageCallResult
   */
  public static revert(output: Fr[], reason?: Error): AvmMessageCallResult {
    return new AvmMessageCallResult(true, output, reason);
  }
}
