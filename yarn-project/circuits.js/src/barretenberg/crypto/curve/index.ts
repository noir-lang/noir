import { Fr, Point, PrivateKey } from '../../../index.js';

/**
 * An interface representing curve operations.
 */
export interface Curve {
  /**
   * Multiplies a point by a private key (adds the point `privateKey` amount of time).
   * @param point - Point to multiply.
   * @param privateKey - Private key to multiply by.
   * @returns Result of the multiplication.
   */
  mul(point: Point, privateKey: PrivateKey): Point;

  /**
   * Gets a random field element.
   * @returns Random field element.
   */
  getRandomFr(): Fr;

  /**
   * Converts a 512 bits long buffer to a field.
   * @param uint512Buf - The buffer to convert.
   * @returns Buffer representation of the field element.
   */
  reduce512BufferToFr(uint512Buf: Buffer): Fr;

  /**
   * Point generator
   * @returns The generator for the curve.
   */
  generator(): Point;
}
