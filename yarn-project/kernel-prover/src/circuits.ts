// See aztec3/constants.hpp
// Copied here for prototyping purposes

import { AztecAddress, Fr, FunctionData, TxContext } from '@aztec/circuits.js';

export class TxRequest {
  constructor(
    public readonly from: AztecAddress,
    public readonly to: AztecAddress,
    public readonly functionData: FunctionData,
    public readonly args: Fr[],
    public readonly txContext: TxContext,
    public readonly nonce: Fr,
    public readonly chainId: Fr,
  ) {}

  toBuffer() {
    return Buffer.alloc(0);
  }
}

export class Signature {
  public static SIZE = 64;

  constructor(public readonly buffer: Buffer) {}
}
