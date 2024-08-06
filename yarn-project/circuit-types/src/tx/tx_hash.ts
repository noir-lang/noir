import { BaseHashType } from '@aztec/foundation/hash';

/**
 * A class representing hash of Aztec transaction.
 */
export class TxHash extends BaseHashType {
  constructor(
    /**
     * The buffer containing the hash.
     */
    hash: Buffer,
  ) {
    super(hash);
  }
}
