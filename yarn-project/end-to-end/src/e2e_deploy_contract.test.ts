import { AztecAddress, Contract, ContractDeployer, Fr, Wallet, isContractDeployed } from '@aztec/aztec.js';
import { CompleteAddress, getContractDeploymentInfo } from '@aztec/circuits.js';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContractAbi } from '@aztec/noir-contracts/artifacts';
import { AztecRPC, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_deploy_contract', () => {
  let aztecRpcServer: AztecRPC;
  let accounts: CompleteAddress[];
  let logger: DebugLogger;
  let wallet: Wallet;
  let teardown: () => Promise<void>;

  beforeEach(async () => {
    ({ teardown, aztecRpcServer, accounts, logger, wallet } = await setup());
  }, 100_000);

  afterEach(() => teardown());

  /**
   * Milestone 1.1.
   * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
   */
  it('should deploy a contract', async () => {
    const publicKey = accounts[0].publicKey;
    const salt = Fr.random();
    const deploymentData = await getContractDeploymentInfo(TestContractAbi, [], salt, publicKey);
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer, publicKey);
    const tx = deployer.deploy().send({ contractAddressSalt: salt });
    logger(`Tx sent with hash ${await tx.getTxHash()}`);
    const receipt = await tx.getReceipt();
    expect(receipt).toEqual(
      expect.objectContaining({
        status: TxStatus.PENDING,
        error: '',
      }),
    );
    logger(`Receipt received and expecting contract deployment at ${receipt.contractAddress}`);
    const isMined = await tx.isMined({ interval: 0.1 });
    const receiptAfterMined = await tx.getReceipt();

    expect(isMined).toBe(true);
    expect(receiptAfterMined).toEqual(
      expect.objectContaining({
        status: TxStatus.MINED,
        error: '',
        contractAddress: deploymentData.completeAddress.address,
      }),
    );
    const contractAddress = receiptAfterMined.contractAddress!;
    expect(await isContractDeployed(aztecRpcServer, contractAddress)).toBe(true);
    expect(await isContractDeployed(aztecRpcServer, AztecAddress.random())).toBe(false);
  }, 30_000);

  /**
   * Verify that we can produce multiple rollups.
   */
  it('should deploy one contract after another in consecutive rollups', async () => {
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer);

    for (let index = 0; index < 2; index++) {
      logger(`Deploying contract ${index + 1}...`);
      const tx = deployer.deploy().send({ contractAddressSalt: Fr.random() });
      const isMined = await tx.isMined({ interval: 0.1 });
      expect(isMined).toBe(true);
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
    }
  }, 30_000);

  /**
   * Verify that we can deploy multiple contracts and interact with all of them.
   */
  it('should deploy multiple contracts and interact with them', async () => {
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer);

    for (let index = 0; index < 2; index++) {
      logger(`Deploying contract ${index + 1}...`);
      const receipt = await deployer.deploy().send({ contractAddressSalt: Fr.random() }).wait({ wallet });

      const contract = await Contract.at(receipt.contractAddress!, TestContractAbi, wallet);
      logger(`Sending TX to contract ${index + 1}...`);
      await contract.methods.getPublicKey(accounts[0].address).send().wait();
    }
  }, 30_000);

  /**
   * Milestone 1.2.
   * https://hackmd.io/-a5DjEfHTLaMBR49qy6QkA
   */
  it('should not deploy a contract with the same salt twice', async () => {
    const contractAddressSalt = Fr.random();
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer);

    {
      const tx = deployer.deploy().send({ contractAddressSalt });
      const isMined = await tx.isMined({ interval: 0.1 });

      expect(isMined).toBe(true);
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.MINED);
      expect(receipt.error).toBe('');
    }

    {
      await expect(deployer.deploy().send({ contractAddressSalt }).wait()).rejects.toThrowError(
        /A settled tx with equal hash/,
      );
    }
  }, 30_000);
});
