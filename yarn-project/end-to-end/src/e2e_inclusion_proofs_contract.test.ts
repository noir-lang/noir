import { AccountWallet, AztecAddress, CompleteAddress, Fr, INITIAL_L2_BLOCK_NUM, PXE } from '@aztec/aztec.js';
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
  let deploymentBlockNumber: number;
  const publicValue = 236n;

  beforeAll(async () => {
    ({ pxe, teardown, wallets, accounts } = await setup(1));

    const receipt = await InclusionProofsContract.deploy(wallets[0], publicValue).send().wait();
    contract = receipt.contract;
    deploymentBlockNumber = receipt.blockNumber!;
  }, 100_000);

  afterAll(() => teardown());

  it('proves note existence and its nullifier non-existence and nullifier non-existence failure case', async () => {
    // Owner of a note
    const owner = accounts[0].address;
    let noteCreationBlockNumber: number;
    {
      // Create a note
      const value = 100n;
      const receipt = await contract.methods.create_note(owner, value).send().wait({ debug: true });

      noteCreationBlockNumber = receipt.blockNumber!;
      const { newCommitments, visibleNotes } = receipt.debugInfo!;

      expect(newCommitments.length).toBe(1);
      expect(visibleNotes.length).toBe(1);
      const [receivedValue, receivedOwner, _randomness] = visibleNotes[0].note.items;
      expect(receivedValue.toBigInt()).toBe(value);
      expect(receivedOwner).toEqual(owner.toField());
    }

    {
      // Prove note inclusion in a given block.
      const ignoredCommitment = 0; // Not ignored only when the note doesn't exist
      await contract.methods.test_note_inclusion_proof(owner, noteCreationBlockNumber, ignoredCommitment).send().wait();
    }

    {
      // Prove that the note has not been nullified
      // TODO(#3535): Prove the nullifier non-inclusion at older block to test archival node. This is currently not
      // possible because of issue https://github.com/AztecProtocol/aztec-packages/issues/3535
      const blockNumber = await pxe.getBlockNumber();
      const ignoredNullifier = 0; // Not ignored only when the note doesn't exist
      await contract.methods.test_nullifier_non_inclusion_proof(owner, blockNumber, ignoredNullifier).send().wait();
    }

    {
      // We test the failure case now --> The proof should fail when the nullifier already exists
      const receipt = await contract.methods.nullify_note(owner).send().wait({ debug: true });
      const { newNullifiers } = receipt.debugInfo!;
      expect(newNullifiers.length).toBe(2);

      const blockNumber = await pxe.getBlockNumber();
      const nullifier = newNullifiers[1];
      // Note: getLowNullifierMembershipWitness returns the membership witness of the nullifier itself and not
      // the low nullifier when the nullifier already exists in the tree and for this reason the execution fails
      // on low_nullifier.value < nullifier.value check.
      await expect(
        contract.methods.test_nullifier_non_inclusion_proof(owner, blockNumber, nullifier).send().wait(),
      ).rejects.toThrowError(
        /Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed/,
      );
    }
  });

  it('proves note validity (note commitment inclusion and nullifier non-inclusion)', async () => {
    // Owner of a note
    const owner = accounts[0].address;
    let noteCreationBlockNumber: number;
    {
      // Create a note
      const value = 100n;
      const receipt = await contract.methods.create_note(owner, value).send().wait({ debug: true });

      noteCreationBlockNumber = receipt.blockNumber!;
      const { newCommitments, visibleNotes } = receipt.debugInfo!;

      expect(newCommitments.length).toBe(1);
      expect(visibleNotes.length).toBe(1);
      const [receivedValue, receivedOwner, _randomness] = visibleNotes[0].note.items;
      expect(receivedValue.toBigInt()).toBe(value);
      expect(receivedOwner).toEqual(owner.toField());
    }

    {
      // Prove note validity
      await contract.methods.test_note_validity_proof(owner, noteCreationBlockNumber).send().wait();
    }
  });

  it('note existence failure case', async () => {
    // Owner of a note - ignored in the contract since the note won't be found and the spare random note commitment
    // will be used instead
    const owner = AztecAddress.random();

    // Choose random block number between deployment and current block number to test archival node
    const blockNumber = await getRandomBlockNumberSinceDeployment();
    const randomNoteCommitment = Fr.random();
    await expect(
      contract.methods.test_note_inclusion_proof(owner, blockNumber, randomNoteCommitment).send().wait(),
    ).rejects.toThrow(/Leaf value: 0x[0-9a-fA-F]+ not found in NOTE_HASH_TREE/);
  });

  it('proves an existence of a public value in private context', async () => {
    // Choose random block number between deployment and current block number to test archival node
    const blockNumber = await getRandomBlockNumberSinceDeployment();

    await contract.methods.test_public_value_inclusion_proof(publicValue, blockNumber).send().wait();
  });

  it('public value existence failure case', async () => {
    // Choose random block number between first block and current block number to test archival node
    const blockNumber = await getRandomBlockNumber();

    const randomPublicValue = Fr.random();
    await expect(
      contract.methods.test_public_value_inclusion_proof(randomPublicValue, blockNumber).send().wait(),
    ).rejects.toThrow(/Public value does not match value in witness/);
  });

  it('proves existence of a nullifier in private context', async () => {
    // Choose random block number between deployment and current block number to test archival node
    const blockNumber = await getRandomBlockNumberSinceDeployment();
    const block = await pxe.getBlock(blockNumber);
    const nullifier = block?.newNullifiers[0];

    await contract.methods.test_nullifier_inclusion_proof(nullifier!, blockNumber).send().wait();
  });

  it('nullifier existence failure case', async () => {
    // Choose random block number between first block and current block number to test archival node
    const blockNumber = await getRandomBlockNumber();
    const randomNullifier = Fr.random();

    await expect(
      contract.methods.test_nullifier_inclusion_proof(randomNullifier, blockNumber).send().wait(),
    ).rejects.toThrow(/Low nullifier witness not found for nullifier 0x[0-9a-fA-F]+ at block/);
  });

  const getRandomBlockNumberSinceDeployment = async () => {
    const currentBlockNumber = await pxe.getBlockNumber();
    return deploymentBlockNumber + Math.floor(Math.random() * (currentBlockNumber - deploymentBlockNumber));
  };

  const getRandomBlockNumber = async () => {
    const currentBlockNumber = await pxe.getBlockNumber();
    return deploymentBlockNumber + Math.floor(Math.random() * (currentBlockNumber - INITIAL_L2_BLOCK_NUM));
  };
});
