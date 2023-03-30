import { AztecNode } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, ContractAbi, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';
import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { createDebugLogger } from '@aztec/foundation';
import { TestContractAbi } from '@aztec/noir-contracts/examples';
import { createAztecNode } from './create_aztec_node.js';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';
import { createProvider, deployRollupContract, deployYeeterContract } from './deploy_l1_contracts.js';

const { ETHEREUM_HOST = 'http://localhost:8545' } = process.env;
const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_deploy_contract');
describe('e2e_deploy_contract', () => {
  let provider: WalletProvider;
  let node: AztecNode;
  let aztecRpcServer: AztecRPCServer;
  let rollupAddress: EthAddress;
  let yeeterAddress: EthAddress;
  let accounts: AztecAddress[];
  const abi = TestContractAbi as ContractAbi;

  beforeAll(async () => {
    provider = createProvider(ETHEREUM_HOST, MNEMONIC, 1);
    const ethRpc = new EthereumRpc(provider);
    logger('Deploying contracts...');
    rollupAddress = await deployRollupContract(provider, ethRpc);
    yeeterAddress = await deployYeeterContract(provider, ethRpc);
    logger('Deployed contracts...');
  });

  beforeEach(async () => {
    node = await createAztecNode(rollupAddress, yeeterAddress, ETHEREUM_HOST, provider.getPrivateKey(0)!);
    aztecRpcServer = await createAztecRpcServer(1, node);
    accounts = await aztecRpcServer.getAccounts();
  });

  afterEach(async () => {
    await node.stop();
    await aztecRpcServer.stop();
  });

  /**
   * Milestone 1.1
   * https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#Interfaces-and-Responsibilities
   */
  it('should deploy a contract', async () => {
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const tx = deployer.deploy().send();
    logger(`Tx sent!`);
    const receipt = await tx.getReceipt();
    expect(receipt).toEqual(
      expect.objectContaining({
        from: accounts[0],
        to: undefined,
        status: TxStatus.PENDING,
        error: '',
      }),
    );
    logger(`Receipt received`);
    const isMined = await tx.isMined();
    const receiptAfterMined = await tx.getReceipt();

    expect(isMined).toBe(true);
    expect(receiptAfterMined.status).toBe(TxStatus.MINED);
    const contractAddress = receipt.contractAddress!;
    expect(await aztecRpcServer.isContractDeployed(contractAddress)).toBe(true);
    expect(await aztecRpcServer.isContractDeployed(AztecAddress.random())).toBe(false);
  }, 30_000);

  /**
   * Milestone 1.2
   * https://hackmd.io/-a5DjEfHTLaMBR49qy6QkA
   */
  it.skip('should not deploy a contract with the same salt twice', async () => {
    const contractAddressSalt = Fr.random();
    const deployer = new ContractDeployer(abi, aztecRpcServer);

    {
      const tx = deployer.deploy().send({ contractAddressSalt });
      const isMined = await tx.isMined();
      expect(isMined).toBe(true);

      const receipt = await tx.getReceipt();
      expect(receipt.status).toBe(TxStatus.MINED);
      expect(receipt.error).toBe('');
    }

    {
      const tx = deployer.deploy().send({ contractAddressSalt });
      const isMined = await tx.isMined();
      expect(isMined).toBe(false);

      const receipt = await deployer.deploy().send({ contractAddressSalt }).getReceipt();
      expect(receipt.status).toBe(TxStatus.DROPPED);
      expect(receipt.error).not.toBe('');
    }
  });
});
