import { randomTxHash } from '@aztec/circuit-types';

import { expect } from '@jest/globals';
import { type Ed25519PeerId, type PeerId } from '@libp2p/interface-peer-id';
import { mock } from 'jest-mock-extended';

import { KnownTxLookup } from './known_txs.js';

const createMockPeerId = (peerId: string): PeerId => {
  return mock<Ed25519PeerId>({
    toString: () => peerId,
  });
};

describe('Known Txs', () => {
  it('Returns false when a peer has not seen a tx', () => {
    const knownTxs = new KnownTxLookup();

    const peer = createMockPeerId('Peer 1');
    const txHash = randomTxHash();

    expect(knownTxs.hasPeerSeenTx(peer, txHash.toString())).toEqual(false);
  });

  it('Returns true when a peer has seen a tx', () => {
    const knownTxs = new KnownTxLookup();

    const peer = createMockPeerId('Peer 1');
    const peer2 = createMockPeerId('Peer 2');
    const txHash = randomTxHash();

    knownTxs.addPeerForTx(peer, txHash.toString());

    expect(knownTxs.hasPeerSeenTx(peer, txHash.toString())).toEqual(true);
    expect(knownTxs.hasPeerSeenTx(peer2, txHash.toString())).toEqual(false);

    knownTxs.addPeerForTx(peer2, txHash.toString());

    expect(knownTxs.hasPeerSeenTx(peer, txHash.toString())).toEqual(true);
    expect(knownTxs.hasPeerSeenTx(peer2, txHash.toString())).toEqual(true);
  });
});
