import {
  pedersenCompress,
  pedersenGetHash,
  pedersenGetHashTree,
  pedersenHashInputs,
} from '@aztec/circuits.js/barretenberg';
import { IWasmModule } from '@aztec/foundation/wasm';
import { Hasher } from '@aztec/types';

/**
 * A helper class encapsulating Pedersen hash functionality.
 */
export class Pedersen implements Hasher {
  constructor(private wasm: IWasmModule) {}

  public compress(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    return pedersenCompress(this.wasm, lhs, rhs);
  }

  public compressInputs(inputs: Buffer[]): Buffer {
    return pedersenHashInputs(this.wasm, inputs);
  }

  public hashToField(data: Uint8Array): Buffer {
    return pedersenGetHash(this.wasm, Buffer.from(data));
  }

  public hashToTree(leaves: Buffer[]): Promise<Buffer[]> {
    return Promise.resolve(pedersenGetHashTree(this.wasm, leaves));
  }
}
