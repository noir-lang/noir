import { type PeerId } from '@libp2p/interface-peer-id';

/**
 * Keeps a record of which Peers have 'seen' which transactions.
 */
export class KnownTxLookup {
  private lookup: { [key: string]: { [key: string]: boolean } } = {};

  constructor() {}

  /**
   * Inform this lookup that a peer has 'seen' a transaction.
   * @param peerId - The peerId of the peer that has 'seen' the transaction.
   * @param txHash - The thHash of the 'seen' transaction.
   */
  public addPeerForTx(peerId: PeerId, txHash: string) {
    const peerIdAsString = peerId.toString();
    const existingLookup = this.lookup[txHash];
    if (existingLookup === undefined) {
      const newLookup: { [key: string]: boolean } = {};
      newLookup[peerIdAsString] = true;
      this.lookup[txHash] = newLookup;
      return;
    }
    existingLookup[peerIdAsString] = true;
  }

  /**
   * Determine if a peer has 'seen' a transaction.
   * @param peerId - The peerId of the peer.
   * @param txHash - The thHash of the transaction.
   * @returns A boolean indicating if the transaction has been 'seen' by the peer.
   */
  public hasPeerSeenTx(peerId: PeerId, txHash: string) {
    const existingLookup = this.lookup[txHash];
    if (existingLookup === undefined) {
      return false;
    }
    const peerIdAsString = peerId.toString();
    return !!existingLookup[peerIdAsString];
  }

  /**
   * Updates the lookup from the result of settled txs
   * These txs will be cleared out of the lookup.
   * It is possible that some txs could still be gossiped for a
   * short period of time meaning they come back into this lookup
   * but this should be infrequent and cause no undesirable effects
   * @param txHashes - The hashes of the newly settled transactions
   */
  public handleSettledTxs(txHashes: string[]) {
    for (const txHash of txHashes) {
      delete this.lookup[txHash];
    }
  }
}
