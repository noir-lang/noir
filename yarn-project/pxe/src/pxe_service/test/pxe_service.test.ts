import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { L1ContractAddresses } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { TestKeyStore } from '@aztec/key-store';
import { AztecNode, L2Tx, PXE, mockTx } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { MemoryDB } from '../../database/memory_db.js';
import { PXEServiceConfig } from '../../index.js';
import { PXEService } from '../pxe_service.js';
import { pxeTestSuite } from './pxe_test_suite.js';

async function createPXEService(): Promise<PXE> {
  const keyStore = new TestKeyStore(await Grumpkin.new());
  const node = mock<AztecNode>();
  const db = new MemoryDB();
  const config: PXEServiceConfig = {
    l2BlockPollingIntervalMS: 100,
  };

  // Setup the relevant mocks
  node.getBlockNumber.mockResolvedValue(2);
  node.getVersion.mockResolvedValue(1);
  node.getChainId.mockResolvedValue(1);
  const mockedContracts: L1ContractAddresses = {
    rollupAddress: EthAddress.random(),
    registryAddress: EthAddress.random(),
    inboxAddress: EthAddress.random(),
    outboxAddress: EthAddress.random(),
    contractDeploymentEmitterAddress: EthAddress.random(),
    decoderHelperAddress: EthAddress.random(),
  };
  node.getL1ContractAddresses.mockResolvedValue(mockedContracts);

  return new PXEService(keyStore, node, db, config);
}

pxeTestSuite('PXEService', createPXEService);

describe('PXEService', () => {
  let keyStore: TestKeyStore;
  let node: MockProxy<AztecNode>;
  let db: MemoryDB;
  let config: PXEServiceConfig;

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

    const rpc = new PXEService(keyStore, node, db, config);
    await expect(rpc.sendTx(duplicateTx)).rejects.toThrowError(/A settled tx with equal hash/);
  });
});
