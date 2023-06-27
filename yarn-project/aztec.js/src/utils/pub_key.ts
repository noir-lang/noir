import { Point } from '../index.js';

/**
 * Converts a Point type to a public key represented by BigInt coordinates
 * @param point - The Point to convert.
 * @returns An object with x & y coordinates represented as bigints.
 */
export function pointToPublicKey(point: Point) {
  const x = point.x.toBigInt();
  const y = point.y.toBigInt();
  return {
    x,
    y,
  };
}
