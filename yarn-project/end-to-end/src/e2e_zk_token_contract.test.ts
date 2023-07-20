import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { ZkTokenContract } from '@aztec/noir-contracts/types';
import { AztecRPC, TxStatus } from '@aztec/types';

import { expectUnencryptedLogsFromLastBlockToBe, expectsNumOfEncryptedLogsInTheLastBlockToBe, setup } from './utils.js';

describe('e2e_zk_token_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let wallet: Wallet;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let contract: ZkTokenContract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, wallet, logger } = await setup(2));
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const [balance] = await contract.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployContract = async (initialBalance: bigint, owner: AztecAddress) => {
    logger(`Deploying L2 contract...`);
    const tx = ZkTokenContract.deploy(aztecRpcServer, initialBalance, owner).send();
    const receipt = await tx.getReceipt();
    contract = new ZkTokenContract(receipt.contractAddress!, wallet);
    await tx.isMined(0, 0.1);
    const minedReceipt = await tx.getReceipt();
    expect(minedReceipt.status).toEqual(TxStatus.MINED);
    logger('L2 contract deployed');
    return contract;
  };

  /**
   * Milestone 1.3.
   * https://hackmd.io/AG5rb9DyTRu3y7mBptWauA
   */
  it('1.3 should deploy zk token contract with initial token minted to the account', async () => {
    const initialBalance = 987n;
    await deployContract(initialBalance, accounts[0]);
    await expectBalance(accounts[0], initialBalance);
    await expectBalance(accounts[1], 0n);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Balance set in constructor']);
  }, 30_000);

  /**
   * Milestone 1.4.
   */
  it('1.4 should call mint and increase balance', async () => {
    const mintAmount = 65n;

    const [owner] = accounts;

    const deployedContract = await deployContract(0n, owner);
    await expectBalance(owner, 0n);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 0);

    const tx = deployedContract.methods.mint(mintAmount, owner).send({ origin: owner });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    await expectBalance(owner, mintAmount);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Coins minted']);
  }, 60_000);

  /**
   * Milestone 1.5.
   */
  it('1.5 should call transfer and increase balance of another account', async () => {
    const initialBalance = 987n;
    const transferAmount = 654n;
    const [owner, receiver] = accounts;

    await deployContract(initialBalance, owner);

    await expectBalance(owner, initialBalance);
    await expectBalance(receiver, 0n);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Balance set in constructor']);

    const tx = contract.methods.transfer(transferAmount, owner, receiver).send({ origin: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);

    await expectBalance(owner, initialBalance - transferAmount);
    await expectBalance(receiver, transferAmount);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
    await expectUnencryptedLogsFromLastBlockToBe(aztecNode, ['Coins transferred']);
  }, 60_000);
});
