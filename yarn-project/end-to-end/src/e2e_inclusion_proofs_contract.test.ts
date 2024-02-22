import {
  AccountWallet,
  AztecAddress,
  CompleteAddress,
  EthAddress,
  Fr,
  INITIAL_L2_BLOCK_NUM,
  PXE,
  Point,
  getContractInstanceFromDeployParams,
} from '@aztec/aztec.js';
import { NewContractData } from '@aztec/circuits.js';
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
      let newNoteHashes, visibleNotes: any;
      const value = 100n;
      let validNoteBlockNumber: any;

      it('should return the correct values for creating a note', async () => {
        // Create a note
        const receipt = await contract.methods.create_note(owner, value).send().wait({ debug: true });

        noteCreationBlockNumber = receipt.blockNumber!;
        ({ newNoteHashes, visibleNotes } = receipt.debugInfo!);
      });

      it('should return the correct values for creating a note', () => {
        expect(newNoteHashes.length).toBe(1);
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

      describe('we will test the vailure case by nullifying a note', () => {
        let receipt: any;
        let currentBlockNumber: any;
        // We test the failure case now --> The proof should fail when the nullifier already exists
        it('nullifies a note and grabs block number', async () => {
          receipt = await contract.methods.nullify_note(owner).send().wait({ debug: true });
          currentBlockNumber = await pxe.getBlockNumber();

          const { newNullifiers } = receipt!.debugInfo!;
          expect(newNullifiers.length).toBe(2);
          // const nullifier = newNullifiers[1];
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
        const { newNoteHashes, visibleNotes } = receipt.debugInfo!;

        expect(newNoteHashes.length).toBe(1);
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

  describe('public value existence at a slot', () => {
    it('proves an existence of a public value in private context', async () => {
      // Choose random block number between deployment and current block number to test archival node
      const blockNumber = await getRandomBlockNumberSinceDeployment();

      await contract.methods.test_public_value_inclusion(publicValue, true, blockNumber).send().wait();
      await contract.methods.test_public_value_inclusion(publicValue, false, 0n).send().wait();
    });

    it('public value existence failure case', async () => {
      // Choose random block number between first block and current block number to test archival node
      const blockNumber = await getRandomBlockNumber();
      const randomPublicValue = Fr.random();
      await expect(
        contract.methods.test_public_value_inclusion(randomPublicValue, true, blockNumber).send().wait(),
      ).rejects.toThrow('Public value does not match the witness');
      await expect(
        contract.methods.test_public_value_inclusion(randomPublicValue, false, 0n).send().wait(),
      ).rejects.toThrow('Public value does not match the witness');
    });

    it('proves existence of uninitialized public value', async () => {
      const blockNumber = await getRandomBlockNumber();
      await contract.methods.test_public_unused_value_inclusion(blockNumber).send().wait();
    });
  });

  describe('nullifier inclusion', () => {
    it('proves existence of a nullifier in private context', async () => {
      // Choose random block number between deployment and current block number to test archival node
      const blockNumber = await getRandomBlockNumberSinceDeployment();
      const block = await pxe.getBlock(blockNumber);
      const nullifier = block?.body.txEffects[0].newNullifiers[0];

      await contract.methods.test_nullifier_inclusion(nullifier!, true, blockNumber).send().wait();
      await contract.methods.test_nullifier_inclusion(nullifier!, false, 0n).send().wait();
    });

    it('nullifier existence failure case', async () => {
      // Choose random block number between first block and current block number to test archival node
      const blockNumber = await getRandomBlockNumber();
      const randomNullifier = Fr.random();

      await expect(
        contract.methods.test_nullifier_inclusion(randomNullifier, true, blockNumber).send().wait(),
      ).rejects.toThrow(`Low nullifier witness not found for nullifier ${randomNullifier.toString()} at block`);

      await expect(contract.methods.test_nullifier_inclusion(randomNullifier, false, 0n).send().wait()).rejects.toThrow(
        `Low nullifier witness not found for nullifier ${randomNullifier.toString()} at block`,
      );
    });
  });

  describe('contract inclusion', () => {
    // InclusionProofs contract doesn't have associated public key because it's not an account contract
    const publicKey = Point.ZERO;
    let contractClassId: Fr;
    let initializationHash: Fr;
    let portalContractAddress: EthAddress;

    beforeAll(() => {
      const contractArtifact = contract.artifact;
      const constructorArgs = [publicValue];
      portalContractAddress = EthAddress.random();

      const instance = getContractInstanceFromDeployParams(
        contractArtifact,
        constructorArgs,
        contractAddressSalt,
        publicKey,
        portalContractAddress,
      );

      contractClassId = instance.contractClassId;
      initializationHash = instance.initializationHash;
    });

    it('proves existence of a contract', async () => {
      // Choose random block number between first block and current block number to test archival node
      const blockNumber = await getRandomBlockNumberSinceDeployment();

      // Note: We pass in preimage of AztecAddress instead of just AztecAddress in order for the contract to be able to
      //       test that the contract was deployed with correct constructor parameters.
      await contract.methods
        .test_contract_inclusion(
          publicKey,
          contractAddressSalt,
          contractClassId,
          initializationHash,
          portalContractAddress,
          blockNumber,
        )
        .send()
        .wait();
    });

    // TODO(@spalladino): Re-enable once we add check for non-inclusion based on nullifier
    it.skip('contract existence failure case', async () => {
      // This should fail because we choose a block number before the contract was deployed
      const blockNumber = deploymentBlockNumber - 1;
      const contractData = new NewContractData(contract.address, portalContractAddress, contractClassId);
      const leaf = contractData.hash();

      await expect(
        contract.methods
          .test_contract_inclusion(
            publicKey,
            contractAddressSalt,
            contractClassId,
            initializationHash,
            portalContractAddress,
            blockNumber,
          )
          .send()
          .wait(),
      ).rejects.toThrow(`Leaf value: ${leaf.toString()} not found in CONTRACT_TREE`);
    });
  });

  const getRandomBlockNumberSinceDeployment = async () => {
    return deploymentBlockNumber + Math.floor(Math.random() * ((await pxe.getBlockNumber()) - deploymentBlockNumber));
  };

  const getRandomBlockNumber = async () => {
    return deploymentBlockNumber + Math.floor(Math.random() * ((await pxe.getBlockNumber()) - INITIAL_L2_BLOCK_NUM));
  };
});
