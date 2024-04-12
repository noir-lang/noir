import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type Hasher } from '@aztec/types/interfaces';

/**
 * A helper class encapsulating Pedersen hash functionality.
 * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
 * purposes.
 */
export class Pedersen implements Hasher {
  /*
   * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
   * purposes.
   */
  public hash(lhs: Uint8Array, rhs: Uint8Array): Buffer {
    return pedersenHash([Fr.fromBuffer(Buffer.from(lhs)), Fr.fromBuffer(Buffer.from(rhs))]).toBuffer();
  }

  /*
   * @deprecated Don't call pedersen directly in production code. Instead, create suitably-named functions for specific
   * purposes.
   */
  public hashInputs(inputs: Buffer[]): Buffer {
    const inputFields = inputs.map(i => Fr.fromBuffer(i));
    return pedersenHash(inputFields).toBuffer();
  }
}
