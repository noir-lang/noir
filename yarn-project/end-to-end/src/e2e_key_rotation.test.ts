import { createAccounts } from '@aztec/accounts/testing';
import {
  type AztecAddress,
  type AztecNode,
  type DebugLogger,
  ExtendedNote,
  Fq,
  Fr,
  Note,
  type PXE,
  type TxHash,
  type Wallet,
  computeSecretHash,
  retryUntil,
} from '@aztec/aztec.js';
// docs:start:imports
import { type PublicKey, derivePublicKeyFromSecretKey } from '@aztec/circuits.js';
import { TestContract, TokenContract } from '@aztec/noir-contracts.js';

// docs:end:imports
import { jest } from '@jest/globals';

import { expectsNumOfNoteEncryptedLogsInTheLastBlockToBe, setup, setupPXEService } from './fixtures/utils.js';

const TIMEOUT = 120_000;

const SHARED_MUTABLE_DELAY = 5;

describe('e2e_key_rotation', () => {
  jest.setTimeout(TIMEOUT);

  let aztecNode: AztecNode;
  let pxeA: PXE;
  let pxeB: PXE;
  let walletA: Wallet;
  let walletB: Wallet;
  let logger: DebugLogger;
  let teardownA: () => Promise<void>;
  let teardownB: () => Promise<void>;

  let testContract: TestContract;
  let contractWithWalletA: TokenContract;
  let contractWithWalletB: TokenContract;

  let tokenAddress: AztecAddress;

  const initialBalance = 987n;

  beforeAll(async () => {
    ({
      aztecNode,
      pxe: pxeA,
      wallets: [walletA],
      logger,
      teardown: teardownA,
    } = await setup(1));

    ({ pxe: pxeB, teardown: teardownB } = await setupPXEService(aztecNode, {}, undefined, true));
    [walletB] = await createAccounts(pxeB, 1);

    // We deploy test and token contracts
    testContract = await TestContract.deploy(walletA).send().deployed();
    const tokenInstance = await deployTokenContract(initialBalance, walletA.getAddress(), pxeA);
    tokenAddress = tokenInstance.address;

    // Add account B to wallet A
    await pxeA.registerRecipient(walletB.getCompleteAddress());
    // Add account A to wallet B
    await pxeB.registerRecipient(walletA.getCompleteAddress());

    // Add token to PXE B (PXE A already has it because it was deployed through it)
    await pxeB.registerContract({
      artifact: TokenContract.artifact,
      instance: tokenInstance,
    });

    contractWithWalletA = await TokenContract.at(tokenAddress, walletA);
    contractWithWalletB = await TokenContract.at(tokenAddress, walletB);
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

  const crossDelay = async () => {
    for (let i = 0; i < SHARED_MUTABLE_DELAY; i++) {
      // We send arbitrary tx to mine a block
      await testContract.methods.emit_unencrypted(0).send().wait();
    }
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
      await mintTokens(contract, admin, initialAdminBalance, pxe);
    }

    logger.info('L2 contract deployed');

    return contract.instance;
  };

  const mintTokens = async (contract: TokenContract, recipient: AztecAddress, balance: bigint, pxe: PXE) => {
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);

    const receipt = await contract.methods.mint_private(balance, secretHash).send().wait();

    const note = new Note([new Fr(balance), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      recipient,
      contract.address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      receipt.txHash,
    );
    await pxe.addNote(extendedNote);

    await contract.methods.redeem_shield(recipient, balance, secret).send().wait();
  };

  it(`Rotates keys and uses them`, async () => {
    // 1. We check that setup set initial balances as expected
    await expectTokenBalance(walletA, tokenAddress, walletA.getAddress(), initialBalance);
    await expectTokenBalance(walletB, tokenAddress, walletB.getAddress(), 0n);

    // 2. Transfer funds from A to B via PXE A
    let txHashTransfer1: TxHash;
    const transfer1Amount = 654n;
    {
      ({ txHash: txHashTransfer1 } = await contractWithWalletA.methods
        .transfer(walletB.getAddress(), transfer1Amount)
        .send()
        .wait());

      // Check balances and logs are as expected
      await expectTokenBalance(walletA, tokenAddress, walletA.getAddress(), initialBalance - transfer1Amount);
      await expectTokenBalance(walletB, tokenAddress, walletB.getAddress(), transfer1Amount);
      await expectsNumOfNoteEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
    }

    // 3. Rotates B key
    let newNpkM: PublicKey;
    {
      // docs:start:create_keys
      const newNskM = Fq.random();
      newNpkM = derivePublicKeyFromSecretKey(newNskM);
      // docs:end:create_keys

      // docs:start:rotateNullifierKeys
      // This function saves the new nullifier secret key for the account in our PXE,
      // and calls the key registry with the derived nullifier public key.
      await walletB.rotateNullifierKeys(newNskM);
      // docs:end:rotateNullifierKeys
      await crossDelay();
    }

    // 4. Transfer funds from A to B via PXE A
    let txHashTransfer2: TxHash;
    const transfer2Amount = 321n;
    {
      ({ txHash: txHashTransfer2 } = await contractWithWalletA.methods
        .transfer(walletB.getAddress(), transfer2Amount)
        .send()
        .wait());

      await expectTokenBalance(
        walletA,
        tokenAddress,
        walletA.getAddress(),
        initialBalance - transfer1Amount - transfer2Amount,
      );
      await expectTokenBalance(walletB, tokenAddress, walletB.getAddress(), transfer1Amount + transfer2Amount);
    }

    // 5. Now we check that a correct nullifier keys were used in both transfers
    {
      await awaitUserSynchronized(walletB, walletB.getAddress());
      const transfer1Notes = await walletB.getIncomingNotes({ txHash: txHashTransfer1 });
      const transfer2Notes = await walletB.getIncomingNotes({ txHash: txHashTransfer2 });
      expect(transfer1Notes.length).toBe(1);
      expect(transfer2Notes.length).toBe(1);
      // Second field in the token note is the npk_m_hash
      const noteNpkMHashTransfer1 = transfer1Notes[0].note.items[1];
      const noteNpkMHashTransfer2 = transfer2Notes[0].note.items[1];

      // Now we check the note created in transfer 2 used the new npk_m_hash
      expect(noteNpkMHashTransfer2.equals(newNpkM.hash())).toBe(true);
      // We sanity check that the note created in transfer 1 had old npk_m_hash by checking it's different from the new
      // one
      expect(noteNpkMHashTransfer2.equals(noteNpkMHashTransfer1)).toBe(false);
    }

    // 6. Finally we check that all the B notes are spendable by transferring full B balance to A
    // --> this way we verify that it's possible to obtain both keys via oracles
    {
      await contractWithWalletB.methods
        .transfer(walletA.getAddress(), transfer1Amount + transfer2Amount)
        .send()
        .wait();

      await expectTokenBalance(walletA, tokenAddress, walletA.getAddress(), initialBalance);
      await expectTokenBalance(walletB, tokenAddress, walletB.getAddress(), 0n);
    }
  }, 600_000);
});
