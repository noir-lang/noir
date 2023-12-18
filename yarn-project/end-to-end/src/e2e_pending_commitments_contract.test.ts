import { AztecAddress, AztecNode, CompleteAddress, DebugLogger, Fr, TxStatus, Wallet } from '@aztec/aztec.js';
import { PendingCommitmentsContract } from '@aztec/noir-contracts/types';

import { setup } from './fixtures/utils.js';

describe('e2e_pending_commitments_contract', () => {
  let aztecNode: AztecNode | undefined;
  let wallet: Wallet;
  let logger: DebugLogger;
  let owner: AztecAddress;
  let teardown: () => Promise<void>;
  let contract: PendingCommitmentsContract;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ teardown, aztecNode, accounts, wallet, logger } = await setup(2));
    owner = accounts[0].address;
  }, 100_000);

  afterEach(() => teardown());

  const expectCommitmentsSquashedExcept = async (exceptFirstFew: number) => {
    const blockNum = await aztecNode!.getBlockNumber();
    const block = (await aztecNode!.getBlocks(blockNum, 1))[0];

    // all new commitments should be zero (should be squashed)
    for (let c = 0; c < exceptFirstFew; c++) {
      expect(block.newCommitments[c]).not.toEqual(Fr.ZERO);
    }

    for (let c = exceptFirstFew; c < block.newCommitments.length; c++) {
      expect(block.newCommitments[c]).toEqual(Fr.ZERO);
    }
  };

  const expectNullifiersSquashedExcept = async (exceptFirstFew: number) => {
    const blockNum = await aztecNode!.getBlockNumber();
    const block = (await aztecNode!.getBlocks(blockNum, 1))[0];

    // 0th nullifier should be nonzero (txHash), all others should be zero (should be squashed)
    for (let n = 0; n < exceptFirstFew + 1; n++) {
      logger(`Expecting nullifier ${n} to be nonzero`);
      expect(block.newNullifiers[n]).not.toEqual(Fr.ZERO); // 0th nullifier is txHash
    }
    for (let n = exceptFirstFew + 1; n < block.newNullifiers.length; n++) {
      expect(block.newNullifiers[n]).toEqual(Fr.ZERO);
    }
  };

  const deployContract = async () => {
    logger(`Deploying L2 contract...`);
    contract = await PendingCommitmentsContract.deploy(wallet).send().deployed();
    logger('L2 contract deployed');
    return contract;
  };

  it('Aztec.nr function can "get" notes it just "inserted"', async () => {
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    const receipt = await deployedContract.methods
      .test_insert_then_get_then_nullify_flat(mintAmount, owner)
      .send()
      .wait();
    expect(receipt.status).toBe(TxStatus.MINED);
  }, 60_000);

  it('Squash! Aztec.nr function can "create" and "nullify" note in the same TX', async () => {
    // Kernel will squash the noteHash and its nullifier.
    // Realistic way to describe this test is "Mint note A, then burn note A in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    const receipt = await deployedContract.methods
      .test_insert_then_get_then_nullify_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
        deployedContract.methods.get_note_zero_balance.selector,
      )
      .send()
      .wait();

    expect(receipt.status).toBe(TxStatus.MINED);

    await expectCommitmentsSquashedExcept(0);
    await expectNullifiersSquashedExcept(0);
  }, 60_000);

  it('Squash! Aztec.nr function can "create" 2 notes and "nullify" both in the same TX', async () => {
    // Kernel will squash both noteHashes and their nullifier.
    // Realistic way to describe this test is "Mint notes A and B, then burn both in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    const receipt = await deployedContract.methods
      .test_insert2_then_get2_then_nullify2_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();

    expect(receipt.status).toBe(TxStatus.MINED);

    await expectCommitmentsSquashedExcept(0);
    await expectNullifiersSquashedExcept(0);
  }, 60_000);

  it('Squash! Aztec.nr function can "create" 2 notes and "nullify" 1 in the same TX (kernel will squash one note + nullifier)', async () => {
    // Kernel will squash one noteHash and its nullifier.
    // The other note will become persistent!
    // Realistic way to describe this test is "Mint notes A and B, then burn note A in the same transaction"
    const mintAmount = 65n;

    const deployedContract = await deployContract();

    const receipt = await deployedContract.methods
      .test_insert2_then_get2_then_nullify1_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
      )
      .send()
      .wait();

    expect(receipt.status).toBe(TxStatus.MINED);

    await expectCommitmentsSquashedExcept(1);
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
    const receipt0 = await deployedContract.methods.insert_note(mintAmount, owner).send().wait();
    expect(receipt0.status).toBe(TxStatus.MINED);

    await expectCommitmentsSquashedExcept(1); // first TX just creates 1 persistent note
    await expectNullifiersSquashedExcept(0);

    // create another note, and nullify it and AND nullify the above-created note in the same TX
    const receipt1 = await deployedContract.methods
      .test_insert1_then_get2_then_nullify2_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.insert_note.selector,
        deployedContract.methods.get_then_nullify_note.selector,
        deployedContract.methods.get_note_zero_balance.selector,
      )
      .send()
      .wait();

    expect(receipt1.status).toBe(TxStatus.MINED);

    // second TX creates 1 note, but it is squashed!
    await expectCommitmentsSquashedExcept(0);
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
    const receipt = await deployedContract.methods.insert_note(mintAmount, owner).send().wait();

    expect(receipt.status).toBe(TxStatus.MINED);

    // There is a single new commitment/note.
    await expectCommitmentsSquashedExcept(1);

    const receipt2 = await deployedContract.methods
      .test_insert_then_get_then_nullify_all_in_nested_calls(
        mintAmount,
        owner,
        deployedContract.methods.dummy.selector,
        deployedContract.methods.get_then_nullify_note.selector,
        deployedContract.methods.get_note_zero_balance.selector,
      )
      .send()
      .wait();

    expect(receipt2.status).toBe(TxStatus.MINED);

    // There is a single new nullifier.
    await expectNullifiersSquashedExcept(1);
  }, 60_000);
});
