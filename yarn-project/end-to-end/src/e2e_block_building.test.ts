import { AztecNode, getConfigEnvVars } from '@aztec/aztec-node';
import { AztecRPCServer, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';

import { mnemonicToAccount } from 'viem/accounts';
import { createAztecRpcServer } from './create_aztec_rpc_client.js';
import { deployL1Contracts } from './deploy_l1_contracts.js';
import { ContractAbi } from '@aztec/noir-contracts';
import { TestContractAbi } from '@aztec/noir-contracts/examples';
import times from 'lodash.times';
import { createDebugLogger } from '@aztec/foundation/log';

const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_block_building');

const config = getConfigEnvVars();

describe('e2e_block_building', () => {
  let node: AztecNode;
  let aztecRpcServer: AztecRPCServer;

  const abi = TestContractAbi as ContractAbi;

  beforeEach(async () => {
    const account = mnemonicToAccount(MNEMONIC);
    const privKey = account.getHdKey().privateKey;
    console.log(config);
    const { rollupAddress, unverifiedDataEmitterAddress } = await deployL1Contracts(config.rpcUrl, account, logger);

    config.publisherPrivateKey = Buffer.from(privKey!);
    config.rollupContract = rollupAddress;
    config.unverifiedDataEmitterContract = unverifiedDataEmitterAddress;

    node = await AztecNode.createAndSync(config);
    aztecRpcServer = await createAztecRpcServer(1, node);
  }, 60_000);

  afterEach(async () => {
    await node?.stop();
    await aztecRpcServer?.stop();
  });

  it('should assemble a block with multiple txs', async () => {
    // Assemble 10 contract deployment txs
    // We need to create them sequentially since we cannot have parallel calls to a circuit
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const methods = times(10, () => deployer.deploy());

    for (const i in methods) {
      await methods[i].create({ contractAddressSalt: new Fr(BigInt(i + 1)) });
    }

    // Send them simultaneously to be picked up by the sequencer
    const txs = await Promise.all(methods.map(method => method.send()));
    logger(`Txs sent with hashes: `);
    for (const tx of txs) logger(` ${await tx.getTxHash()}`);

    // Await txs to be mined and assert they are all mined on the same block
    await Promise.all(txs.map(tx => tx.isMined()));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
    expect(receipts.map(r => r.status)).toEqual(times(10, () => TxStatus.MINED));
    expect(receipts.map(r => r.blockNumber)).toEqual(times(10, () => receipts[0].blockNumber));

    // Assert all contracts got deployed
    const areDeployed = await Promise.all(receipts.map(r => aztecRpcServer.isContractDeployed(r.contractAddress!)));
    expect(areDeployed).toEqual(times(10, () => true));
  }, 60_000);
});
