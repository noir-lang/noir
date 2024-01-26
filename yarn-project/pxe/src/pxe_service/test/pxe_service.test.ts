import { AztecNode, INITIAL_L2_BLOCK_NUM, L2Tx, PXE, mockTx } from '@aztec/circuit-types';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { L1ContractAddresses } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { TestKeyStore } from '@aztec/key-store';
import { AztecLmdbStore } from '@aztec/kv-store';

import { MockProxy, mock } from 'jest-mock-extended';

import { KVPxeDatabase } from '../../database/kv_pxe_database.js';
import { PxeDatabase } from '../../database/pxe_database.js';
import { PXEServiceConfig } from '../../index.js';
import { PXEService } from '../pxe_service.js';
import { pxeTestSuite } from './pxe_test_suite.js';

async function createPXEService(): Promise<PXE> {
  const kvStore = await AztecLmdbStore.openTmp();
  const keyStore = new TestKeyStore(new Grumpkin(), kvStore);
  const node = mock<AztecNode>();
  const db = new KVPxeDatabase(kvStore);
  const config: PXEServiceConfig = { l2BlockPollingIntervalMS: 100, l2StartingBlock: INITIAL_L2_BLOCK_NUM };

  // Setup the relevant mocks
  node.getBlockNumber.mockResolvedValue(2);
  node.getVersion.mockResolvedValue(1);
  node.getChainId.mockResolvedValue(1);
  const mockedContracts: L1ContractAddresses = {
    availabilityOracleAddress: EthAddress.random(),
    rollupAddress: EthAddress.random(),
    registryAddress: EthAddress.random(),
    inboxAddress: EthAddress.random(),
    outboxAddress: EthAddress.random(),
    contractDeploymentEmitterAddress: EthAddress.random(),
  };
  node.getL1ContractAddresses.mockResolvedValue(mockedContracts);

  return Promise.resolve(new PXEService(keyStore, node, db, config));
}

pxeTestSuite('PXEService', createPXEService);

describe('PXEService', () => {
  let keyStore: TestKeyStore;
  let node: MockProxy<AztecNode>;
  let db: PxeDatabase;
  let config: PXEServiceConfig;

  beforeEach(async () => {
    const kvStore = await AztecLmdbStore.openTmp();
    keyStore = new TestKeyStore(new Grumpkin(), kvStore);
    node = mock<AztecNode>();
    db = new KVPxeDatabase(kvStore);
    config = { l2BlockPollingIntervalMS: 100, l2StartingBlock: INITIAL_L2_BLOCK_NUM };
  });

  it('throws when submitting a tx with a nullifier of already settled tx', async () => {
    const settledTx = L2Tx.random();
    const duplicateTx = mockTx();

    node.getTx.mockResolvedValue(settledTx);

    const pxe = new PXEService(keyStore, node, db, config);
    await expect(pxe.sendTx(duplicateTx)).rejects.toThrowError(/A settled tx with equal hash/);
  });
});
