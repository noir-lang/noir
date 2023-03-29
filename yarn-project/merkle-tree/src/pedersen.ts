import { pedersenCompress, pedersenGetHash, pedersenGetHashTree } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';

import { Hasher } from './hasher.js';

export class Pedersen implements Hasher {
  constructor(private wasm: BarretenbergWasm) {}

  public compress(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    return pedersenCompress(this.wasm, lhs, rhs);
  }
  public hashToField(data: Uint8Array): Buffer {
    return pedersenGetHash(this.wasm, Buffer.from(data));
  }
  public hashToTree(leaves: Buffer[]): Promise<Buffer[]> {
    return Promise.resolve(pedersenGetHashTree(this.wasm, leaves));
  }
}
