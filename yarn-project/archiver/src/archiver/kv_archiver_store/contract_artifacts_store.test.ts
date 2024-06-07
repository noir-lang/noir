import { AztecAddress } from '@aztec/circuits.js';
import { openTmpStore } from '@aztec/kv-store/utils';
import { BenchmarkingContractArtifact } from '@aztec/noir-contracts.js/Benchmarking';

import { beforeEach } from '@jest/globals';

import { KVArchiverDataStore } from './kv_archiver_store.js';

describe('Contract Artifact Store', () => {
  let archiverStore: KVArchiverDataStore;

  beforeEach(() => {
    archiverStore = new KVArchiverDataStore(openTmpStore());
  });

  it('Should add and return contract artifacts', async () => {
    const artifact = BenchmarkingContractArtifact;
    const address = AztecAddress.random();
    await archiverStore.addContractArtifact(address, artifact);
    await expect(archiverStore.getContractArtifact(address)).resolves.toEqual(artifact);
  });
});
