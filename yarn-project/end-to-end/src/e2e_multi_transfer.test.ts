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
  let recipients: AztecAddress[];
  let initialBalance: bigint;

  let zkTokenContract: PrivateTokenAirdropContract;
  let multiTransferContract: MultiTransferContract;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ aztecNode, aztecRpcServer, accounts, logger, wallet } = await setup(numberOfAccounts + 1)); // 1st being the `owner`
    ownerAddress = accounts[0].address;
    recipients = accounts.slice(1).map(a => a.address);

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
    await zkTokenContract.methods
      .batchTransfer(ownerAddress, [200n, 300n, 400n], [ownerAddress, ownerAddress, ownerAddress], 0)
      .send()
      .wait();

    await expectBalance(zkTokenContract, ownerAddress, initialBalance);
    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 4);

    const amounts: bigint[] = [20n, 25n, 30n, 40n, 50n, 60n, 75n, 80n, 85n, 100n, 120n, 130n];
    const amountSum = amounts.reduce((a, b) => a + b, 0n);
    const noteOffsets: bigint[] = [0n, 0n, 0n, 0n];

    // Transaction 2
    logger(`multiTransfer()...`);
    await multiTransferContract.methods
      .multiTransfer(zkTokenContract.address.toField(), recipients, amounts, ownerAddress, noteOffsets)
      .send()
      .wait({ timeout: 1000 }); // mining timeout ≥ time needed for the test to finish.

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
   *
   * Transaction 2: Spend more notes than it's allowed in a single call, to transfer some amount to a recipient.
   * It will destroy the largest note (400n) plus 8 small notes (50n * 8n).
   * 4 notes will be destroyed first: [400n, 50n, 50n, 50n]
   * And another 4 + 1 notes will be burnt in two function calls: [50n, 50n, 50n, 50n] and [50n]
   * One change note (10n) will be created for the sender.
   * One note will be created for the recipient.
   */
  it('create 12 small notes out of 1 large note and transfer to a recipient', async () => {
    // Transaction 1
    logger(`split multiTransfer()...`);
    {
      const amounts: bigint[] = [50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n, 50n];
      const noteOffsets: bigint[] = [0n, 0n, 3n, 6n];
      const repeatedSelfAdddress: AztecAddress[] = Array(12).fill(ownerAddress);

      await multiTransferContract.methods
        .multiTransfer(zkTokenContract.address.toField(), repeatedSelfAdddress, amounts, ownerAddress, noteOffsets)
        .send()
        .wait({ timeout: 100 }); // mining timeout ≥ time needed for the test to finish.

      await expectBalance(zkTokenContract, ownerAddress, initialBalance);
      await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 16);
    }

    // Transaction 2
    logger(`transfer()`);
    {
      const transferAmount = 400n + 50n * 7n + 40n;
      const recipient = recipients[0];
      await expectBalance(zkTokenContract, recipient, 0n);

      await zkTokenContract.methods.transfer(transferAmount, recipient).send().wait();

      await expectBalance(zkTokenContract, ownerAddress, initialBalance - transferAmount);
      await expectBalance(zkTokenContract, recipient, transferAmount);
      await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
    }
  }, 100_000);
});
