import {
  type AztecAddress,
  type AztecNode,
  type CompleteAddress,
  type DebugLogger,
  Fr,
  type Wallet,
} from '@aztec/aztec.js';
import { PendingNoteHashesContract } from '@aztec/noir-contracts.js/PendingNoteHashes';

import { setup } from './fixtures/utils.js';

describe('e2e_pending_note_hashes_contract', () => {
  let aztecNode: AztecNode;
  let wallet: Wallet;
  let logger: DebugLogger;
  let owner: AztecAddress;
  let teardown: () => Promise<void>;
  let contract: PendingNoteHashesContract;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ teardown, aztecNode, accounts, wallet, logger } = await setup(2));
    owner = accounts[0].address;
  }, 100_000);

  afterEach(() => teardown());

  const expectNoteHashesSquashedExcept = async (exceptFirstFew: number) => {
    const blockNum = await aztecNode.getBlockNumber();
    const block = (await aztecNode.getBlocks(blockNum, 1))[0];

    const noteHashes = block.body.txEffects.flatMap(txEffect => txEffect.noteHashes);

    // all new note hashes should be zero (should be squashed)
    for (let c = 0; c < exceptFirstFew; c++) {
      expect(noteHashes[c]).not.toEqual(Fr.ZERO);
    }

    for (let c = exceptFirstFew; c < noteHashes.length; c++) {
      expect(noteHashes[c]).toEqual(Fr.ZERO);
    }
  };

  const expectNullifiersSquashedExcept = async (exceptFirstFew: number) => {
    const blockNum = await aztecNode.getBlockNumber();
    const block = (await aztecNode.getBlocks(blockNum, 1))[0];

    const nullifierArray = block.body.txEffects.flatMap(txEffect => txEffect.nullifiers);

    // 0th nullifier should be nonzero (txHash), all others should be zero (should be squashed)
    for (let n = 0; n < exceptFirstFew + 1; n++) {
      logger(`Expecting nullifier ${n} to be nonzero`);
      expect(nullifierArray[n]).not.toEqual(Fr.ZERO); // 0th nullifier is txHash
    }
    for (let n = exceptFirstFew + 1; n < nullifierArray.length; n++) {
      expect(nullifierArray[n]).toEqual(Fr.ZERO);
    }
  };

  const deployContract = async () => {
    logger(`Deploying L2 contract...`);
    contract = await PendingNoteHashesContract.deploy(wallet).send().deployed();
    logger('L2 contract deployed');
    return contract;
  };

  it('Aztec.nr function can "get" notes it just "inserted"', async () => {
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    await deployedContract.methods.test_insert_then_get_then_nullify_flat(mintAmount, owner).send().wait();
  }, 60_000);

  it('Squash! Aztec.nr function can "create" and "nullify" note in the same TX', async () => {
    // Kernel will squash the noteHash and its nullifier.
    // Realistic way to describe this test is "Mint note A, then burn note A in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    await deployedContract.methods
      .test_insert_then_get_then_nullify_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();
    await expect(deployedContract.methods.get_note_zero_balance(owner).send().wait()).rejects.toThrow(
      `Assertion failed: Cannot return zero notes`,
    );

    await expectNoteHashesSquashedExcept(0);
    await expectNullifiersSquashedExcept(0);
  }, 60_000);

  it('Squash! Aztec.nr function can "create" 2 notes and "nullify" both in the same TX', async () => {
    // Kernel will squash both noteHashes and their nullifier.
    // Realistic way to describe this test is "Mint notes A and B, then burn both in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    await deployedContract.methods
      .test_insert2_then_get2_then_nullify2_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();

    await expectNoteHashesSquashedExcept(0);
    await expectNullifiersSquashedExcept(0);
  }, 60_000);

  it('Squash! Aztec.nr function can "create" 2 notes and "nullify" 1 in the same TX (kernel will squash one note + nullifier)', async () => {
    // Kernel will squash one noteHash and its nullifier.
    // The other note will become persistent!
    // Realistic way to describe this test is "Mint notes A and B, then burn note A in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    await deployedContract.methods
      .test_insert2_then_get2_then_nullify1_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();

    await expectNoteHashesSquashedExcept(1);
    await expectNullifiersSquashedExcept(0);
  }, 60_000);

  it('Squash! Aztec.nr function can nullify a pending note and a persistent in the same TX', async () => {
    // Create 1 note in isolated TX.
    // Then, in a separate TX, create 1 new note and nullify BOTH notes.
    // In this second TX, the kernel will squash one note + nullifier,
    // but the nullifier for the persistent note (from the first TX) will itself become persistent.
    // Realistic way to describe this test is "Mint note A, then burn note A in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    // create persistent note
    await deployedContract.methods.insert_note(mintAmount, owner).send().wait();

    await expectNoteHashesSquashedExcept(1); // first TX just creates 1 persistent note
    await expectNullifiersSquashedExcept(0);

    // create another note, and nullify it and AND nullify the above-created note in the same TX
    await deployedContract.methods
      .test_insert1_then_get2_then_nullify2_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();
    await expect(deployedContract.methods.get_note_zero_balance(owner).send().wait()).rejects.toThrow(
      `Assertion failed: Cannot return zero notes`,
    );

    // second TX creates 1 note, but it is squashed!
    await expectNoteHashesSquashedExcept(0);
    // the nullifier corresponding to this transient note is squashed, but the
    // other nullifier corresponding to the persistent note becomes persistent itself.
    await expectNullifiersSquashedExcept(1);
  }, 60_000);

  it('get_notes function filters a nullified note created in a previous transaction', async () => {
    // Create a note in an isolated transaction.
    // In a subsequent transaction, we nullify the note and a call to 'get note' should
    // not return anything.
    // Remark: This test can be seen as a simplification of the previous one but has the merit to
    // isolate the simplest 'get note' filtering with a pending nullifier on a persistent note.
    const mintAmount = 65n;

    const deployedContract = await deployContract();
    await deployedContract.methods.insert_note(mintAmount, owner).send().wait();

    // There is a single new note hash.
    await expectNoteHashesSquashedExcept(1);

    await deployedContract.methods
      .test_insert_then_get_then_nullify_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.dummy.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();

    // There is a single new nullifier.
    await expectNullifiersSquashedExcept(1);
  }, 60_000);
});
