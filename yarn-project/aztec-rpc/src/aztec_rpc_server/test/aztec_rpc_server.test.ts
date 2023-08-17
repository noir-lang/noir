import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { TestKeyStore } from '@aztec/key-store';
import { AztecNode, AztecRPC } from '@aztec/types';

import { mock } from 'jest-mock-extended';

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
  node.getBlockHeight.mockResolvedValue(2);
  node.getVersion.mockResolvedValue(1);
  node.getChainId.mockResolvedValue(1);
  node.getRollupAddress.mockResolvedValue(EthAddress.random());

  return new AztecRPCServer(keyStore, node, db, config);
}

aztecRpcTestSuite('AztecRPCServer', createAztecRpcServer);
