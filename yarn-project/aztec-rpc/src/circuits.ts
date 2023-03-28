import { AztecAddress, Fr } from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation';

export class Signature {
  public static SIZE = 64;

  public static random() {
    return new Signature(randomBytes(Signature.SIZE));
  }

  constructor(public readonly buffer: Buffer) {}
}

export function generateContractAddress(
  deployerAddress: AztecAddress,
  salt: Fr,
  args: Fr[],
  // functionLeaves: Fr[],
) {
  return AztecAddress.random();
}

export function selectorToNumber(selector: Buffer) {
  return selector.readUInt32BE();
}
