import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Contract, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { MultiTransferContract, PrivateTokenAirdropContract } from '@aztec/noir-contracts/types';
import { AztecRPC, CompleteAddress } from '@aztec/types';

import { expectsNumOfEncryptedLogsInTheLastBlockToBe, setup } from './fixtures/utils.js';

/**
 * Multi-transfer payments is an example application to demonstrate how a payroll application could be built using aztec.
 * In the current version of aztec, each multi-transfer can support only 12 recipients per transaction. The sender
 * can decide which note can be spent.
 */
describe('multi-transfer payments', () => {
  const numberOfAccounts = 12;

  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let logger: DebugLogger;
  let ownerAddress: AztecAddress;
  const recipients: AztecAddress[] = [];
  let initialBalance: bigint;

  let zkTokenContract: PrivateTokenAirdropContract;
  let multiTransferContract: MultiTransferContract;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ aztecNode, aztecRpcServer, accounts, logger, wallet } = await setup(numberOfAccounts + 1)); // 1st being the `owner`
    ownerAddress = accounts[0].address;

    for (let i = 1; i < accounts.length; i++) {
      const account = accounts[i].address;
      recipients.push(account);
    }

    logger(`Deploying zk token contract...`);
    initialBalance = 1000n;
    await deployZkTokenContract(initialBalance, ownerAddress);

    logger(`Deploying multi-transfer contract...`);
    await deployMultiTransferContract();
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  }, 30_000);

  const deployZkTokenContract = async (initialBalance: bigint, owner: AztecAddress) => {
    logger(`Deploying zk token contract...`);
    zkTokenContract = await PrivateTokenAirdropContract.deploy(wallet, initialBalance, owner).send().deployed();
    logger(`zk token contract deployed at ${zkTokenContract.address}`);
  };

  const deployMultiTransferContract = async () => {
    logger(`Deploying multi-transfer contract...`);
    multiTransferContract = await MultiTransferContract.deploy(wallet).send().deployed();
    logger(`multi-transfer contract deployed at ${multiTransferContract.address}`);
  };

  const expectBalance = async (tokenContract: Contract, owner: AztecAddress, expectedBalance: bigint) => {
    const balance = await tokenContract.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  /**
   * Payroll example
   *
   * Transaction 1:
   * The sender first splits 1000 to create new notes (for himself) with values 100, 200, 300, 400:
   * 0: sender: [1000]
   *             |
   *             +-- [100 (change), 200, 300, 400]
   *
   * Transaction 2:
   * In the next transaction, the sender wants to spend all four notes created in the previous transaction:
   * index:     [0    1    2    3   4   5   6   7]
   * 0: sender: [100, 200, 300, 400]
   *             |
   *             +-- [25 (change), 20, 25, 30] // first batchTx call
   *
   * index:     [0    1    2    3   4   5   6   7]
   * 1: sender: [200, 300, 400, 25]
   *             |
   *             +-- [50 (change), 40, 50, 60] // second batchTx call
   *
   * index:     [0    1    2   3   4   5   6   7]
   * 2: sender: [300, 400, 25, 50]
   *             |
   *             +-- [60 (change), 75, 80, 85] // third batchTx call
   *
   * index:     [0    1   2   3   4   5   6   7]
   * 3: sender: [400, 25, 50, 60]
   *             |
   *             +-- [50 (change), 100, 120, 130] // fourth batchTx call
   *
   */
  it('12 transfers per transactions should work', async () => {
    // Transaction 1
    logger(`self batchTransfer()`);
    const batchTransferTx = zkTokenContract.methods
      .batchTransfer(ownerAddress, [200n, 300n, 400n], [ownerAddress, ownerAddress, ownerAddress], 0)
      .send({ origin: ownerAddress });
    await batchTransferTx.isMined();
    const batchTransferTxReceipt = await batchTransferTx.getReceipt();
    logger(`consumption Receipt status: ${batchTransferTxReceipt.status}`);
    await expectBalance(zkTokenContract, ownerAddress, initialBalance);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 4);

    const amounts: bigint[] = [20n, 25n, 30n, 40n, 50n, 60n, 75n, 80n, 85n, 100n, 120n, 130n];
    const amountSum = amounts.reduce((a, b) => a + b, 0n);
    const noteOffsets: bigint[] = [0n, 0n, 0n, 0n];

    // Transaction 2
    logger(`multiTransfer()...`);
    const multiTransferTx = multiTransferContract.methods
      .multiTransfer(
        zkTokenContract.address.toField(),
        recipients,
        amounts,
        ownerAddress,
        zkTokenContract.methods.batchTransfer.selector.toField(),
        noteOffsets,
      )
      .send({ origin: ownerAddress });
    await multiTransferTx.isMined({ timeout: 1000 }); // mining timeout ≥ time needed for the test to finish.
    const multiTransferTxReceipt = await multiTransferTx.getReceipt();
    logger(`Consumption Receipt status: ${multiTransferTxReceipt.status}`);

    await expectBalance(zkTokenContract, ownerAddress, initialBalance - amountSum);
    for (let index = 0; index < numberOfAccounts; index++) {
      await expectBalance(zkTokenContract, recipients[index], amounts[index]);
    }
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 16);
  }, 100_000);

  /**
   * Creating change notes for self.
   *
   * Transaction 1: Splits the 1000 note to create 12 notes x 50 each.
   * index:     [0    1   2   3   4   5   6   7]
   * 0: sender: [1000]
   *             |
   *             +-- [850, 50, 50, 50]
   *
   * index:     [0    1   2   3    4   5   6   7]
   * 1: sender: [850, 50, 50, 50]
   *             |
   *             +-- [700, 50, 50, 50]
   *
   * index:     [0   1   2   3    4   5   6   7]
   * 2: sender: [50, 50, 50, 700, 50, 50, 50]
   *                         |
   *                         +-- [550, 50, 50, 50]
   *
   * index:     [0   1   2   3   4   5   6    7]
   * 3: sender: [50, 50, 50, 50, 50, 50, 550, 50, 50, 50]
   *                                     |
   *                                     +-- [400, 50, 50, 50]
   *
   * End state:
   * sender: [50, 50, 50, 50, 50, 50, 50, 50, 50, 400, 50, 50, 50]
   */
  it('create 12 small notes out of 1 large note', async () => {
    // Transaction 1
    const amounts: bigint[] = [50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n];
    const noteOffsets: bigint[] = [0n, 0n, 3n, 6n];
    const repeatedSelfAdddress: AztecAddress[] = Array(12).fill(ownerAddress);

    logger(`split multiTransfer()...`);
    const multiTransferTx = multiTransferContract.methods
      .multiTransfer(
        zkTokenContract.address.toField(),
        repeatedSelfAdddress,
        amounts,
        ownerAddress,
        zkTokenContract.methods.batchTransfer.selector.toField(),
        noteOffsets,
      )
      .send({ origin: ownerAddress });
    await multiTransferTx.isMined({ timeout: 100 }); // mining timeout ≥ time needed for the test to finish.
    const multiTransferTxReceipt = await multiTransferTx.getReceipt();
    logger(`Consumption Receipt status: ${multiTransferTxReceipt.status}`);

    await expectBalance(zkTokenContract, ownerAddress, initialBalance);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 16);
  }, 100_000);
});
