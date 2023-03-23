/* eslint-disable */
import { Hasher } from './hasher.js';
import { default as sha256 } from 'sha256';

export class Pedersen implements Hasher {
  public compress(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    return Buffer.from(sha256(Buffer.concat([lhs, rhs])), 'hex');
  }
  public hashToField(data: Uint8Array): Buffer {
    return Buffer.from(sha256(Buffer.from(data)), 'hex');
  }
  public hashToTree(leaves: Buffer[]): Promise<Buffer[]> {
    return Promise.resolve([Buffer.alloc(32)]);
  }
}
