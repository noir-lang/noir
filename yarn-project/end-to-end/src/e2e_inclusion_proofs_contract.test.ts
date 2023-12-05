import { AccountWallet, AztecAddress, CompleteAddress, Fr, PXE } from '@aztec/aztec.js';
import { InclusionProofsContract } from '@aztec/noir-contracts/types';

import { jest } from '@jest/globals';
import { type MemDown, default as memdown } from 'memdown';

import { setup } from './fixtures/utils.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const TIMEOUT = 90_000;

describe('e2e_inclusion_proofs_contract', () => {
  jest.setTimeout(TIMEOUT);

  let pxe: PXE;
  let teardown: () => Promise<void>;
  let wallets: AccountWallet[];
  let accounts: CompleteAddress[];

  let contract: InclusionProofsContract;
  const publicValue = 236n;

  beforeAll(async () => {
    ({ pxe, teardown, wallets, accounts } = await setup(1));

    contract = await InclusionProofsContract.deploy(wallets[0], publicValue).send().deployed();
  }, 100_000);

  afterAll(() => teardown());

  it('proves note existence and its nullifier non-existence and nullifier non-existence failure case', async () => {
    // Owner of a note
    const owner = accounts[0].address;
    {
      // Create a note
      const value = 100n;
      const receipt = await contract.methods.create_note(owner, value).send().wait({ debug: true });
      const { newCommitments, visibleNotes } = receipt.debugInfo!;
      expect(newCommitments.length).toBe(1);
      expect(visibleNotes.length).toBe(1);
      const [receivedValue, receivedOwner, _randomness] = visibleNotes[0].note.items;
      expect(receivedValue.toBigInt()).toBe(value);
      expect(receivedOwner).toEqual(owner.toField());
    }

    {
      // Prove note inclusion in a given block.
      // TODO: Use here note block number from the creation note tx to test archival node. This is currently not
      // possible because of issue #3564
      const blockNumber = await pxe.getBlockNumber();
      const ignoredCommitment = 0; // Not ignored only when the note doesn't exist
      await contract.methods.proveNoteInclusion(owner, blockNumber, ignoredCommitment).send().wait();
    }

    {
      // Prove that the note has not been nullified
      // TODO: Use here note block number from the creation note tx to test archival node. This is currently not
      // possible because of issue #3564
      const blockNumber = await pxe.getBlockNumber();
      const ignoredNullifier = 0; // Not ignored only when the note doesn't exist
      await contract.methods.proveNullifierNonInclusion(owner, blockNumber, ignoredNullifier).send().wait();
    }

    {
      // We test the failure case now --> The proof should fail when the nullifier already exists
      const receipt = await contract.methods.nullifyNote(owner).send().wait({ debug: true });
      const { newNullifiers } = receipt.debugInfo!;
      expect(newNullifiers.length).toBe(2);

      const blockNumber = await pxe.getBlockNumber();
      const nullifier = newNullifiers[1];
      // Note: getLowNullifierMembershipWitness returns the membership witness of the nullifier itself and not
      // the low nullifier when the nullifier already exists in the tree and for this reason the execution fails
      // on low_nullifier.value < nullifier.value check.
      await expect(
        contract.methods.proveNullifierNonInclusion(owner, blockNumber, nullifier).send().wait(),
      ).rejects.toThrowError(
        /Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed/,
      );
    }
  });

  it('note existence failure case', async () => {
    // Owner of a note
    const owner = AztecAddress.random();

    const blockNumber = await pxe.getBlockNumber();
    const randomNoteCommitment = Fr.random();
    await expect(
      contract.methods.proveNoteInclusion(owner, blockNumber, randomNoteCommitment).send().wait(),
    ).rejects.toThrow(/Leaf value: 0x[0-9a-fA-F]+ not found in NOTE_HASH_TREE/);
  });

  it('proves an existence of a public value in private context', async () => {
    const blockNumber = await pxe.getBlockNumber();
    await contract.methods.provePublicValueInclusion(publicValue, blockNumber).send().wait();
  });

  it('public value existence failure case', async () => {
    const blockNumber = await pxe.getBlockNumber();
    const randomPublicValue = Fr.random();
    await expect(
      contract.methods.provePublicValueInclusion(randomPublicValue, blockNumber).send().wait(),
    ).rejects.toThrow(/Proving public value inclusion failed/);
  });

  it('proves existence of a nullifier in private context', async () => {
    const blockNumber = await pxe.getBlockNumber();
    const block = await pxe.getBlock(blockNumber);
    const nullifier = block?.newNullifiers[0];

    await contract.methods.proveNullifierInclusion(nullifier!, blockNumber).send().wait();
  });

  it('nullifier existence failure case', async () => {
    const blockNumber = await pxe.getBlockNumber();
    const randomNullifier = Fr.random();

    await expect(contract.methods.proveNullifierInclusion(randomNullifier, blockNumber).send().wait()).rejects.toThrow(
      /Low nullifier witness not found for nullifier 0x[0-9a-fA-F]+ at block/,
    );
  });
});
