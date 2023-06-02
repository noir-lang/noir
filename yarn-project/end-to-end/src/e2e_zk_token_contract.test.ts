import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, TxStatus } from '@aztec/aztec.js';
import { ZkTokenContractAbi } from '@aztec/noir-contracts/examples';

import { DebugLogger } from '@aztec/foundation/log';
import { pointToPublicKey, setup } from './utils.js';

describe('e2e_zk_token_contract', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  let contract: Contract;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, logger } = await setup(2));
  }, 30_000);

  afterEach(async () => {
    await aztecNode?.stop();
    await aztecRpcServer?.stop();
  });

  const expectBalance = async (owner: AztecAddress, expectedBalance: bigint) => {
    const ownerPublicKey = await aztecRpcServer.getAccountPublicKey(owner);
    const [balance] = await contract.methods.getBalance(pointToPublicKey(ownerPublicKey)).view({ from: owner });
    logger(`Account ${owner} balance: ${balance}`);
    expect(balance).toBe(expectedBalance);
  };

  const deployContract = async (initialBalance = 0n, owner = { x: 0n, y: 0n }) => {
    logger(`Deploying L2 contract...`);
    const deployer = new ContractDeployer(ZkTokenContractAbi, aztecRpcServer);
    const tx = deployer.deploy(initialBalance, owner).send();
    const receipt = await tx.getReceipt();
    contract = new Contract(receipt.contractAddress!, ZkTokenContractAbi, aztecRpcServer);
    await tx.isMined(0, 0.1);
    await tx.getReceipt();
    logger('L2 contract deployed');
    return contract;
  };

  /**
   * Milestone 1.3.
   * https://hackmd.io/AG5rb9DyTRu3y7mBptWauA
   */
  it('1.3 should deploy zk token contract with initial token minted to the account', async () => {
    const initialBalance = 987n;
    const owner = await aztecRpcServer.getAccountPublicKey(accounts[0]);
    await deployContract(initialBalance, pointToPublicKey(owner));
    await expectBalance(accounts[0], initialBalance);
    await expectBalance(accounts[1], 0n);
  }, 30_000);

  /**
   * Milestone 1.4.
   */
  it('1.4 should call mint and increase balance', async () => {
    const mintAmount = 65n;

    const [owner, receiver] = accounts;

    const deployedContract = await deployContract(
      0n,
      pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)),
    );
    await expectBalance(owner, 0n);
    await expectBalance(receiver, 0n);

    const tx = deployedContract.methods
      .mint(mintAmount, pointToPublicKey(await aztecRpcServer.getAccountPublicKey(receiver)))
      .send({ from: receiver });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
    await expectBalance(receiver, mintAmount);
  }, 60_000);

  /**
   * Milestone 1.5.
   */
  it('1.5 should call transfer and increase balance of another account', async () => {
    const initialBalance = 987n;
    const transferAmount = 654n;
    const [owner, receiver] = accounts;

    await deployContract(initialBalance, pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)));

    await expectBalance(owner, initialBalance);
    await expectBalance(receiver, 0n);

    const tx = contract.methods
      .transfer(
        transferAmount,
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(owner)),
        pointToPublicKey(await aztecRpcServer.getAccountPublicKey(receiver)),
      )
      .send({ from: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);

    await expectBalance(owner, initialBalance - transferAmount);
    await expectBalance(receiver, transferAmount);
  }, 60_000);
});
