import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { TestKeyStore } from '@aztec/key-store';
import { AztecNode, AztecRPC, L2Tx, mockTx } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { MemoryDB } from '../../database/memory_db.js';
import { EthAddress, RpcServerConfig } from '../../index.js';
import { AztecRPCServer } from '../aztec_rpc_server.js';
import { aztecRpcTestSuite } from './aztec_rpc_test_suite.js';

async function createAztecRpcServer(): Promise<AztecRPC> {
  const keyStore = new TestKeyStore(await Grumpkin.new());
  const node = mock<AztecNode>();
  const db = new MemoryDB();
  const config: RpcServerConfig = {
    l2BlockPollingIntervalMS: 100,
  };

  // Setup the relevant mocks
  node.getBlockNumber.mockResolvedValue(2);
  node.getVersion.mockResolvedValue(1);
  node.getChainId.mockResolvedValue(1);
  node.getRollupAddress.mockResolvedValue(EthAddress.random());

  return new AztecRPCServer(keyStore, node, db, config);
}

aztecRpcTestSuite('AztecRPCServer', createAztecRpcServer);

describe('AztecRPCServer', () => {
  let keyStore: TestKeyStore;
  let node: MockProxy<AztecNode>;
  let db: MemoryDB;
  let config: RpcServerConfig;

  beforeEach(async () => {
    keyStore = new TestKeyStore(await Grumpkin.new());
    node = mock<AztecNode>();
    db = new MemoryDB();
    config = {
      l2BlockPollingIntervalMS: 100,
    };
  });

  it('throws when submitting a tx with a nullifier of already settled tx', async () => {
    const settledTx = L2Tx.random();
    const duplicateTx = mockTx();

    node.getTx.mockResolvedValue(settledTx);

    const rpc = new AztecRPCServer(keyStore, node, db, config);
    await expect(rpc.sendTx(duplicateTx)).rejects.toThrowError(/A settled tx with equal hash/);
  });
});
