import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { AztecAddress, ContractDeployer, Fr, isContractDeployed } from '@aztec/aztec.js';
import { getContractDeploymentInfo } from '@aztec/circuits.js';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContractAbi } from '@aztec/noir-contracts/artifacts';
import { AztecRPC, TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

describe('e2e_deploy_contract', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let accounts: AztecAddress[];
  let logger: DebugLogger;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, accounts, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  /**
   * Milestone 1.1.
   * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
   */
  it('should deploy a contract', async () => {
    const publicKey = (await aztecRpcServer.getPublicKeyAndPartialAddress(accounts[0]))[0];
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
        contractAddress: deploymentData.address,
      }),
    );
    logger(`Receipt received and expecting contract deployment at ${receipt.contractAddress}`);
    const isMined = await tx.isMined({ interval: 0.1 });
    const receiptAfterMined = await tx.getReceipt();

    expect(isMined).toBe(true);
    expect(receiptAfterMined.status).toBe(TxStatus.MINED);
    const contractAddress = receipt.contractAddress!;
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
      const tx = deployer.deploy().send({ contractAddressSalt });
      const isMined = await tx.isMined({ interval: 0.1 });
      expect(isMined).toBe(false);
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.DROPPED);
      expect(receipt.error).toBe('Tx dropped by P2P node.');
    }
  }, 30_000);
});
