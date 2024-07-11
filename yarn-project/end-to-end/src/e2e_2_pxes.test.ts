import { getUnsafeSchnorrAccount } from '@aztec/accounts/single_key';
import { createAccounts } from '@aztec/accounts/testing';
import {
  type AztecAddress,
  type AztecNode,
  type DebugLogger,
  ExtendedNote,
  Fr,
  Note,
  type PXE,
  type Wallet,
  computeSecretHash,
  retryUntil,
  sleep,
} from '@aztec/aztec.js';
import { ChildContract, TestContract, TokenContract } from '@aztec/noir-contracts.js';

import { expect, jest } from '@jest/globals';

import { expectsNumOfNoteEncryptedLogsInTheLastBlockToBe, setup, setupPXEService } from './fixtures/utils.js';

const TIMEOUT = 120_000;

describe('e2e_2_pxes', () => {
  jest.setTimeout(TIMEOUT);

  let aztecNode: AztecNode | undefined;
  let pxeA: PXE;
  let pxeB: PXE;
  let walletA: Wallet;
  let walletB: Wallet;
  let logger: DebugLogger;
  let teardownA: () => Promise<void>;
  let teardownB: () => Promise<void>;

  beforeEach(async () => {
    ({
      aztecNode,
      pxe: pxeA,
      wallets: [walletA],
      logger,
      teardown: teardownA,
    } = await setup(1));

    ({ pxe: pxeB, teardown: teardownB } = await setupPXEService(aztecNode!, {}, undefined, true));

    [walletB] = await createAccounts(pxeB, 1);
    /*TODO(post-honk): We wait 5 seconds for a race condition in setting up two nodes.
     What is a more robust solution? */
    await sleep(5000);
  });

  afterEach(async () => {
    await teardownB();
    await teardownA();
  });

  const awaitUserSynchronized = async (wallet: Wallet, owner: AztecAddress) => {
    const isUserSynchronized = async () => {
      return await wallet.isAccountStateSynchronized(owner);
    };
    await retryUntil(isUserSynchronized, `synch of user ${owner.toString()}`, 10);
  };

  const expectTokenBalance = async (
    wallet: Wallet,
    tokenAddress: AztecAddress,
    owner: AztecAddress,
    expectedBalance: bigint,
    checkIfSynchronized = true,
  ) => {
    if (checkIfSynchronized) {
      // First wait until the corresponding PXE has synchronized the account
      await awaitUserSynchronized(wallet, owner);
    }

    // Then check the balance
    const contractWithWallet = await TokenContract.at(tokenAddress, wallet);
    const balance = await contractWithWallet.methods.balance_of_private(owner).simulate({ from: owner });
    logger.info(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployTokenContract = async (initialAdminBalance: bigint, admin: AztecAddress, pxe: PXE) => {
    logger.info(`Deploying Token contract...`);
    const contract = await TokenContract.deploy(walletA, admin, 'TokenName', 'TokenSymbol', 18).send().deployed();

    if (initialAdminBalance > 0n) {
      // Minter is minting to herself so contract as minter is the same as contract as recipient
      await mintTokens(contract, contract, admin, initialAdminBalance, pxe);
    }

    logger.info('L2 contract deployed');

    return contract;
  };

  const mintTokens = async (
    contractAsMinter: TokenContract,
    contractAsRecipient: TokenContract,
    recipient: AztecAddress,
    balance: bigint,
    recipientPxe: PXE,
  ) => {
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);

    const receipt = await contractAsMinter.methods.mint_private(balance, secretHash).send().wait();

    const note = new Note([new Fr(balance), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      recipient,
      contractAsMinter.address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      receipt.txHash,
    );
    await recipientPxe.addNote(extendedNote);

    await contractAsRecipient.methods.redeem_shield(recipient, balance, secret).send().wait();
  };

  it('transfers funds from user A to B via PXE A followed by transfer from B to A via PXE B', async () => {
    const initialBalance = 987n;
    const transferAmount1 = 654n;
    const transferAmount2 = 323n;

    const token = await deployTokenContract(initialBalance, walletA.getAddress(), pxeA);

    // Add account B to wallet A
    await pxeA.registerRecipient(walletB.getCompleteAddress());
    // Add account A to wallet B
    await pxeB.registerRecipient(walletA.getCompleteAddress());

    // Add token to PXE B (PXE A already has it because it was deployed through it)
    await pxeB.registerContract(token);

    // Check initial balances and logs are as expected
    await expectTokenBalance(walletA, token.address, walletA.getAddress(), initialBalance);
    await expectTokenBalance(walletB, token.address, walletB.getAddress(), 0n);
    await expectsNumOfNoteEncryptedLogsInTheLastBlockToBe(aztecNode, 1);

    // Transfer funds from A to B via PXE A
    const contractWithWalletA = await TokenContract.at(token.address, walletA);
    await contractWithWalletA.methods.transfer(walletB.getAddress(), transferAmount1).send().wait();

    // Check balances and logs are as expected
    await expectTokenBalance(walletA, token.address, walletA.getAddress(), initialBalance - transferAmount1);
    await expectTokenBalance(walletB, token.address, walletB.getAddress(), transferAmount1);
    await expectsNumOfNoteEncryptedLogsInTheLastBlockToBe(aztecNode, 2);

    // Transfer funds from B to A via PXE B
    const contractWithWalletB = await TokenContract.at(token.address, walletB);
    await contractWithWalletB.methods.transfer(walletA.getAddress(), transferAmount2).send().wait({ interval: 0.1 });

    // Check balances and logs are as expected
    await expectTokenBalance(
      walletA,
      token.address,
      walletA.getAddress(),
      initialBalance - transferAmount1 + transferAmount2,
    );
    await expectTokenBalance(walletB, token.address, walletB.getAddress(), transferAmount1 - transferAmount2);
    await expectsNumOfNoteEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
  });

  const deployChildContractViaServerA = async () => {
    logger.info(`Deploying Child contract...`);
    const contract = await ChildContract.deploy(walletA).send().deployed();
    logger.info('Child contract deployed');

    return contract.instance;
  };

  const awaitServerSynchronized = async (server: PXE) => {
    const isServerSynchronized = async () => {
      return await server.isGlobalStateSynchronized();
    };
    await retryUntil(isServerSynchronized, 'server sync', 10);
  };

  const getChildStoredValue = (child: { address: AztecAddress }, pxe: PXE) =>
    pxe.getPublicStorageAt(child.address, new Fr(1));

  it('user calls a public function on a contract deployed by a different user using a different PXE', async () => {
    const childCompleteAddress = await deployChildContractViaServerA();

    await awaitServerSynchronized(pxeA);

    // Add Child to PXE B
    await pxeB.registerContract({
      artifact: ChildContract.artifact,
      instance: childCompleteAddress,
    });

    const newValueToSet = new Fr(256n);

    const childContractWithWalletB = await ChildContract.at(childCompleteAddress.address, walletB);
    await childContractWithWalletB.methods.pub_inc_value(newValueToSet).send().wait({ interval: 0.1 });

    await awaitServerSynchronized(pxeA);

    const storedValueOnB = await getChildStoredValue(childCompleteAddress, pxeB);
    expect(storedValueOnB).toEqual(newValueToSet);

    const storedValueOnA = await getChildStoredValue(childCompleteAddress, pxeA);
    expect(storedValueOnA).toEqual(newValueToSet);
  });

  it('private state is "zero" when PXE does not have the account secret key', async () => {
    const userABalance = 100n;
    const userBBalance = 150n;

    const token = await deployTokenContract(userABalance, walletA.getAddress(), pxeA);
    const contractWithWalletA = await TokenContract.at(token.address, walletA);

    // Add account B to wallet A
    await pxeA.registerRecipient(walletB.getCompleteAddress());
    // Add account A to wallet B
    await pxeB.registerRecipient(walletA.getCompleteAddress());

    // Add token to PXE B (PXE A already has it because it was deployed through it)
    await pxeB.registerContract(token);

    // Mint tokens to user B
    const contractWithWalletB = await TokenContract.at(token.address, walletB);
    await mintTokens(contractWithWalletA, contractWithWalletB, walletB.getAddress(), userBBalance, pxeB);

    // Check that user A balance is 100 on server A
    await expectTokenBalance(walletA, token.address, walletA.getAddress(), userABalance);
    // Check that user B balance is 150 on server B
    await expectTokenBalance(walletB, token.address, walletB.getAddress(), userBBalance);

    // CHECK THAT PRIVATE BALANCES ARE 0 WHEN ACCOUNT'S SECRET KEYS ARE NOT REGISTERED
    // Note: Not checking if the account is synchronized because it is not registered as an account (it would throw).
    const checkIfSynchronized = false;
    // Check that user A balance is 0 on server B
    await expectTokenBalance(walletB, token.address, walletA.getAddress(), 0n, checkIfSynchronized);
    // Check that user B balance is 0 on server A
    await expectTokenBalance(walletA, token.address, walletB.getAddress(), 0n, checkIfSynchronized);
  });

  it('permits migrating an account from one PXE to another', async () => {
    const secretKey = Fr.random();
    const account = getUnsafeSchnorrAccount(pxeA, secretKey, Fr.random());
    const completeAddress = account.getCompleteAddress();
    const wallet = await account.waitSetup();

    await expect(wallet.isAccountStateSynchronized(completeAddress.address)).resolves.toBe(true);
    const accountOnB = getUnsafeSchnorrAccount(pxeB, secretKey, account.salt);
    const walletOnB = await accountOnB.getWallet();

    // need to register first otherwise the new PXE won't know about the account
    await expect(walletOnB.isAccountStateSynchronized(completeAddress.address)).rejects.toThrow();

    await accountOnB.register();
    // registering should wait for the account to be synchronized
    await expect(walletOnB.isAccountStateSynchronized(completeAddress.address)).resolves.toBe(true);
  });

  it('permits sending funds to a user before they have registered the contract', async () => {
    const initialBalance = 987n;
    const transferAmount1 = 654n;

    const token = await deployTokenContract(initialBalance, walletA.getAddress(), pxeA);
    const tokenAddress = token.address;

    // Add account B to wallet A
    await pxeA.registerRecipient(walletB.getCompleteAddress());
    // Add account A to wallet B
    await pxeB.registerRecipient(walletA.getCompleteAddress());

    // Check initial balances and logs are as expected
    await expectTokenBalance(walletA, tokenAddress, walletA.getAddress(), initialBalance);
    // don't check userB yet

    await expectsNumOfNoteEncryptedLogsInTheLastBlockToBe(aztecNode, 1);

    // Transfer funds from A to B via PXE A
    const contractWithWalletA = await TokenContract.at(tokenAddress, walletA);
    await contractWithWalletA.methods.transfer(walletB.getAddress(), transferAmount1).send().wait();

    // now add the contract and check balances
    await pxeB.registerContract(token);
    await expectTokenBalance(walletA, tokenAddress, walletA.getAddress(), initialBalance - transferAmount1);
    await expectTokenBalance(walletB, tokenAddress, walletB.getAddress(), transferAmount1);
  });

  it('permits sending funds to a user, and spending them, before they have registered the contract', async () => {
    const initialBalance = 987n;
    const transferAmount1 = 654n;
    const transferAmount2 = 323n;

    // setup an account that is shared across PXEs
    const sharedSecretKey = Fr.random();
    const sharedAccountOnA = getUnsafeSchnorrAccount(pxeA, sharedSecretKey, Fr.random());
    const sharedAccountAddress = sharedAccountOnA.getCompleteAddress();
    const sharedWalletOnA = await sharedAccountOnA.waitSetup();
    await expect(sharedWalletOnA.isAccountStateSynchronized(sharedAccountAddress.address)).resolves.toBe(true);

    const sharedAccountOnB = getUnsafeSchnorrAccount(pxeB, sharedSecretKey, sharedAccountOnA.salt);
    await sharedAccountOnB.register();
    const sharedWalletOnB = await sharedAccountOnB.getWallet();

    // Register wallet B in the pxe of wallet A
    await pxeA.registerRecipient(walletB.getCompleteAddress());

    // deploy the contract on PXE A
    const token = await deployTokenContract(initialBalance, walletA.getAddress(), pxeA);

    // Transfer funds from A to Shared Wallet via PXE A
    const contractWithWalletA = await TokenContract.at(token.address, walletA);
    await contractWithWalletA.methods.transfer(sharedAccountAddress.address, transferAmount1).send().wait();

    // Now send funds from Shared Wallet to B via PXE A
    const contractWithSharedWalletA = await TokenContract.at(token.address, sharedWalletOnA);
    await contractWithSharedWalletA.methods.transfer(walletB.getAddress(), transferAmount2).send().wait();

    // check balances from PXE-A's perspective
    await expectTokenBalance(walletA, token.address, walletA.getAddress(), initialBalance - transferAmount1);
    await expectTokenBalance(
      sharedWalletOnA,
      token.address,
      sharedAccountAddress.address,
      transferAmount1 - transferAmount2,
    );

    // now add the contract and check balances from PXE-B's perspective.
    // The process should be:
    // PXE-B had previously deferred the notes from A -> Shared, and Shared -> B
    // PXE-B adds the contract
    // PXE-B reprocesses the deferred notes, and sees the nullifier for A -> Shared
    await pxeB.registerContract(token);
    await expectTokenBalance(walletB, token.address, walletB.getAddress(), transferAmount2);
    await expect(sharedWalletOnB.isAccountStateSynchronized(sharedAccountAddress.address)).resolves.toBe(true);
    await expectTokenBalance(
      sharedWalletOnB,
      token.address,
      sharedAccountAddress.address,
      transferAmount1 - transferAmount2,
    );
  });

  it('adds and fetches a nullified note', async () => {
    // 1. Deploys test contract through PXE A
    const testContract = await TestContract.deploy(walletA).send().deployed();

    // 2. Create a note
    const noteStorageSlot = 10;
    const noteValue = 5;
    let note: ExtendedNote;
    {
      const owner = walletA.getAddress();
      const outgoingViewer = owner;

      const receipt = await testContract.methods
        .call_create_note(noteValue, owner, outgoingViewer, noteStorageSlot)
        .send()
        .wait({ debug: true });
      const { visibleIncomingNotes, visibleOutgoingNotes } = receipt.debugInfo!;
      expect(visibleIncomingNotes).toHaveLength(1);
      note = visibleIncomingNotes![0];

      // Since owner is the same as outgoing viewer the incoming and outgoing notes should be the same
      expect(visibleOutgoingNotes).toHaveLength(1);
      expect(visibleOutgoingNotes![0]).toEqual(note);
    }

    // 3. Nullify the note
    {
      const receipt = await testContract.methods.call_destroy_note(noteStorageSlot).send().wait({ debug: true });
      // Check that we got 2 nullifiers - 1 for tx hash, 1 for the note
      expect(receipt.debugInfo?.nullifiers).toHaveLength(2);
    }

    // 4. Adds the nullified public key note to PXE B
    {
      // We need to register the recipient to be able to obtain IvpkM for the note
      await pxeB.registerRecipient(walletA.getCompleteAddress());
      // We need to register the contract to be able to compute the note hash by calling compute_note_hash_and_optionally_a_nullifier(...)
      await pxeB.registerContract(testContract);
      await pxeB.addNullifiedNote(note);
    }

    // 5. Try fetching the nullified note
    {
      const testContractWithWalletB = await TestContract.at(testContract.address, walletB);
      const noteValue = await testContractWithWalletB.methods.call_get_notes(noteStorageSlot, true).simulate();
      expect(noteValue).toBe(noteValue);
      // --> We have successfully obtained the nullified note from PXE B verifying that pxe.addNullifiedNote(...) works
    }
  });
});
