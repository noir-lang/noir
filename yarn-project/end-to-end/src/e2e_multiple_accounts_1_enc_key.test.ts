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
  computeMessageSecretHash,
  generatePublicKey,
} from '@aztec/aztec.js';
import { TokenContract } from '@aztec/noir-contracts.js/Token';

import { expectsNumOfEncryptedLogsInTheLastBlockToBe, setup } from './fixtures/utils.js';

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

    const encryptionPrivateKey = GrumpkinScalar.random();

    for (let i = 0; i < numAccounts; i++) {
      logger(`Deploying account contract ${i}/3...`);
      const signingPrivateKey = GrumpkinScalar.random();
      const account = getSchnorrAccount(pxe, encryptionPrivateKey, signingPrivateKey);
      const wallet = await account.waitSetup({ interval: 0.1 });
      const completeAddress = account.getCompleteAddress();
      wallets.push(wallet);
      accounts.push(completeAddress);
    }
    logger('Account contracts deployed');

    // Verify that all accounts use the same encryption key
    const encryptionPublicKey = generatePublicKey(encryptionPrivateKey);

    for (const account of accounts) {
      expect(account.publicKey).toEqual(encryptionPublicKey);
    }

    logger(`Deploying Token...`);
    const token = await TokenContract.deploy(wallets[0], accounts[0], 'TokenName', 'TokenSymbol', 18).send().deployed();
    tokenAddress = token.address;
    logger(`Token deployed at ${tokenAddress}`);

    const secret = Fr.random();
    const secretHash = computeMessageSecretHash(secret);

    const receipt = await token.methods.mint_private(initialBalance, secretHash).send().wait();

    const storageSlot = new Fr(5);
    const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // TransparentNote

    const note = new Note([new Fr(initialBalance), secretHash]);
    const extendedNote = new ExtendedNote(
      note,
      accounts[0].address,
      token.address,
      storageSlot,
      noteTypeId,
      receipt.txHash,
    );
    await pxe.addNote(extendedNote);

    await token.methods.redeem_shield(accounts[0], initialBalance, secret).send().wait();
  }, 100_000);

  afterEach(() => teardown());

  const expectBalance = async (userIndex: number, expectedBalance: bigint) => {
    const wallet = wallets[userIndex];
    const owner = accounts[userIndex];

    // Then check the balance
    const contractWithWallet = await TokenContract.at(tokenAddress, wallet);
    const balance = await contractWithWallet.methods.balance_of_private(owner).simulate({ from: owner.address });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const transfer = async (
    senderIndex: number,
    receiverIndex: number,
    transferAmount: bigint,
    expectedBalances: bigint[],
  ) => {
    logger(`Transfer ${transferAmount} from ${accounts[senderIndex]} to ${accounts[receiverIndex]}...`);

    const sender = accounts[senderIndex];
    const receiver = accounts[receiverIndex];

    const contractWithWallet = await TokenContract.at(tokenAddress, wallets[senderIndex]);

    await contractWithWallet.methods.transfer(sender, receiver, transferAmount, 0).send().wait();

    for (let i = 0; i < expectedBalances.length; i++) {
      await expectBalance(i, expectedBalances[i]);
    }

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);

    logger(`Transfer ${transferAmount} from ${sender} to ${receiver} successful`);
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
  }, 180_000);
});
