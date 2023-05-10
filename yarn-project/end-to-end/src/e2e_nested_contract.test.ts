import { AztecNode, getConfigEnvVars } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer, Contract, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { ContractAbi } from '@aztec/foundation/abi';
import { ChildAbi, ParentAbi } from '@aztec/noir-contracts/examples';

import { mnemonicToAccount } from 'viem/accounts';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';
import { deployL1Contracts } from './deploy_l1_contracts.js';

const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_nested_contract');

const config = getConfigEnvVars();

describe('e2e_nested_contract', () => {
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
    await node.stop();
    await aztecRpcServer.stop();
  });

  const deployContract = async (abi: ContractAbi) => {
    logger(`Deploying L2 contract ${abi.name}...`);
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const tx = deployer.deploy().send();

    await tx.isMined(0, 0.1);

    const receipt = await tx.getReceipt();
    const contract = new Contract(receipt.contractAddress!, abi, aztecRpcServer);
    logger('L2 contract deployed');
    return contract;
  };

  const addressToField = (address: AztecAddress): bigint => {
    return Fr.fromBuffer(address.toBuffer()).value;
  };

  /**
   * Milestone 3.
   */
  it('should mine transactions that perform nested calls', async () => {
    const parentContract = await deployContract(ParentAbi);
    const childContract = await deployContract(ChildAbi);

    logger('Parent & Child contracts deployed');

    const tx = parentContract.methods
      .entryPoint(addressToField(childContract.address), Fr.fromBuffer(childContract.methods.value.selector).value)
      .send({ from: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 100_000);

  it('should mine transactions that perform public nested calls', async () => {
    const parentContract = await deployContract(ParentAbi);
    const childContract = await deployContract(ChildAbi);

    logger('Parent & Child contracts deployed');

    const tx = parentContract.methods
      .pubEntryPoint(
        addressToField(childContract.address),
        Fr.fromBuffer(childContract.methods.pubValue.selector).value,
        42n,
      )
      .send({ from: accounts[0] });

    await tx.isMined(0, 0.1);
    const receipt = await tx.getReceipt();

    expect(receipt.status).toBe(TxStatus.MINED);
  }, 100_000);
});
