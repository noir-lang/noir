import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer, ContractDeployer, Fr, TxStatus } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContractAbi } from '@aztec/noir-contracts/examples';
import times from 'lodash.times';
import { setup } from './utils.js';

describe('e2e_block_building', () => {
  let aztecNode: AztecNodeService;
  let aztecRpcServer: AztecRPCServer;
  let logger: DebugLogger;

  const abi = TestContractAbi;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
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
