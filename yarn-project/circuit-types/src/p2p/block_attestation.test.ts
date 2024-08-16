// Serde test for the block attestation type
import { BlockAttestation } from './block_attestation.js';
import { makeBlockAttestation, randomSigner } from './mocks.js';

describe('Block Attestation serialization / deserialization', () => {
  it('Should serialize / deserialize', async () => {
    const attestation = await makeBlockAttestation();

    const serialized = attestation.toBuffer();
    const deserialized = BlockAttestation.fromBuffer(serialized);

    expect(deserialized).toEqual(attestation);
  });

  it('Should serialize / deserialize + recover sender', async () => {
    const account = randomSigner();

    const proposal = await makeBlockAttestation(account);
    const serialized = proposal.toBuffer();
    const deserialized = BlockAttestation.fromBuffer(serialized);

    expect(deserialized).toEqual(proposal);

    // Recover signature
    const sender = await deserialized.getSender();
    expect(sender.toChecksumString()).toEqual(account.address);
  });
});
