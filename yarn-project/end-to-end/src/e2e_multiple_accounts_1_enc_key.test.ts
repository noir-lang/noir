import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, StoredKeyAccountContract, Wallet, generatePublicKey } from '@aztec/aztec.js';
import { PrivateKey } from '@aztec/circuits.js';
import { Schnorr } from '@aztec/circuits.js/barretenberg';
import { DebugLogger } from '@aztec/foundation/log';
import { SchnorrMultiKeyAccountContractAbi } from '@aztec/noir-contracts/artifacts';
import { ZkTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import {
  createNewAccount,
  expectUnencryptedLogsFromLastBlockToBe,
  expectsNumOfEncryptedLogsInTheLastBlockToBe,
  setup,
} from './utils.js';

describe('e2e_multiple_accounts_1_enc_key', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  const wallets: Wallet[] = [];
  const accounts: AztecAddress[] = [];
  let logger: DebugLogger;

  let zkTokenAddress: AztecAddress;

  const initialBalance = 987n;
  const numAccounts = 3;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, logger } = await setup(0));

    const encryptionPrivateKey = PrivateKey.random();
    for (let i = 0; i < numAccounts; i++) {
      logger(`Deploying account contract ${i}/3...`);
      const signingPrivateKey = PrivateKey.random();
      const createWallet = async (address: AztecAddress, useProperKey: boolean) =>
        new StoredKeyAccountContract(
          address,
          useProperKey ? signingPrivateKey : PrivateKey.random(),
          await Schnorr.new(),
        );

      const schnorr = await Schnorr.new();
      const signingPublicKey = schnorr.computePublicKey(signingPrivateKey);
      const constructorArgs = [signingPublicKey.x, signingPublicKey.y];

      const { wallet, address } = await createNewAccount(
        aztecRpcServer,
        SchnorrMultiKeyAccountContractAbi,
        constructorArgs,
        encryptionPrivateKey,
        true,
        createWallet,
      );
      wallets.push(wallet);
      accounts.push(address);
    }
    logger('Account contracts deployed');

    // Verify that all accounts use the same encryption key
    const encryptionPublicKey = await generatePublicKey(encryptionPrivateKey);
    for (let i = 0; i < numAccounts; i++) {
      const accountEncryptionPublicKey = await aztecRpcServer.getPublicKey(accounts[i]);
      expect(accountEncryptionPublicKey).toEqual(encryptionPublicKey);
    }

    logger(`Deploying ZK Token...`);
    const tx = ZkTokenContract.deploy(aztecRpcServer, initialBalance, accounts[0]).send();
    const receipt = await tx.getReceipt();
    zkTokenAddress = receipt.contractAddress!;
    await tx.isMined({ interval: 0.1 });
    const minedReceipt = await tx.getReceipt();
    expect(minedReceipt.status).toEqual(TxStatus.MINED);
    logger('ZK Token deployed');
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  const expectBalance = async (userIndex: number, expectedBalance: bigint) => {
    const wallet = wallets[userIndex];
    const owner = accounts[userIndex];

    // Then check the balance
    const contractWithWallet = await ZkTokenContract.create(zkTokenAddress, wallet);
    const [balance] = await contractWithWallet.methods.getBalance(owner).view({ from: owner });
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

    const contractWithWallet = await ZkTokenContract.create(zkTokenAddress, wallets[senderIndex]);

    const tx = contractWithWallet.methods.transfer(transferAmount, sender, receiver).send({ origin: sender });
    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);

    for (let i = 0; i < expectedBalances.length; i++) {
      await expectBalance(i, expectedBalances[i]);
    }

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Coins transferred']);

    logger(`Transfer ${transferAmount} from ${sender} to ${receiver} successful`);
  };

  /**
   * Tests the ability of the Aztec RPC server to handle multiple accounts under the same encryption key.
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
