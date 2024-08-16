import { type PrivateKeyAccount } from 'viem';

import { InMemoryAttestationPool } from './memory_attestation_pool.js';
import { generateAccount, mockAttestation } from './mocks.js';

const NUMBER_OF_SIGNERS_PER_TEST = 4;

describe('MemoryAttestationPool', () => {
  let ap: InMemoryAttestationPool;
  let signers: PrivateKeyAccount[];

  beforeEach(() => {
    ap = new InMemoryAttestationPool();
    signers = Array.from({ length: NUMBER_OF_SIGNERS_PER_TEST }, generateAccount);
  });

  it('should add attestation to pool', async () => {
    const slotNumber = 420;
    const attestations = await Promise.all(signers.map(signer => mockAttestation(signer, slotNumber)));

    await ap.addAttestations(attestations);

    const retreivedAttestations = await ap.getAttestationsForSlot(BigInt(slotNumber));

    expect(retreivedAttestations.length).toBe(NUMBER_OF_SIGNERS_PER_TEST);
    expect(retreivedAttestations).toEqual(attestations);

    // Delete by slot
    await ap.deleteAttestationsForSlot(BigInt(slotNumber));

    const retreivedAttestationsAfterDelete = await ap.getAttestationsForSlot(BigInt(slotNumber));
    expect(retreivedAttestationsAfterDelete.length).toBe(0);
  });

  it('Should store attestations by differing slot', async () => {
    const slotNumbers = [1, 2, 3, 4];
    const attestations = await Promise.all(signers.map((signer, i) => mockAttestation(signer, slotNumbers[i])));

    await ap.addAttestations(attestations);

    for (const attestation of attestations) {
      const slot = attestation.header.globalVariables.slotNumber;

      const retreivedAttestations = await ap.getAttestationsForSlot(slot.toBigInt());
      expect(retreivedAttestations.length).toBe(1);
      expect(retreivedAttestations[0]).toEqual(attestation);
      expect(retreivedAttestations[0].header.globalVariables.slotNumber).toEqual(slot);
    }
  });

  it('Should delete attestations', async () => {
    const slotNumber = 420;
    const attestations = await Promise.all(signers.map(signer => mockAttestation(signer, slotNumber)));

    await ap.addAttestations(attestations);

    const retreivedAttestations = await ap.getAttestationsForSlot(BigInt(slotNumber));
    expect(retreivedAttestations.length).toBe(NUMBER_OF_SIGNERS_PER_TEST);
    expect(retreivedAttestations).toEqual(attestations);

    await ap.deleteAttestations(attestations);

    const gottenAfterDelete = await ap.getAttestationsForSlot(BigInt(slotNumber));
    expect(gottenAfterDelete.length).toBe(0);
  });
});
