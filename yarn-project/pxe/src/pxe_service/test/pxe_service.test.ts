import { type AztecNode, type PXE, TxEffect, mockTx } from '@aztec/circuit-types';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js/constants';
import { type L1ContractAddresses } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { KeyStore } from '@aztec/key-store';
import { openTmpStore } from '@aztec/kv-store/utils';

import { type MockProxy, mock } from 'jest-mock-extended';

import { KVPxeDatabase } from '../../database/kv_pxe_database.js';
import { type PxeDatabase } from '../../database/pxe_database.js';
import { type PXEServiceConfig } from '../../index.js';
import { TestPrivateKernelProver } from '../../kernel_prover/test/test_circuit_prover.js';
import { PXEService } from '../pxe_service.js';
import { pxeTestSuite } from './pxe_test_suite.js';

function createPXEService(): Promise<PXE> {
  const kvStore = openTmpStore();
  const keyStore = new KeyStore(kvStore);
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
    gasTokenAddress: EthAddress.random(),
    gasPortalAddress: EthAddress.random(),
  };
  node.getL1ContractAddresses.mockResolvedValue(mockedContracts);

  return Promise.resolve(new PXEService(keyStore, node, db, new TestPrivateKernelProver(), config));
}

pxeTestSuite('PXEService', createPXEService);

describe('PXEService', () => {
  let keyStore: KeyStore;
  let node: MockProxy<AztecNode>;
  let db: PxeDatabase;
  let config: PXEServiceConfig;

  beforeEach(() => {
    const kvStore = openTmpStore();
    keyStore = new KeyStore(kvStore);
    node = mock<AztecNode>();
    db = new KVPxeDatabase(kvStore);
    config = { l2BlockPollingIntervalMS: 100, l2StartingBlock: INITIAL_L2_BLOCK_NUM, proverEnabled: false };
  });

  it('throws when submitting a tx with a nullifier of already settled tx', async () => {
    const settledTx = TxEffect.random();
    const duplicateTx = mockTx();

    node.getTxEffect.mockResolvedValue(settledTx);

    const pxe = new PXEService(keyStore, node, db, new TestPrivateKernelProver(), config);
    await expect(pxe.sendTx(duplicateTx)).rejects.toThrow(/A settled tx with equal hash/);
  });
});
