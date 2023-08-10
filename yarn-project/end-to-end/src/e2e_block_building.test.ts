import { AztecNodeService } from '@aztec/aztec-node';
import { AztecRPCServer } from '@aztec/aztec-rpc';
import { ContractDeployer, Fr, isContractDeployed } from '@aztec/aztec.js';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContractAbi } from '@aztec/noir-contracts/artifacts';
import { AztecRPC, TxStatus } from '@aztec/types';

import times from 'lodash.times';

import { setup } from './fixtures/utils.js';

describe('e2e_block_building', () => {
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;
  let logger: DebugLogger;

  const abi = TestContractAbi;

  beforeEach(async () => {
    ({ aztecNode, aztecRpcServer, logger } = await setup());
  }, 100_000);

  afterEach(async () => {
    await aztecNode?.stop();
    if (aztecRpcServer instanceof AztecRPCServer) {
      await aztecRpcServer?.stop();
    }
  });

  it('should assemble a block with multiple txs', async () => {
    // Assemble N contract deployment txs
    // We need to create them sequentially since we cannot have parallel calls to a circuit
    const TX_COUNT = 8;
    const deployer = new ContractDeployer(abi, aztecRpcServer);
    const methods = times(TX_COUNT, () => deployer.deploy());

    for (const i in methods) {
      await methods[i].create({ contractAddressSalt: new Fr(BigInt(i + 1)) });
      await methods[i].simulate({});
    }

    // Send them simultaneously to be picked up by the sequencer
    const txs = await Promise.all(methods.map(method => method.send()));
    logger(`Txs sent with hashes: `);
    for (const tx of txs) logger(` ${await tx.getTxHash()}`);

    // Await txs to be mined and assert they are all mined on the same block
    await Promise.all(txs.map(tx => tx.isMined()));
    const receipts = await Promise.all(txs.map(tx => tx.getReceipt()));
    expect(receipts.map(r => r.status)).toEqual(times(TX_COUNT, () => TxStatus.MINED));
    expect(receipts.map(r => r.blockNumber)).toEqual(times(TX_COUNT, () => receipts[0].blockNumber));

    // Assert all contracts got deployed
    const areDeployed = await Promise.all(receipts.map(r => isContractDeployed(aztecRpcServer, r.contractAddress!)));
    expect(areDeployed).toEqual(times(TX_COUNT, () => true));
  }, 60_000);
});
