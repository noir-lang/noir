import { AztecNode, getConfigEnvVars } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation';
import { TestContractAbi } from '@aztec/noir-contracts/examples';

import { mnemonicToAccount } from 'viem/accounts';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';
import { deployL1Contracts } from './deploy_l1_contracts.js';

const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_deploy_contract');

const config = getConfigEnvVars();

describe('e2e_deploy_contract', () => {
  let node: AztecNode;
  let aztecRpcServer: AztecRPCServer;
  let accounts: AztecAddress[];

  beforeEach(async () => {
    const account = mnemonicToAccount(MNEMONIC);
    const privKey = account.getHdKey().privateKey;
    const { rollupAddress, unverifiedDataEmitterAddress } = await deployL1Contracts(config.rpcUrl, account, logger);

    config.publisherPrivateKey = Buffer.from(privKey!);
    config.rollupContract = rollupAddress;
    config.unverifiedDataEmitterContract = unverifiedDataEmitterAddress;

    node = await AztecNode.createAndSync(config);
    aztecRpcServer = await createAztecRpcServer(1, node);
    accounts = await aztecRpcServer.getAccounts();
  }, 60_000);

  afterEach(async () => {
    await node?.stop();
    await aztecRpcServer?.stop();
  });

  /**
   * Milestone 1.1
   * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
   */
  it('should deploy a contract', async () => {
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer);
    const tx = deployer.deploy().send();
    logger(`Tx sent with hash ${await tx.getTxHash()}`);
    const receipt = await tx.getReceipt();
    expect(receipt).toEqual(
      expect.objectContaining({
        from: accounts[0],
        to: undefined,
        status: TxStatus.PENDING,
        error: '',
      }),
    );
    logger(`Receipt received and expecting contract deployment at ${receipt.contractAddress}`);
    const isMined = await tx.isMined(0, 0.1);
    const receiptAfterMined = await tx.getReceipt();

    expect(isMined).toBe(true);
    expect(receiptAfterMined.status).toBe(TxStatus.MINED);
    const contractAddress = receipt.contractAddress!;
    expect(await aztecRpcServer.isContractDeployed(contractAddress)).toBe(true);
    expect(await aztecRpcServer.isContractDeployed(AztecAddress.random())).toBe(false);
  }, 30_000);

  /**
   * Verify that we can produce multiple rollups
   */
  it('should deploy one contract after another in consecutive rollups', async () => {
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer);

    for (let index = 0; index < 2; index++) {
      logger(`Deploying contract ${index + 1}...`);
      const tx = deployer.deploy().send({ contractAddressSalt: Fr.random() });
      const isMined = await tx.isMined(0, 0.1);
      expect(isMined).toBe(true);
      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
    }
  }, 30_000);

  /**
   * Milestone 1.2
   * https://hackmd.io/-a5DjEfHTLaMBR49qy6QkA
   */
  it('should not deploy a contract with the same salt twice', async () => {
    const contractAddressSalt = Fr.random();
    const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer);

    {
      const tx = deployer.deploy().send({ contractAddressSalt });
      const isMined = await tx.isMined(0, 0.1);

      expect(isMined).toBe(true);
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.MINED);
      expect(receipt.error).toBe('');
    }

    {
      const tx = deployer.deploy().send({ contractAddressSalt });
      const isMined = await tx.isMined(0, 0.1);
      expect(isMined).toBe(false);
      const receipt = await tx.getReceipt();

      expect(receipt.status).toBe(TxStatus.DROPPED);
      expect(receipt.error).toBe('Tx dropped by P2P node.');
    }
  }, 30_000);
});
