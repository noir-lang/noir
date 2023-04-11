import {
  pedersenCompress,
  pedersenCompressInputs,
  pedersenGetHash,
  pedersenGetHashTree,
} from '@aztec/barretenberg.js/crypto';

import { WasmWrapper } from '@aztec/foundation/wasm';
import { Hasher } from './hasher.js';

export class Pedersen implements Hasher {
  constructor(private wasm: WasmWrapper) {}

  public compress(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    return pedersenCompress(this.wasm, lhs, rhs);
  }
  public compressInputs(inputs: Buffer[]): Buffer {
    return pedersenCompressInputs(this.wasm, inputs);
  }
  public hashToField(data: Uint8Array): Buffer {
    return pedersenGetHash(this.wasm, Buffer.from(data));
  }
  public hashToTree(leaves: Buffer[]): Promise<Buffer[]> {
    return Promise.resolve(pedersenGetHashTree(this.wasm, leaves));
  }
}
