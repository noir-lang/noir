import {
  type AccountWallet,
  AztecAddress,
  type CompleteAddress,
  Fr,
  INITIAL_L2_BLOCK_NUM,
  type PXE,
  getContractInstanceFromDeployParams,
} from '@aztec/aztec.js';
import { deployInstance, registerContractClass } from '@aztec/aztec.js/deployment';
import { randomInt } from '@aztec/foundation/crypto';
import { StatefulTestContract, StatefulTestContractArtifact } from '@aztec/noir-contracts.js';
import { InclusionProofsContract } from '@aztec/noir-contracts.js/InclusionProofs';

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
  const contractAddressSalt = Fr.random();

  beforeAll(async () => {
    ({ pxe, teardown, wallets, accounts } = await setup(1));

    const receipt = await InclusionProofsContract.deploy(wallets[0], publicValue).send({ contractAddressSalt }).wait();
    contract = receipt.contract;
    deploymentBlockNumber = receipt.blockNumber!;
  }, 100_000);

  afterAll(() => teardown());

  describe('note inclusion and nullifier non-inclusion', () => {
    let owner: AztecAddress;

    beforeAll(() => {
      owner = accounts[0].address;
    });

    describe('proves note existence and its nullifier non-existence and nullifier non-existence failure case', () => {
      // Owner of a note
      let noteCreationBlockNumber: number;
      let noteHashes, visibleNotes: any;
      const value = 100n;
      let validNoteBlockNumber: any;

      it('should return the correct values for creating a note', async () => {
        // Create a note
        const receipt = await contract.methods.create_note(owner, value).send().wait({ debug: true });

        noteCreationBlockNumber = receipt.blockNumber!;
        ({ noteHashes, visibleNotes } = receipt.debugInfo!);
      });

      it('should return the correct values for creating a note', () => {
        expect(noteHashes.length).toBe(1);
        expect(visibleNotes.length).toBe(1);
        const [receivedValue, receivedOwner, _randomness] = visibleNotes[0].note.items;
        expect(receivedValue.toBigInt()).toBe(value);
        expect(receivedOwner).toEqual(owner.toField());
      });

      it('should not throw because the note is included', async () => {
        // Prove note inclusion in a given block.
        await contract.methods.test_note_inclusion(owner, true, noteCreationBlockNumber, false).send().wait();

        await contract.methods.test_note_inclusion(owner, false, 0n, false).send().wait();
      });

      it('should not throw because the note is not nullified', async () => {
        // Prove that the note has not been nullified with block_number
        // TODO(#3535): Prove the nullifier non-inclusion at older block to test archival node. This is currently not
        // possible because of issue https://github.com/AztecProtocol/aztec-packages/issues/3535
        const blockNumber = await pxe.getBlockNumber();
        await contract.methods.test_note_not_nullified(owner, true, blockNumber, false).send().wait();
        await contract.methods.test_note_not_nullified(owner, false, 0n, false).send().wait();
      });

      it('should not throw because is both included, not nullified, and therefore valid', async () => {
        validNoteBlockNumber = await pxe.getBlockNumber();
        await contract.methods.test_note_validity(owner, true, validNoteBlockNumber, false).send().wait();
        await contract.methods.test_note_validity(owner, false, 0n, false).send().wait();
      });

      describe('we will test the failure case by nullifying a note', () => {
        let currentBlockNumber: number;

        // We test the failure case now --> The proof should fail when the nullifier already exists
        it('nullifies a note and grabs block number', async () => {
          const { debugInfo } = await contract.methods.nullify_note(owner).send().wait({ debug: true });
          currentBlockNumber = await pxe.getBlockNumber();

          expect(debugInfo!.nullifiers.length).toBe(2);
        });

        // Note: getLowNullifierMembershipWitness returns the membership witness of the nullifier itself and not
        // the low nullifier when the nullifier already exists in the tree and for this reason the execution fails
        // on low_nullifier.value < nullifier.value check.
        it('should throw when testing if note is not nullified at the current block', async () => {
          await expect(
            contract.methods.test_note_not_nullified(owner, true, currentBlockNumber, true).send().wait(),
          ).rejects.toThrow(
            /Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed/,
          );
          await expect(contract.methods.test_note_not_nullified(owner, false, 0n, true).send().wait()).rejects.toThrow(
            /Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed/,
          );
        });

        it('should not throw when we test inclusion of nullified note', async () => {
          await contract.methods.test_note_inclusion(owner, true, noteCreationBlockNumber, true).send().wait();

          await contract.methods.test_note_inclusion(owner, false, 0n, true).send().wait();
        });

        it('should throw when we test validity', async () => {
          const blockNumber = await pxe.getBlockNumber();
          await expect(
            contract.methods.test_note_validity(owner, true, blockNumber, true).send().wait(),
          ).rejects.toThrow(
            /Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed/,
          );
          await expect(contract.methods.test_note_validity(owner, false, 0n, true).send().wait()).rejects.toThrow(
            /Proving nullifier non-inclusion failed: low_nullifier.value < nullifier.value check failed/,
          );
        });

        it('should not throw because the note was not nullified yet at validNoteBlockNumber', async () => {
          await contract.methods.test_note_not_nullified(owner, true, validNoteBlockNumber, true).send().wait();
          await contract.methods.test_note_validity(owner, true, validNoteBlockNumber, true).send().wait();
        });
      });
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
        const { noteHashes, visibleNotes } = receipt.debugInfo!;

        expect(noteHashes.length).toBe(1);
        expect(visibleNotes.length).toBe(1);
        const [receivedValue, receivedOwner, _randomness] = visibleNotes[0].note.items;
        expect(receivedValue.toBigInt()).toBe(value);
        expect(receivedOwner).toEqual(owner.toField());
      }

      {
        // Prove note validity
        await contract.methods.test_note_validity(owner, true, noteCreationBlockNumber, false).send().wait();
        await contract.methods.test_note_validity(owner, false, 0n, false).send().wait();
      }
    });

    it('note existence failure case', async () => {
      // Owner of a note - ignored in the contract since the note won't be found and the spare random note commitment
      // will be used instead
      const owner = AztecAddress.fromField(new Fr(88n));

      // Choose random block number between deployment and current block number to test archival node
      const blockNumber = await getRandomBlockNumberSinceDeployment();

      await expect(
        contract.methods.test_note_inclusion_fail_case(owner, true, blockNumber).send().wait(),
      ).rejects.toThrow(/Leaf value: .* not found in NOTE_HASH_TREE/);

      await expect(contract.methods.test_note_inclusion_fail_case(owner, false, 0n).send().wait()).rejects.toThrow(
        /Leaf value: .* not found in NOTE_HASH_TREE/,
      );
    });
  });

  describe('historical storage reads', () => {
    it('reads a historical public value in private context', async () => {
      // Choose random block number between deployment and current block number to test archival node
      const blockNumber = await getRandomBlockNumberSinceDeployment();

      await contract.methods.test_storage_historical_read(publicValue, true, blockNumber).send().wait();
      await contract.methods.test_storage_historical_read(publicValue, false, 0n).send().wait();
    });

    it('reads an older (unset) public value', async () => {
      const blockNumber = getRandomBlockNumberBeforeDeployment();
      await contract.methods.test_storage_historical_read(0, true, blockNumber).send().wait();
    });

    it('reads a historical unset public value in private context', async () => {
      // This test scenario is interesting because the codepath for storage values that were never set is different
      // (since they don't exist in the tree).
      const blockNumber = await getRandomBlockNumber();
      await contract.methods.test_storage_historical_read_unset_slot(blockNumber).send().wait();
    });
  });

  describe('nullifier inclusion', () => {
    it('proves existence of a nullifier in private context', async () => {
      // Choose random block number between deployment and current block number to test archival node
      const blockNumber = await getRandomBlockNumberSinceDeployment();
      const block = await pxe.getBlock(blockNumber);
      const nullifier = block?.body.txEffects[0].nullifiers[0];

      await contract.methods.test_nullifier_inclusion(nullifier!, true, blockNumber).send().wait();
      await contract.methods.test_nullifier_inclusion(nullifier!, false, 0n).send().wait();
    });

    it('proves existence of a nullifier in public context', async () => {
      const block = await pxe.getBlock(deploymentBlockNumber);
      const nullifier = block?.body.txEffects[0].nullifiers[0];

      await contract.methods.test_nullifier_inclusion_from_public(nullifier!).send().wait();
    });

    it('nullifier existence failure case', async () => {
      // Choose random block number between first block and current block number to test archival node
      const blockNumber = await getRandomBlockNumber();
      const randomNullifier = Fr.random();

      await expect(
        contract.methods.test_nullifier_inclusion(randomNullifier, true, blockNumber).send().wait(),
      ).rejects.toThrow(`Nullifier witness not found for nullifier ${randomNullifier.toString()} at block`);

      await expect(contract.methods.test_nullifier_inclusion(randomNullifier, false, 0n).send().wait()).rejects.toThrow(
        `Nullifier witness not found for nullifier ${randomNullifier.toString()} at block`,
      );
    });
  });

  describe('contract inclusion', () => {
    const assertInclusion = async (
      address: AztecAddress,
      blockNumber: number,
      opts: { testDeploy: boolean; testInit: boolean },
    ) => {
      const { testDeploy, testInit } = opts;
      // Assert contract was publicly deployed or initialized in the block in which it was deployed
      await contract.methods.test_contract_inclusion(address, blockNumber, testDeploy, testInit).send().wait();

      // And prove that it was not before that
      const olderBlock = blockNumber - 2;
      await contract.methods.test_contract_non_inclusion(address, olderBlock, testDeploy, testInit).send().wait();

      // Or that the positive call fails when trying to prove in the older block
      await expect(
        contract.methods.test_contract_inclusion(address, olderBlock, testDeploy, testInit).prove(),
      ).rejects.toThrow(/not found/);
    };

    it('proves public deployment of a contract', async () => {
      // Publicly deploy another contract (so we don't test on the same contract)
      const initArgs = [accounts[0], 42n];
      const instance = getContractInstanceFromDeployParams(StatefulTestContractArtifact, { constructorArgs: initArgs });
      await (await registerContractClass(wallets[0], StatefulTestContractArtifact)).send().wait();
      const receipt = await deployInstance(wallets[0], instance).send().wait();

      await assertInclusion(instance.address, receipt.blockNumber!, { testDeploy: true, testInit: false });
    });

    it('proves initialization of a contract', async () => {
      // Initialize (but not deploy) a test contract
      const receipt = await StatefulTestContract.deploy(wallets[0], accounts[0], 42n)
        .send({ skipClassRegistration: true, skipPublicDeployment: true })
        .wait();

      await assertInclusion(receipt.contract.address, receipt.blockNumber!, { testDeploy: false, testInit: true });
    });
  });

  const getRandomBlockNumberSinceDeployment = async () => {
    return deploymentBlockNumber + randomInt((await pxe.getBlockNumber()) - deploymentBlockNumber);
  };

  const getRandomBlockNumber = async () => {
    return deploymentBlockNumber + randomInt((await pxe.getBlockNumber()) - INITIAL_L2_BLOCK_NUM);
  };

  const getRandomBlockNumberBeforeDeployment = () => {
    return randomInt(deploymentBlockNumber - INITIAL_L2_BLOCK_NUM) + INITIAL_L2_BLOCK_NUM;
  };
});
