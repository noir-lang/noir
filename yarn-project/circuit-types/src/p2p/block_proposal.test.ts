// Serde test for the block proposal type
import { BlockProposal } from './block_proposal.js';
import { makeBlockProposal, randomSigner } from './mocks.js';

describe('Block Proposal serialization / deserialization', () => {
  it('Should serialize / deserialize', async () => {
    const proposal = await makeBlockProposal();

    const serialized = proposal.toBuffer();
    const deserialized = BlockProposal.fromBuffer(serialized);

    expect(deserialized).toEqual(proposal);
  });

  it('Should serialize / deserialize + recover sender', async () => {
    const account = randomSigner();

    const proposal = await makeBlockProposal(account);
    const serialized = proposal.toBuffer();
    const deserialized = BlockProposal.fromBuffer(serialized);

    expect(deserialized).toEqual(proposal);

    // Recover signature
    const sender = await deserialized.getSender();
    expect(sender.toChecksumString()).toEqual(account.address);
  });
});
