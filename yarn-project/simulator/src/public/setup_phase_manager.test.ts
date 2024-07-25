import { mockTx } from '@aztec/circuit-types';
import { GlobalVariables, Header } from '@aztec/circuits.js';
import { type PublicExecutor } from '@aztec/simulator';
import { type MerkleTreeOperations, type TreeInfo } from '@aztec/world-state';

import { it } from '@jest/globals';
import { type MockProxy, mock } from 'jest-mock-extended';

import { type ContractsDataSourcePublicDB, type WorldStatePublicDB } from './public_db_sources.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';
import { SetupPhaseManager } from './setup_phase_manager.js';

class TestSetupPhaseManager extends SetupPhaseManager {
  override extractEnqueuedPublicCalls(tx: any) {
    return super.extractEnqueuedPublicCalls(tx);
  }
}

describe('setup_phase_manager', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let publicExecutor: MockProxy<PublicExecutor>;
  let publicContractsDB: MockProxy<ContractsDataSourcePublicDB>;
  let publicWorldStateDB: MockProxy<WorldStatePublicDB>;
  let publicKernel: MockProxy<PublicKernelCircuitSimulator>;

  let root: Buffer;

  let phaseManager: TestSetupPhaseManager;

  beforeEach(() => {
    db = mock<MerkleTreeOperations>();
    publicExecutor = mock<PublicExecutor>();
    publicContractsDB = mock<ContractsDataSourcePublicDB>();
    publicWorldStateDB = mock<WorldStatePublicDB>();

    root = Buffer.alloc(32, 5);
    db.getTreeInfo.mockResolvedValue({ root } as TreeInfo);
    publicKernel = mock<PublicKernelCircuitSimulator>();
    phaseManager = new TestSetupPhaseManager(
      db,
      publicExecutor,
      publicKernel,
      GlobalVariables.empty(),
      Header.empty(),
      publicContractsDB,
      publicWorldStateDB,
    );
  });

  it('does not extract non-revertible calls when none exist', function () {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0, numberOfRevertiblePublicCallRequests: 0 });

    const enqueuedNonRevertibleCalls = phaseManager.extractEnqueuedPublicCalls(tx);

    expect(enqueuedNonRevertibleCalls).toEqual([]);
  });
});
