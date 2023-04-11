import { Fr, keccak } from '@aztec/foundation';
import { TxHash } from './tx_hash.js';

interface TxData {
  newCommitments: Fr[];
  newNullifiers: Fr[];
  newContracts: Fr[];
}

/**
 * Utility function to generate tx hash.
 * @param txData - The data from which to generate the hash.
 * @returns A hash of the tx data that identifies the tx.
 */
export function createTxHash({ newCommitments, newNullifiers, newContracts }: TxData) {
  const data = Buffer.concat(
    [
      newCommitments.map(x => x.toBuffer()),
      newNullifiers.map(x => x.toBuffer()),
      newContracts.map(x => x.toBuffer()),
    ].flat(),
  );
  return new TxHash(keccak(data));
}
