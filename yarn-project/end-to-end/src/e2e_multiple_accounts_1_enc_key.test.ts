import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AztecAddress,
  type AztecNode,
  type CompleteAddress,
  type DebugLogger,
  ExtendedNote,
  Fr,
  GrumpkinScalar,
  Note,
  type PXE,
  type Wallet,
  computeSecretHash,
  deriveKeys,
} from '@aztec/aztec.js';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { expectsNumOfNoteEncryptedLogsInTheLastBlockToBe, setup } from './fixtures/utils.js';

describe('e2e_multiple_accounts_1_enc_key', () => {
  let aztecNode: AztecNode | undefined;
  let pxe: PXE;
  const wallets: Wallet[] = [];
  const accounts: CompleteAddress[] = [];
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let tokenAddress: AztecAddress;

  const initialBalance = 987n;
  const numAccounts = 3;

  beforeEach(async () => {
    ({ teardown, aztecNode, pxe, logger } = await setup(0));

    const encryptionPrivateKey = Fr.random();

    for (let i = 0; i < numAccounts; i++) {
      logger.info(`Deploying account contract ${i}/3...`);
      const signingPrivateKey = GrumpkinScalar.random();
      const account = getSchnorrAccount(pxe, encryptionPrivateKey, signingPrivateKey);
      const wallet = await account.waitSetup({ interval: 0.1 });
      const completeAddress = account.getCompleteAddress();
      wallets.push(wallet);
      accounts.push(completeAddress);
    }
    logger.info('Account contracts deployed');

    // Verify that all accounts use the same encryption key
    const encryptionPublicKey = deriveKeys(encryptionPrivateKey).publicKeys.masterIncomingViewingPublicKey;

    for (const account of accounts) {
      expect(account.publicKeys.masterIncomingViewingPublicKey).toEqual(encryptionPublicKey);
    }

    logger.info(`Deploying Token...`);
    const token = await TokenContract.deploy(wallets[0], accounts[0], 'TokenName', 'TokenSymbol', 18).send().deployed();
    tokenAddress = token.address;
    logger.info(`Token deployed at ${tokenAddress}`);

    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);

    const receipt = await token.methods.mint_private(initialBalance, secretHash).send().wait();

    const note = new Note([new Fr(initialBalance), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      accounts[0].address,
      token.address,
      TokenContract.storage.pending_shields.slot,
      TokenContract.notes.TransparentNote.id,
      receipt.txHash,
    );
    await wallets[0].addNote(extendedNote);

    await token.methods.redeem_shield(accounts[0], initialBalance, secret).send().wait();
  });

  afterEach(() => teardown());

  const expectBalance = async (userIndex: number, expectedBalance: bigint) => {
    const wallet = wallets[userIndex];
    const owner = accounts[userIndex];

    // Then check the balance
    const contractWithWallet = await TokenContract.at(tokenAddress, wallet);
    const balance = await contractWithWallet.methods.balance_of_private(owner).simulate({ from: owner.address });
    logger.info(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const transfer = async (
    senderIndex: number,
    receiverIndex: number,
    transferAmount: bigint,
    expectedBalances: bigint[],
  ) => {
    logger.info(`Transfer ${transferAmount} from ${accounts[senderIndex]} to ${accounts[receiverIndex]}...`);

    const sender = accounts[senderIndex];
    const receiver = accounts[receiverIndex];

    const contractWithWallet = await TokenContract.at(tokenAddress, wallets[senderIndex]);

    await contractWithWallet.methods.transfer(receiver, transferAmount).send().wait();

    for (let i = 0; i < expectedBalances.length; i++) {
      await expectBalance(i, expectedBalances[i]);
    }

    await expectsNumOfNoteEncryptedLogsInTheLastBlockToBe(aztecNode, 2);

    logger.info(`Transfer ${transferAmount} from ${sender} to ${receiver} successful`);
  };

  /**
   * Tests the ability of the Private eXecution Environment (PXE) to handle multiple accounts under the same encryption key.
   */
  it('spends notes from multiple account under the same encryption key', async () => {
    const transferAmount1 = 654n; // account 0 -> account 1
    const transferAmount2 = 123n; // account 0 -> account 2
    const transferAmount3 = 210n; // account 1 -> account 2

    await expectBalance(0, initialBalance);
    await expectBalance(1, 0n);
    await expectBalance(2, 0n);

    const expectedBalancesAfterTransfer1 = [initialBalance - transferAmount1, transferAmount1, 0n];
    await transfer(0, 1, transferAmount1, expectedBalancesAfterTransfer1);

    const expectedBalancesAfterTransfer2 = [
      expectedBalancesAfterTransfer1[0] - transferAmount2,
      expectedBalancesAfterTransfer1[1],
      transferAmount2,
    ];
    await transfer(0, 2, transferAmount2, expectedBalancesAfterTransfer2);

    const expectedBalancesAfterTransfer3 = [
      expectedBalancesAfterTransfer2[0],
      expectedBalancesAfterTransfer2[1] - transferAmount3,
      expectedBalancesAfterTransfer2[2] + transferAmount3,
    ];
    await transfer(1, 2, transferAmount3, expectedBalancesAfterTransfer3);
  }, 120_000);
});
