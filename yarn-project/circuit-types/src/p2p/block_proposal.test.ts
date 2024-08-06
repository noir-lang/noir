// Serde test for the block proposal type
import { makeHeader } from '@aztec/circuits.js/testing';

import { TxHash } from '../index.js';
import { BlockProposal } from './block_proposal.js';

describe('Block Proposal serialization / deserialization', () => {
  const makeBlockProposal = (): BlockProposal => {
    const blockHeader = makeHeader(1);
    const txs = [0, 1, 2, 3, 4, 5].map(() => TxHash.random());
    const signature = Buffer.alloc(64, 1);

    return new BlockProposal(blockHeader, txs, signature);
  };

  it('Should serialize / deserialize', () => {
    const proposal = makeBlockProposal();

    const serialized = proposal.toBuffer();
    const deserialized = BlockProposal.fromBuffer(serialized);

    expect(deserialized).toEqual(proposal);
  });
});
