/**
 * An interface representing curve operations.
 */
export interface Curve {
  /**
   * Multiplies a point by a scalar (adds the point `scalar` amount of time).
   * @param point - Point to multiply.
   * @param scalar - Scalar to multiply by.
   * @returns Result of the multiplication.
   */
  mul(point: Uint8Array, scalar: Uint8Array): Buffer;

  /**
   * Gets a random field element.
   * @returns Random field element.
   */
  getRandomFr(): Buffer;

  /**
   * Converts a 512 bits long buffer to a field.
   * @param uint512Buf - The buffer to convert.
   * @returns Buffer representation of the field element.
   */
  reduce512BufferToFr(uint512Buf: Buffer): Buffer;

  /**
   * Point generator
   * @returns The generator for the curve.
   */
  generator(): Buffer;
}
