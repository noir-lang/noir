import { AztecAddress, Wallet } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { PrivateTokenContract } from '@aztec/noir-contracts/types';
import { AztecNode, CompleteAddress, TxStatus } from '@aztec/types';

import { expectsNumOfEncryptedLogsInTheLastBlockToBe, setup } from './fixtures/utils.js';

describe('e2e_private_token_contract', () => {
  let aztecNode: AztecNode | undefined;
  let wallet: Wallet;
  let logger: DebugLogger;
  let owner: AztecAddress;
  let receiver: AztecAddress;
  let teardown: () => Promise<void>;
  let contract: PrivateTokenContract;

  beforeEach(async () => {
    let accounts: CompleteAddress[];
    ({ teardown, aztecNode, accounts, wallet, logger } = await setup(2));
    owner = accounts[0].address;
    receiver = accounts[1].address;
  }, 100_000);

  afterEach(() => teardown());

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const balance = await contract.methods.getBalance(owner).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployContract = async (initialBalance: bigint, owner: AztecAddress) => {
    logger(`Deploying L2 contract...`);
    contract = await PrivateTokenContract.deploy(wallet, initialBalance, owner).send().deployed();
    logger(`L2 contract deployed at ${contract.address}`);
  };

  /**
   * Milestone 1.3.
   * https://hackmd.io/AG5rb9DyTRu3y7mBptWauA
   */
  it('1.3 should deploy private token contract with initial token minted to the account', async () => {
    const initialBalance = 987n;
    await deployContract(initialBalance, owner);
    await expectBalance(owner, initialBalance);
    await expectBalance(receiver, 0n);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
  }, 30_000);

  /**
   * Milestone 1.4.
   */
  it('1.4 should call mint and increase balance', async () => {
    const mintAmount = 65n;

    await deployContract(0n, owner);
    await expectBalance(owner, 0n);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 0);

    const tx = contract.methods.mint(mintAmount, owner).send();

    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    await expectBalance(owner, mintAmount);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);
  }, 60_000);

  /**
   * Milestone 1.5.
   */
  it('1.5 should call transfer and increase balance of another account', async () => {
    const initialBalance = 987n;
    const transferAmount = 654n;

    await deployContract(initialBalance, owner);

    await expectBalance(owner, initialBalance);
    await expectBalance(receiver, 0n);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 1);

    const tx = contract.methods.transfer(transferAmount, receiver).send();

    await tx.isMined({ interval: 0.1 });
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);

    await expectBalance(owner, initialBalance - transferAmount);
    await expectBalance(receiver, transferAmount);

    await expectsNumOfEncryptedLogsInTheLastBlockToBe(aztecNode, 2);
  }, 60_000);
});
