import { Grumpkin } from '@aztec/circuits.js/barretenberg';
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

/**
 * Method for generating a public grumpkin key from a private key.
 * @param privateKey - The private key.
 * @returns The generated public key.
 */
export async function generatePublicKey(privateKey: Buffer): Promise<Point> {
  const grumpkin = await Grumpkin.new();
  return Point.fromBuffer(grumpkin.mul(grumpkin.generator(), privateKey));
}
